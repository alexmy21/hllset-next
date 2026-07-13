#!/usr/bin/env python3
"""Textual TUI for hllset commit workflow.

Shows staged files, their HLLSet fingerprints, Noether steering check,
and commit/abort actions — all in one terminal dashboard.

Usage:
    python scripts/hllset_commit_tui.py
"""

import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

from textual.app import App, ComposeResult
from textual.containers import Horizontal, Vertical, Grid
from textual.widgets import (
    Header, Footer, Static, DataTable, Button, Log, Label, Input,
)
from textual.binding import Binding
from textual.screen import ModalScreen
from textual.reactive import reactive
from rich.text import Text
from rich.panel import Panel

# ── Configuration ──────────────────────────────────────────────────────

SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = SCRIPT_DIR.parent
HLLSET_BIN = os.environ.get("HLLSET_BINARY", str(PROJECT_ROOT / "target" / "debug" / "hllset"))
META_FILE = PROJECT_ROOT / ".hllset_lattice" / "metadata.json"


# ── HLLSet helpers ─────────────────────────────────────────────────────

def run_hllset(script: str, timeout: float = 15.0) -> str:
    """Run a Lua script via hllset CLI, return stdout string."""
    proc = subprocess.run(
        [HLLSET_BIN, "-e", script],
        capture_output=True, text=True, timeout=timeout,
        cwd=str(PROJECT_ROOT),
    )
    if proc.returncode != 0:
        return f"ERROR: {proc.stderr.strip()[:100]}"
    return proc.stdout.strip()


def get_changed_files() -> list[dict]:
    """Get list of changed files: staged + unstaged, deduplicated by path.

    Staged statuses take priority over unstaged for the same file.
    """
    staged = {}
    unstaged = {}

    # Staged: git diff --cached
    r = subprocess.run(
        ["git", "diff", "--cached", "--name-status"],
        capture_output=True, text=True, cwd=str(PROJECT_ROOT),
    )
    for line in r.stdout.strip().split("\n"):
        if not line: continue
        parts = line.split("\t", 1)
        if len(parts) == 2:
            staged[parts[1]] = parts[0]

    # Unstaged: git diff --name-status (working tree vs index)
    r = subprocess.run(
        ["git", "diff", "--name-status"],
        capture_output=True, text=True, cwd=str(PROJECT_ROOT),
    )
    for line in r.stdout.strip().split("\n"):
        if not line: continue
        parts = line.split("\t", 1)
        if len(parts) == 2:
            unstaged[parts[1]] = parts[0]

    # Untracked: git ls-files --others --exclude-standard
    r = subprocess.run(
        ["git", "ls-files", "--others", "--exclude-standard"],
        capture_output=True, text=True, cwd=str(PROJECT_ROOT),
    )
    for fpath in r.stdout.strip().split("\n"):
        if fpath:
            unstaged[fpath] = "U"  # untracked

    # Merge: staged overrides unstaged
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


# ── Textual App ────────────────────────────────────────────────────────

