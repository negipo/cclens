---
name: resuming-from-history
description: Load a past Claude Code session's full conversation into the current session's context using the cclens CLI, to continue where it left off. Use when the user says things like "based on what we discussed before", "I want to continue the previous conversation", "pick up where we left off", or "recall the past session content". If the session ID is unknown, use the cclens:searching-history skill first.
---

# cclens: Context Reuse

Load a past session's full conversation into the current session's context using the cclens CLI, and use it to continue the work.

## Prerequisites

The session ID must already be identified. If unknown, identify the session first using the cclens:searching-history skill.

## Workflow

`cclens export` returns a compressed, formatted view of the prior session that will not exceed the previous session's context size, so it fits safely in the main context. Load the full text directly — do not summarize and do not dispatch a subagent.

1. Run `cclens export <session-id>` via Bash in the main session.
2. Load every document the prior session relied on. Then respond to the user's request to continue the previous work.

Do not read JSONL files under .claude directly. JSONL is raw data and very large, which would overwhelm the context. `cclens export` returns formatted and compressed output, which is sufficient.

## Loading Referenced Documents

`cclens export` renders conversation text only — tool results are dropped. A document the user presented survives in the output as its path or URL, never as its content.

After the export lands in context, list every document the user pointed at or the work depended on — plan and design docs (`tmp/doc/`, `docs/`), specs, schemas, config files, PR and issue URLs, other shared links — and load each one in full before continuing:

- Local paths: Read (or `cat`)
- PR / issue URLs: `gh pr view` / `gh issue view`, including the diff when the work touched it
- Other URLs: WebFetch

Load the whole document, not the portion the conversation quoted. If a document no longer exists or cannot be fetched, tell the user rather than reconstructing it from the conversation.

## Branch Caveat

The branch name recorded in a session is simply where the user happened to be at the time, not a work instruction. When continuing work, do not reuse the session's branch name directly — confirm with the user whether to pull the latest main and start from there.
