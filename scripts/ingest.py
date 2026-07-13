#!/usr/bin/env python3
"""Core ingest pipeline: file → HLLSet → store → record metadata.

Usage:
    python scripts/ingest.py <file_path>            # single file
    python scripts/ingest.py --batch <dir>           # all files under dir
    python scripts/ingest.py --commit <hash>         # process a git commit

Idempotent: re-running on the same file produces the same HLLSet key.
"""

import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

# ── Configuration ──────────────────────────────────────────────────────

PROJECT_ROOT = Path(__file__).resolve().parent.parent
HLLSET_BIN = os.environ.get("HLLSET_BINARY", str(PROJECT_ROOT / "target" / "debug" / "hllset"))
LATTICE_DIR = PROJECT_ROOT / ".hllset_lattice"
META_FILE = LATTICE_DIR / "metadata.json"

# File extensions to ingest (skip binaries, images, etc.)
CODE_EXTENSIONS = {
    ".rs", ".py", ".md", ".toml", ".lock", ".json", ".lua", ".forth",
    ".ipynb", ".sh", ".yml", ".yaml", ".txt", ".cfg", ".dockerfile",
    ".containerfile", ".gitignore", ".editorconfig", ".xml",
}

# ── Core functions ─────────────────────────────────────────────────────

def hllset_lua(script: str, timeout: float = 30.0) -> dict:
    """Execute a Lua script via the hllset CLI and return parsed JSON.

    Uses a temp file (-f) to avoid shell/argument-length limits with -e.
    """
    import tempfile
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".lua", delete=False, dir=PROJECT_ROOT
    ) as f:
        f.write(script)
        tmp_path = f.name

    try:
        proc = subprocess.run(
            [HLLSET_BIN, "-f", tmp_path],
            capture_output=True, text=True, timeout=timeout,
            cwd=str(PROJECT_ROOT),
        )
        if proc.returncode != 0:
            error = proc.stderr.strip()
            try:
                error = json.loads(error)
            except json.JSONDecodeError:
                pass
            raise RuntimeError(f"hllset error: {error}")
        return json.loads(proc.stdout.strip())
    finally:
        Path(tmp_path).unlink(missing_ok=True)


def tokenize_file(path: Path) -> dict:
    """Read a file, tokenize via hllset.tokenize() with Lua long-string.

    Uses [===[...]===] to safely pass arbitrary content including quotes,
    backslashes, and newlines.

    Returns: {"key": "h:<sha1>", "path": "...", "token_count": N, "popcount": N}
    """
    content = path.read_text(encoding="utf-8", errors="replace")

    if not content.strip():
        return {"key": None, "path": str(path), "token_count": 0, "popcount": 0}

    # Lua long-string: [=N=[content]=N=] handles arbitrary text.
    # Find the largest N where ]=N=] appears in content, then use N+1.
    n = 0
    for i in range(20):
        closing = "]" + ("=" * i) + "]"
        if closing in content:
            n = i + 1
    n = max(n, 3)  # at least 3 for safety margin
    lua_string = f"[{'=' * n}[{content}]{'=' * n}]"

    script = f"""
        local e = hllset.tokenize({lua_string})
        return {{key = e:key(), card = #e, popcount = e:popcount()}}
    """
    result = hllset_lua(script)
    return {
        "key": result.get("key") if isinstance(result, dict) else result,
        "path": str(path),
        "token_count": len(content.split()),
        "popcount": result.get("popcount", 0) if isinstance(result, dict) else 0,
    }


def load_metadata() -> dict:
    """Load or create the lattice metadata file."""
    if META_FILE.exists():
        return json.loads(META_FILE.read_text())
    return {
        "version": 1,
        "created": datetime.now(timezone.utc).isoformat(),
        "files": {},        # path → {"key": ..., "token_count": ..., ...}
        "commits": [],      # [{hash, ts, files: [key, ...], parent: key}]
        "last_commit": None,
    }


def save_metadata(meta: dict):
    """Save metadata atomically."""
    META_FILE.parent.mkdir(parents=True, exist_ok=True)
    tmp = META_FILE.with_suffix(".tmp")
    tmp.write_text(json.dumps(meta, indent=2))
    tmp.replace(META_FILE)


def ingest_file(path: Path, meta: dict) -> dict | None:
    """Ingest a single file. Returns the file record or None if skipped."""
    rel = str(path.relative_to(PROJECT_ROOT))
    ext = path.suffix.lower()

    if ext not in CODE_EXTENSIONS and path.name not in {"Dockerfile", "Containerfile", "Makefile"}:
        return None

    if not path.is_file() or path.stat().st_size == 0:
        return None

    print(f"  ingesting: {rel} ...", end=" ", flush=True)
    try:
        record = tokenize_file(path)
        record["ingested_at"] = datetime.now(timezone.utc).isoformat()
        meta["files"][rel] = record
        print(f"key={record['key'][:16]}... tokens={record['token_count']}")
        return record
    except Exception as e:
        print(f"ERROR: {e}")
        return None


