Intake a document into the agent library.

The user provided: $ARGUMENTS

This may be a file path, a URL, or a block of pasted text. Determine which it is:
- If it looks like a file path, read it with the Read tool.
- If it looks like a URL, fetch it with WebFetch.
- Otherwise treat the argument itself as the raw content.

Once you have the content, analyze it and extract or infer:

1. **`name`** — A concise, title-case display name for this document (e.g. "REST API Design", "Error Handling in Go"). Should be specific enough to distinguish it from other documents but short enough to scan in a list.

2. **`task_type`** — The best-fit category from the existing library. First call `browse_documents` to see what task types already exist and prefer matching an existing one over coining a new one. If the content genuinely doesn't fit any existing type, propose a new one. Common types include: Architecture, Style Guide, Documentation, Testing, Security, Performance.

3. **`content`** — The document body to save. If the source document is already well-structured Markdown, preserve it as-is. If it is prose, code, or another format, reformat it into clean Markdown with:
   - A `# Title` heading matching the name
   - Logical sections with `##` headings
   - Code blocks where appropriate
   - No redundant preamble ("This document describes…")

Once you have all three fields, call `save_document` with `name`, `task_type`, and `content`.

After saving, report:
- The name and task_type you chose
- A one-sentence rationale for the task_type (especially if you created a new one)
- The file path where it was saved (returned by the tool)
