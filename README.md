# cclens

A CLI tool for searching, analyzing, and exporting Claude Code session history.

cclens indexes Claude Code's JSONL conversation files into a local SQLite database and provides fast full-text search across sessions with filtering by branch, date range, and project scope.

## Installation

```bash
cargo install --path .
```

## Usage

### Search sessions

```bash
cclens query "search terms"
cclens query --branch feature/xxx
cclens query "keywords" --after 2025-01-01 --before 2025-01-31
cclens query "term-a|term-b"   # OR search: match any term
cclens query "search terms" --json   # output JSON instead of the default table
```

### List all sessions

```bash
cclens list                          # most recent 30 sessions
cclens list --branch feature/xxx     # filter by branch
cclens list --after 2025-01-01 --before 2025-01-31
cclens list --limit 100              # raise the row cap (default 30)
cclens list --json                   # output JSON instead of the default table
```

`list` shares `query`'s filters but takes no search keyword and shows the most recent sessions by default.

### Inspect a session

```bash
cclens show <session-id>     # metadata as JSON
cclens export <session-id>   # full conversation as Markdown
```

### Install Claude Code skills

```bash
cclens install
```

Installs three skills to `~/.claude/skills/`:

- `cclens-searching-history` -- search past sessions by keyword, branch, or date
- `cclens-exporting-history` -- export session conversations as Markdown
- `cclens-resuming-from-history` -- load past session context into the current session

## How it works

Claude Code stores conversation history as JSONL files under `~/.claude/projects/`. cclens parses these files, extracts metadata (branch, timestamps, message counts), and indexes the content into a SQLite database at `~/.cache/cclens/index.db`. The index is built on first query and updated incrementally.