def ingest_commit(commit_hash: str, meta: dict):
    """Process a git commit: compute D/R/N vs parent commit."""
    import subprocess as sp

    # Get list of changed files
    result = sp.run(
        ["git", "diff", "--name-status", f"{commit_hash}~1", commit_hash],
        capture_output=True, text=True, cwd=str(PROJECT_ROOT),
    )
    if result.returncode != 0:
        print(f"  (first commit or no parent — treating all tracked as N)")
        # Fall back to all tracked files
        result = sp.run(
            ["git", "ls-files"], capture_output=True, text=True, cwd=str(PROJECT_ROOT)
        )
        changed = [("A", f.strip()) for f in result.stdout.strip().split("\n") if f.strip()]
    else:
        changed = []
        for line in result.stdout.strip().split("\n"):
            if not line:
                continue
            parts = line.split("\t", 1)
            if len(parts) == 2:
                changed.append((parts[0], parts[1]))

    added = []
    modified = []
    deleted = []

    for status, fpath in changed:
        if status == "A":
            added.append(fpath)
        elif status == "M":
            modified.append(fpath)
        elif status == "D":
            deleted.append(fpath)
        elif status.startswith("R"):
            deleted.append(fpath)  # renamed = old deleted, new added elsewhere

    commit_record = {
        "hash": commit_hash,
        "ts": datetime.now(timezone.utc).isoformat(),
        "added": added,
        "modified": modified,
        "deleted": deleted,
        "file_keys": {},
    }

    # Ingest added and modified files
    for fpath in added + modified:
        path = PROJECT_ROOT / fpath
        if path.exists():
            record = ingest_file(path, meta)
            if record and record["key"]:
                commit_record["file_keys"][fpath] = record["key"]

    meta["commits"].append(commit_record)
    meta["last_commit"] = commit_hash

    print(f"\n  Commit {commit_hash[:8]}:")
    print(f"    Added:    {len(added)} files")
    print(f"    Modified: {len(modified)} files")
    print(f"    Deleted:  {len(deleted)} files")
    print(f"    Ingested: {len(commit_record['file_keys'])} HLLSets")

    return commit_record


def compute_drn(parent_keys: set, current_keys: set) -> dict:
    """Compute D/R/N between two sets of HLLSet keys."""
    return {
        "R": list(parent_keys & current_keys),   # Retained
        "D": list(parent_keys - current_keys),   # Departed
        "N": list(current_keys - parent_keys),   # New
    }


# ── CLI ────────────────────────────────────────────────────────────────

def cmd_ingest_file(args: list[str]):
    """Ingest a single file."""
    meta = load_metadata()
    path = Path(args[0]).resolve()
    ingest_file(path, meta)
    save_metadata(meta)
    print(f"\nMetadata saved to {META_FILE}")


def cmd_ingest_batch(args: list[str]):
    """Ingest all code files under a directory."""
    meta = load_metadata()
    root = Path(args[0]) if args else PROJECT_ROOT
    count = 0
    for path in sorted(root.rglob("*")):
        if ingest_file(path, meta):
            count += 1
    save_metadata(meta)
    print(f"\nIngested {count} files. Metadata saved to {META_FILE}")


def cmd_ingest_commit(args: list[str]):
    """Ingest a specific git commit."""
    meta = load_metadata()
    commit_hash = args[0] if args else "HEAD"
    ingest_commit(commit_hash, meta)
    save_metadata(meta)
    print(f"\nMetadata saved to {META_FILE}")


def cmd_status(args: list[str]):
    """Show ingestion status."""
    meta = load_metadata()
    print(f"Lattice state ({META_FILE}):")
    print(f"  Files indexed:  {len(meta['files'])}")
    print(f"  Commits:        {len(meta['commits'])}")
    print(f"  Last commit:    {meta.get('last_commit', 'none')}")
    if meta["files"]:
        # Show most recent 5
        print("\n  Recent ingestions:")
        sorted_files = sorted(
            meta["files"].items(),
            key=lambda x: x[1].get("ingested_at", ""),
            reverse=True,
        )
        for path, record in sorted_files[:5]:
            key = record.get("key", "?") or "?"
            print(f"    {path:50s}  key={key[:16]}...  tokens={record.get('token_count', 0)}")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)

    cmd = sys.argv[1]
    args = sys.argv[2:]

    if cmd == "file":
        cmd_ingest_file(args)
    elif cmd == "batch":
        cmd_ingest_batch(args)
    elif cmd == "commit":
        cmd_ingest_commit(args)
    elif cmd == "status":
        cmd_status(args)
    else:
        print(f"Unknown command: {cmd}")
        print("Usage: ingest.py [file|batch|commit|status] [args...]")
        sys.exit(1)
