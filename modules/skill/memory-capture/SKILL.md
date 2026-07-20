---
name: memory-capture
description: >-
  Capture durable CyberOS memory rows from workflow outputs or operator notes.
  Use when user asks to "remember this", "capture this decision", or "write
  this to BRAIN". Outputs memory writer envelopes with least-privilege scope
  checks and skill invocation audit rows.
metadata:
  version: 1.0.0
  module: skill
allowed_memory_scopes:
  read:
    - project:*
  write:
    - memories:facts
    - memories:decisions
allowed_mcp_tools:
  - memory.write_memory
  - audit.append
---

# memory-capture

Normalize an input body, classify the target memory kind, and emit a writer envelope for the canonical memory writer. Never bypass the writer or mutate `.cyberos/memory/store` directly.
