---
name: resuming-from-history
description: Load past Claude Code session conversation content into the current session's context using the cclens CLI. Use when the user says things like "based on what we discussed before", "summarize last week's work", "I want to continue the previous conversation", or "recall the past session content". If the session ID is unknown, use the cclens:searching-history skill first.
---

# cclens: Context Reuse

Retrieve past session conversation content using the cclens CLI and use it as context for the current session.

## Prerequisites

The session ID must already be identified. If unknown, identify the session first using the cclens:searching-history skill.

## Workflow

Conversation logs can be large, so it is preferable to have a subagent read them to avoid overwhelming the main context.

1. Dispatch a subagent to read the output of `cclens export <session-id>`
2. Instruct the subagent to process according to the user's request:
   - If a summary is requested → return a summary of the key points
   - If asked about a specific topic → quote and explain the relevant parts
   - If continuation of work is requested → organize the previous work and remaining tasks
3. Present the subagent's response to the user
4. If continuation work is anticipated (the subagent's summary includes incomplete tasks, the user says "continue", "pick up where we left off", etc.), start loading the full text in the background (`cclens export` into the main context) simultaneously with presenting the summary. Do not wait for the user to ask for it.

## Branch Caveat

The branch name recorded in a session is simply where the user happened to be at the time, not a work instruction. When continuing work, do not reuse the session's branch name directly — confirm with the user whether to pull the latest main and start from there.

Subagent dispatch example:
```
Run the following command to retrieve the conversation log from a past session and process it according to the user's request.

Run command: cclens export {session_id}

User's request: {what the user wants to know/do}

Use only the `cclens export` output as your information source and respond according to the request.
Do not read JSONL files under .claude directly. JSONL is raw data and very large, which would overwhelm the context. `cclens export` returns formatted and compressed output, which is sufficient.
```

## Cross-Session Queries

For requests spanning multiple sessions, such as "summarize last week's work":

1. Retrieve the session list using cclens:searching-history
2. Dispatch subagents in parallel for each session to get summaries
3. Consolidate the summaries and present to the user