class HllsetCommitApp(App):
    """Terminal dashboard for hllset-aware git commit."""

    CSS = """
    Screen {
        layout: grid;
        grid-size: 2;
        grid-rows: 1fr 1fr;
        grid-columns: 1fr 1fr;
    }
    #files-panel {
        row-span: 2;
        border: solid $primary;
    }
    #preview-panel {
        border: solid $secondary;
    }
    #noether-panel {
        border: solid $accent;
    }
    #log-panel {
        border: solid $surface;
    }
    DataTable {
        height: 1fr;
    }
    .status-A { color: $success; }
    .status-M { color: $warning; }
    .status-D { color: $error; }
    .status-U { color: $text-muted; }
    """

    BINDINGS = [
        Binding("c", "commit", "Commit & Ingest"),
        Binding("a", "abort", "Abort"),
        Binding("r", "refresh", "Refresh"),
        Binding("q", "quit", "Quit"),
    ]

    staged_count = reactive(0)
    preview_key = reactive("")

    def compose(self) -> ComposeResult:
        yield Header()
        with Horizontal():
            with Vertical(id="files-panel"):
                yield Static("Staged Files", id="files-title")
                yield DataTable(id="files-table")
            with Vertical():
                with Vertical(id="preview-panel"):
                    yield Static("HLLSet Preview", id="preview-title")
                    yield Static("Select a file to preview", id="preview-info")
                with Vertical(id="noether-panel"):
                    yield Static("Noether Check", id="noether-title")
                    yield Static("|N|-|D|: —", id="noether-divergence")
                    yield Static("Rank flux: —", id="noether-flux")
                    yield Static("Δ²R: —", id="noether-accel")
        yield Footer()

    def on_mount(self):
        self.refresh_files()
        self.refresh_noether()

    def action_refresh(self):
        self.refresh_files()
        self.refresh_noether()

    def action_commit(self):
        self.push_screen(CommitConfirmScreen())

    def action_abort(self):
        self.exit(message="Aborted.")

    def refresh_files(self):
        table = self.query_one("#files-table", DataTable)
        table.clear()
        table.add_columns("St", "File", "Size", "Idx")
        files = get_changed_files()
        staged = sum(1 for f in files if f.get("staged"))
        unstaged = len(files) - staged
        self.staged_count = len(files)
        for f in files:
            status_label = f["status"]
            if not f.get("staged") and f["status"] != "U":
                status_label = f"{f['status']}*"  # asterisk = unstaged
            status = Text(status_label, style=f"status-{f['status']}")
            size_str = f"{f['size']:,}B" if f["size"] < 1024 else f"{f['size']/1024:.1f}KB"
            idx_str = "+" if f.get("staged") else "~"
            table.add_row(status, f["path"], size_str, idx_str)

        self.query_one("#files-title", Static).update(
            f"Changed Files ({self.staged_count} total: {staged} staged, {unstaged} unstaged)"
        )

    def refresh_noether(self):
        """Quick Noether check on all changed files."""
        files = get_changed_files()
        n_count = sum(1 for f in files if f["status"] == "A")
        d_count = sum(1 for f in files if f["status"] == "D")
        m_count = sum(1 for f in files if f["status"] == "M")

        divergence = abs(n_count - d_count)
        total_churn = n_count + d_count + m_count

        self.query_one("#noether-divergence", Static).update(
            f"|N|-|D|: {divergence}  (N={n_count}, D={d_count}, M={m_count})"
        )
        status = "STABLE" if divergence <= 3 else "DIVERGING"
        self.query_one("#noether-flux", Static).update(
            f"Churn: {total_churn} files — {status}"
        )
        self.query_one("#noether-accel", Static).update(
            f"Threshold: {divergence}/{'3' if divergence <= 3 else '~'} "
            f"{'✓' if divergence <= 3 else '⚠'}"
        )


class CommitConfirmScreen(ModalScreen[bool]):
    """Confirmation dialog before commit + ingest."""

    CSS = """
    CommitConfirmScreen {
        align: center middle;
    }
    #confirm-dialog {
        width: 60;
        height: auto;
        border: thick $primary;
        background: $surface;
        padding: 1 2;
    }
    """

    def compose(self) -> ComposeResult:
        yield Vertical(
            Static("Commit & Ingest to HLLSet Lattice", id="confirm-title"),
            Static(""),
            Static("This will:"),
            Static("  1. Run `git commit` with your staged changes"),
            Static("  2. Ingest changed files into the HLLSet lattice"),
            Static("  3. Record D/R/N decomposition for this commit"),
            Static(""),
            Input(placeholder="Commit message", id="commit-msg"),
            Static(""),
            Horizontal(
                Button("Commit & Ingest", variant="primary", id="btn-commit"),
                Button("Cancel", variant="default", id="btn-cancel"),
            ),
            id="confirm-dialog",
        )

    def on_button_pressed(self, event: Button.Pressed):
        if event.button.id == "btn-cancel":
            self.dismiss(False)
        elif event.button.id == "btn-commit":
            msg = self.query_one("#commit-msg", Input).value.strip()
            if not msg:
                self.query_one("#confirm-title", Static).update(
                    "[red]Please enter a commit message[/]"
                )
                return
            # Do the actual commit + ingest
            result = subprocess.run(
                ["git", "commit", "-m", msg],
                capture_output=True, text=True, cwd=str(PROJECT_ROOT),
            )
            if result.returncode == 0:
                # Trigger ingest
                commit_hash = subprocess.run(
                    ["git", "rev-parse", "HEAD"],
                    capture_output=True, text=True, cwd=str(PROJECT_ROOT),
                ).stdout.strip()
                subprocess.run(
                    [sys.executable, str(SCRIPT_DIR / "ingest.py"), "commit", commit_hash],
                    cwd=str(PROJECT_ROOT),
                )
                self.dismiss(True)
            else:
                self.query_one("#confirm-title", Static).update(
                    f"[red]Commit failed: {result.stderr.strip()[:100]}[/]"
                )


def main():
    app = HllsetCommitApp()
    app.run()


if __name__ == "__main__":
    main()
