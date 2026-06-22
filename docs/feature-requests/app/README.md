# APP module - feature request index

The `app` module is CyberOS's own first-party application surfaces: the front-ends that CyberSkill operators and admins use to see and drive CyberOS. It is distinct from the tenant-facing `portal` module - `portal` is the white-labelled surface a client tenant uses, `app` is the CyberSkill-branded surface the operator uses. Both consume already-shipped service APIs; neither adds a backend.

## FRs

| FR | Priority | Title |
|---|---|---|
| [FR-APP-001](FR-APP-001-cds-web-console.md) | p3 | APP CDS web console - operator console over CyberOS service APIs |
| [FR-APP-002](FR-APP-002-desktop-workflow-trigger.md) | p2 | APP desktop workflow trigger - Tauri app to run CyberOS workflows |

## Cross-module dependencies

Both FRs are front-ends over surfaces that already ship:

- **AI**: FR-APP-001 and FR-APP-002 -> FR-AI-022 (gateway HTTP serving surface), FR-AI-105 (model providers behind it)
- **OBS**: FR-APP-001 -> FR-OBS-008 (compliance-view scoping), FR-OBS-002 (tenant-aware Grafana proxy)
- **MCP**: FR-APP-002 -> FR-MCP-001 (spec compliance), FR-MCP-006 (tool-annotation gating)
- **CUO**: FR-APP-002 -> FR-CUO-101 (LangGraph supervisor orchestrates the triggered workflows)
- **AUTH**: FR-APP-001 -> FR-AUTH-004, FR-AUTH-104; FR-APP-002 -> FR-AUTH-004, FR-AUTH-005

`portal` (the tenant-facing surface) is referenced by both FRs for contrast and house style, not consumed.
