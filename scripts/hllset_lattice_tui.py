#!/usr/bin/env python3
"""HLLSet Lattice Dashboard — unified development TUI.

Three frames:
  Left:  Changed Files (auto-detected from git)
  Right-top: Summary Bridge (prompt generation for DeepCode)
  Right-bottom: Lattice Status (Noether check, commit history)

Workflow:
  1. Develop with DeepCode in another terminal
  2. Open this TUI → review changed files
  3. "Generate Prompt" → writes to .deepcode/prompt.txt
  4. Switch to DeepCode: "read .deepcode/prompt.txt, generate summary,
     save to _SUMMARIES/..."
  5. Back to TUI → new summary appears in changed files
  6. "Commit All" → git add -A + git commit (no push, use VS Code for auth)

Usage:
    python scripts/hllset_lattice_tui.py
"""

import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

from textual.app import App, ComposeResult
from textual.containers import Horizontal, Vertical
from textual.widgets import (
    Header, Footer, Static, DataTable, Button, TextArea,
)
from textual.binding import Binding
from textual.reactive import reactive
from rich.text import Text

# ── Configuration ──────────────────────────────────────────────────────

SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = SCRIPT_DIR.parent
HLLSET_BIN = os.environ.get("HLLSET_BINARY", str(PROJECT_ROOT / "target" / "debug" / "hllset"))
PROMPT_FILE = PROJECT_ROOT / ".deepcode" / "prompt.txt"
SUMMARIES_DIR = PROJECT_ROOT / "_SUMMARIES"


# ── Git Helpers ───────────────────────────────────────────────────────

def get_changed_files() -> list[dict]:
    """Get staged + unstaged + untracked files, staged takes priority."""
    staged = {}
    unstaged = {}

    for cmd, target in [(["diff", "--cached"], staged), (["diff"], unstaged)]:
        r = subprocess.run(
            ["git", *cmd, "--name-status"],
            capture_output=True, text=True, cwd=str(PROJECT_ROOT),
        )
        for line in r.stdout.strip().split("\n"):
            if not line: continue
            parts = line.split("\t", 1)
            if len(parts) == 2:
                target[parts[1]] = parts[0]

    r = subprocess.run(
        ["git", "ls-files", "--others", "--exclude-standard"],
        capture_output=True, text=True, cwd=str(PROJECT_ROOT),
    )
    for fpath in r.stdout.strip().split("\n"):
        if fpath:
            unstaged[fpath] = "U"

    all_files = {}
    for fpath, status in unstaged.items():
        all_files[fpath] = {"status": status, "staged": False}
    for fpath, status in staged.items():
        all_files[fpath] = {"status": status, "staged": True}

    files = []
    for fpath, info in all_files.items():
        path = PROJECT_ROOT / fpath
        if path.exists() and path.is_file():
            size = path.stat().st_size
            files.append({
                "status": info["status"],
                "path": fpath,
                "size": size,
                "staged": info["staged"],
            })
    return files


def get_recent_commits(n: int = 5) -> list[str]:
    r = subprocess.run(
        ["git", "log", f"-{n}", "--oneline"],
        capture_output=True, text=True, cwd=str(PROJECT_ROOT),
    )
    return [line.strip() for line in r.stdout.strip().split("\n") if line]


def generate_prompt_file():
    """Write a structured summary request to .deepcode/prompt.txt."""
    PROMPT_FILE.parent.mkdir(parents=True, exist_ok=True)

    files = get_changed_files()
    file_list = "\n".join(
        f"  - {f['path']} ({f['status']}, {'staged' if f['staged'] else 'unstaged'})"
        for f in files
    )

    commits = get_recent_commits(5)
    commit_list = "\n".join(f"  - {c}" for c in commits)

    today = datetime.now().strftime("%Y-%m-%d")
    prompt = f"""Development session summary request — {today}

## Files Changed
{file_list if file_list else "  (none detected — run Refresh if this is wrong)"}

## Recent Commits (for context)
{commit_list}

## Instructions for DeepCode
Please generate a development session summary covering:
1. **Request** — what was asked for and why
2. **Built** — files, crates, functions, tests created or modified
3. **Decisions** — key architectural or design choices made
4. **Future Work** — open questions, next steps

Save the summary to `_SUMMARIES/{today}-<brief-slug>.md` using this format:

```
# Development Session: {today}
## Request
...
## Built
...
## Decisions
...
## Future Work
...
```

The summary will be ingested into the HLLSet lattice as an LLM context file
(l:<sha1>) and committed to the repository alongside the code changes.
"""

    PROMPT_FILE.write_text(prompt)
    return str(PROMPT_FILE)


def read_summary_files() -> list[dict]:
    """Read existing summary files from _SUMMARIES/."""
    SUMMARIES_DIR.mkdir(parents=True, exist_ok=True)
    summaries = []
    for path in sorted(SUMMARIES_DIR.glob("*.md"), reverse=True):
        summaries.append({
            "name": path.name,
            "size": path.stat().st_size,
            "preview": path.read_text()[:200].replace("\n", " "),
        })
    return summaries[:10]


# ── Textual App ───────────────────────────────────────────────────────

