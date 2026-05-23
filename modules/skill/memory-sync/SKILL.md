---
name: memory-sync
description: >-
  Trigger or preview CyberOS memory synchronization between local and cloud
  stores. Use when user asks to "sync memory", "push shareable memories", or
  "preview memory sync". Outputs a dry-run or apply request for the Stage 4
  sync orchestrator.
metadata:
  version: 1.0.0
  module: skill
allowed_memory_scopes:
  read:
    - memories:*
  write:
    - memories:*
allowed_mcp_tools:
  - memory.sync
  - audit.append
---

# memory-sync

Build a sync request with direction, dry-run flag, and sync_class filter.
