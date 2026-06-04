---
name: searching-history
description: Search past Claude Code sessions using the cclens CLI and present resume commands. Use when the user says things like "that thing we did before", "find that session", "the conversation about...", "the PR session", "work history on this branch", when searching past conversations by keyword/branch/URL, when identifying a session for --resume, or when other cclens skills (exporting-history, resuming-from-history) need a session ID. If there is any chance of searching session history, use this skill first.
---

# cclens: Session Search

Search past Claude Code sessions using the cclens CLI and present results to the user.
Other cclens skills (exporting-history, resuming-from-history) require the session ID to be already identified, so session identification should go through this skill.

## Commands

### cclens query

Text search:
```bash
cclens query "search keywords"
```

`query` searches across all projects by default.

OR search (match any of several terms, separated by `|`):
```bash
cclens query "Notion|Slack|Linear"
```

Filter by branch:
```bash
cclens query --branch feature/xxx
```

Filter by date:
```bash
cclens query "keywords" --after 2026-03-01 --before 2026-03-31
```

### cclens show

Output session metadata (start time, branch, message count, etc.) as JSON:
```bash
cclens show <session-id>
```

### cclens export

Output all session messages as Markdown (for content review):
```bash
cclens export <session-id>
```

## Building Queries

Choose the appropriate search method based on the user's request:

- "Review session for PR 42" → `cclens query "42"` or `cclens query "PR 42"`
- If the PR URL is known → get the branch name with `gh pr view <url> --json headRefName` then `cclens query --branch <branch>`
- "Last week's work on example repository" → `cclens query --after 2026-03-29 --before 2026-04-05`
- "Session where we discussed retry logic" → `cclens query "retry logic"`

## Output and Session Identification Flow

The query output is a JSON array. Each session includes snippets of matched messages:
```json
[
  {
    "session_id": "abcd1234-...",
    "project_path": "-Users-example-src-sample-repo",
    "cwd": "/Users/example/src/sample-repo",
    "git_branch": "main",
    "started_at": "2026-03-22T12:56:20Z",
    "match_count": 3,
    "matches": [
      {"role": "user", "snippet": "Add retry logic to the API client", "timestamp": "..."},
      {"role": "assistant", "snippet": "I'll add exponential backoff with a configurable max retries.", "timestamp": "..."}
    ],
    "resume_command": "claude --resume abcd1234-..."
  }
]
```

### Single candidate

Present the resume command directly to the user.

### Multiple candidates

When presenting candidates, also show the leaf directory name (basename of `cwd`, falling back to `project_path`) so the user can tell which project each session belongs to.

Snippets alone are often insufficient for the user to decide. Use the following flow to narrow down:

1. First check if the user can decide based on snippet content
2. If difficult to decide, dispatch subagents in parallel for each candidate session, having them read `cclens export <id>` output and return a ~3-line summary
3. Present the list of summaries to the user and let them choose

Subagent dispatch example:
```
Review the contents of session {session_id} and summarize in 3 lines or fewer.

Run command: cclens export {session_id}
Read the output and report in this format:
- Main topic
- What the user requested
- What the outcome was

Use only the `cclens export` output as your information source.
Do not read JSONL files under .claude directly. JSONL is raw data and very large, which would overwhelm the context.
```

If there are many candidates (5+), it is more context-efficient to first narrow down with --branch or --after/--before before dispatching subagents.

### Zero results

1. Suggest retrying with different keywords
2. Note that `query` already spans all projects; try broadening keywords or using OR (`a|b`) instead