class HllsetLatticeApp(App):
    """Lattice dashboard with file tracking and summary bridge."""

    CSS = """
    Screen {
        layout: grid;
        grid-size: 2;
        grid-rows: 3fr 2fr;
        grid-columns: 2fr 1fr;
    }
    #files-panel {
        row-span: 2;
        border: solid $primary;
    }
    #summary-panel {
        border: solid $secondary;
    }
    #status-panel {
        border: solid $accent;
    }
    DataTable {
        height: 1fr;
    }
    .status-A { color: $success; }
    .status-M { color: $warning; }
    .status-D { color: $error; }
    .status-U { color: $text-muted; }
    TextArea {
        height: 1fr;
    }
    Button {
        margin: 0 1;
    }
    #prompt-status {
        color: $text-muted;
        height: auto;
    }
    """

    BINDINGS = [
        Binding("c", "commit", "Commit All"),
        Binding("g", "generate", "Generate Prompt"),
        Binding("r", "refresh", "Refresh"),
        Binding("q", "quit", "Quit"),
    ]

    def compose(self) -> ComposeResult:
        yield Header()
        with Horizontal():
            with Vertical(id="files-panel"):
                yield Static("Changed Files", id="files-title")
                yield DataTable(id="files-table")
            with Vertical():
                with Vertical(id="summary-panel"):
                    yield Static("Summary Bridge", id="summary-title")
                    yield Static("Click 'Generate Prompt' or press 'g' to create a summary request file for DeepCode.", id="prompt-status")
                    yield Button("Generate Prompt", id="btn-generate", variant="primary")
                    yield Static("", id="summary-files-title")
                    yield DataTable(id="summary-table")
                with Vertical(id="status-panel"):
                    yield Static("Lattice Status", id="status-title")
                    yield Static("", id="noether-status")
                    yield Static("", id="commit-history")
        yield Footer()

    def on_mount(self):
        self.refresh_all()

    def action_refresh(self):
        self.refresh_all()

    def action_generate(self):
        path = generate_prompt_file()
        self.query_one("#prompt-status", Static).update(
            f"[green]Prompt written to {path}[/]\n"
            f"Switch to DeepCode and say: read {path}\n"
            f"Ask it to generate the summary and save to _SUMMARIES/<date>-<slug>.md"
        )
        self.refresh_all()

    def action_commit(self):
        files = get_changed_files()
        unstaged = [f for f in files if not f["staged"]]
        if unstaged:
            subprocess.run(
                ["git", "add", "-A"],
                capture_output=True, cwd=str(PROJECT_ROOT),
            )

        # Find latest summary file to use as commit message body
        summaries = read_summary_files()
        if summaries:
            summary_path = SUMMARIES_DIR / summaries[0]["name"]
            msg = summary_path.read_text().strip()
        else:
            msg = "Development session commit"

        result = subprocess.run(
            ["git", "commit", "-m", msg],
            capture_output=True, text=True, cwd=str(PROJECT_ROOT),
        )

        if result.returncode == 0:
            self.query_one("#prompt-status", Static).update(
                "[green]Committed successfully.[/]\n"
                "Push from VS Code (git push) — auth handled there."
            )
        else:
            self.query_one("#prompt-status", Static).update(
                f"[red]Commit failed: {result.stderr.strip()[:200]}[/]"
            )
        self.refresh_all()

    def refresh_all(self):
        self.refresh_files()
        self.refresh_summaries()
        self.refresh_status()

    def refresh_files(self):
        table = self.query_one("#files-table", DataTable)
        table.clear()
        table.add_columns("St", "File", "Size")
        files = get_changed_files()
        staged = sum(1 for f in files if f.get("staged"))
        unstaged_count = len(files) - staged
        for f in files:
            label = f["status"]
            if not f.get("staged") and f["status"] != "U":
                label = f"{f['status']}*"
            status = Text(label, style=f"status-{f['status']}")
            size_str = f"{f['size']:,}B" if f["size"] < 1024 else f"{f['size']/1024:.1f}KB"
            table.add_row(status, f["path"], size_str)
        self.query_one("#files-title", Static).update(
            f"Changed Files ({len(files)}: {staged} staged, {unstaged_count} unstaged)"
        )

    def refresh_summaries(self):
        table = self.query_one("#summary-table", DataTable)
        table.clear()
        table.add_columns("Summary File", "Preview")
        summaries = read_summary_files()
        for s in summaries:
            table.add_row(s["name"], s["preview"][:100])
        self.query_one("#summary-files-title", Static).update(
            f"Existing Summaries ({len(summaries)})" if summaries else "No summaries yet"
        )

    def refresh_status(self):
        files = get_changed_files()
        n_count = sum(1 for f in files if f["status"] == "A")
        d_count = sum(1 for f in files if f["status"] == "D")
        divergence = abs(n_count - d_count)

        commits = get_recent_commits(3)
        commit_text = "Recent commits:\n" + "\n".join(f"  {c}" for c in commits) if commits else "No commits yet"

        status = "STABLE" if divergence <= 3 else "CHURNING"
        self.query_one("#noether-status", Static).update(
            f"Noether: |N|-|D| = {divergence} → {status}"
        )
        self.query_one("#commit-history", Static).update(commit_text)


if __name__ == "__main__":
    app = HllsetLatticeApp()
    app.run()
