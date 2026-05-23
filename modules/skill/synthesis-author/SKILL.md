---
name: synthesis-author
description: >-
  Author a derived synthesis memory from clustered facts, decisions, or project
  episodes. Use when user asks to "synthesize these memories", "find the pattern
  across these decisions", or "write a reflection summary". Outputs a proposed
  memory body and provenance list for human or dream-applier review.
metadata:
  version: 1.0.0
  module: skill
allowed_memory_scopes:
  read:
    - memories:*
  write:
    - memories:refinements
allowed_mcp_tools:
  - memory.search
  - memory.write_memory
---

# synthesis-author

Create a concise synthesis with cited source memories and no unstated facts.
