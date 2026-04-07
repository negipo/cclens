---
name: exporting-history
description: Export past Claude Code session conversations as Markdown files using the cclens CLI. Use when the user says things like "export the conversation", "save the session as Markdown", "save past conversations", or "output the conversation log". If the session ID is unknown, use the cclens:searching-history skill first.
---

# cclens: Conversation Export

Export past session conversations as Markdown files using the cclens CLI.

## Prerequisites

The session ID must already be identified. If unknown, identify the session first using the cclens:searching-history skill.

## Workflow

1. Run `cclens export <session-id>` to output Markdown to stdout
2. Redirect to a file to save

## Command

```bash
cclens export <session-id> > /tmp/session_export.md
```

The export output is Markdown with user inputs and assistant responses arranged chronologically. Claude Code does not need to read this content — simply save it directly to a file.
