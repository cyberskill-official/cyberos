# APP module - task index

The `app` module is CyberOS's own first-party application surfaces: the front-ends that CyberSkill operators and admins use to see and drive CyberOS. It is distinct from the tenant-facing `portal` module - `portal` is the white-labelled surface a client tenant uses, `app` is the CyberSkill-branded surface the operator uses. Both consume already-shipped service APIs; neither adds a backend.

## FRs

| FR | Priority | Title |
|---|---|---|
| [TASK-APP-001](TASK-APP-001-cds-web-console/spec.md) | p3 | APP CDS web console - operator console over CyberOS service APIs |
| [TASK-APP-002](TASK-APP-002-desktop-workflow-trigger/spec.md) | p2 | APP desktop workflow trigger - Tauri app to run CyberOS workflows |

## Cross-module dependencies

Both FRs are front-ends over surfaces that already ship:

- **AI**: TASK-APP-001 and TASK-APP-002 -> TASK-AI-022 (gateway HTTP serving surface), TASK-AI-105 (model providers behind it)
- **OBS**: TASK-APP-001 -> TASK-OBS-008 (compliance-view scoping), TASK-OBS-002 (tenant-aware Grafana proxy)
- **MCP**: TASK-APP-002 -> TASK-MCP-001 (spec compliance), TASK-MCP-006 (tool-annotation gating)
- **CUO**: TASK-APP-002 -> TASK-CUO-101 (LangGraph supervisor orchestrates the triggered workflows)
- **AUTH**: TASK-APP-001 -> TASK-AUTH-004, TASK-AUTH-104; TASK-APP-002 -> TASK-AUTH-004, TASK-AUTH-005

`portal` (the tenant-facing surface) is referenced by both FRs for contrast and house style, not consumed.
