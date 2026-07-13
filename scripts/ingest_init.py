#!/usr/bin/env python3
"""Backfill: ingest all existing source files into the HLLSet lattice.

Run once to seed the lattice with the current codebase.
Re-running is safe — ingest is idempotent.

Usage:
    python scripts/ingest_init.py
"""

import sys
from pathlib import Path

# Add scripts dir to path so we can import ingest
sys.path.insert(0, str(Path(__file__).resolve().parent))

from ingest import (
    PROJECT_ROOT, META_FILE, load_metadata, save_metadata, ingest_file,
)

EXCLUDE_DIRS = {".git", "target", ".venv", "__pycache__", ".hllset_lattice",
                "node_modules", ".ipynb_checkpoints"}

EXCLUDE_FILES = {"Cargo.lock.d"}


def main():
    meta = load_metadata()
    print(f"Init ingestion — scanning {PROJECT_ROOT}")
    print(f"Existing metadata: {len(meta['files'])} files, {len(meta['commits'])} commits\n")

    count = 0
    skipped = 0

    for path in sorted(PROJECT_ROOT.rglob("*")):
        # Skip excluded dirs
        if any(excl in path.parts for excl in EXCLUDE_DIRS):
            continue
        if path.name in EXCLUDE_FILES:
            continue
        if path.is_dir():
            continue

        rel = str(path.relative_to(PROJECT_ROOT))
        # Skip already ingested (idempotency check)
        if rel in meta["files"]:
            skipped += 1
            continue

        if ingest_file(path, meta):
            count += 1

    save_metadata(meta)

    print(f"\nInit ingestion complete:")
    print(f"  New:     {count} files")
    print(f"  Skipped: {skipped} (already indexed)")
    print(f"  Total:   {len(meta['files'])} files in lattice")
    print(f"  Metadata: {META_FILE}")


if __name__ == "__main__":
    main()
