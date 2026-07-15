# Rename epoch — feature-request to task

Date: 2026-07-14. Commit at time of writing: `e37dc420`.

Every `FR-<MODULE>-<NNN>` id in this repository was mechanically rewritten to
`TASK-<MODULE>-<NNN>`. The mapping is 1:1 and prefix-only — no id was renumbered,
merged, or split.

This file exists because the rename rewrote **archived evidence** as well as live
specs (decision D3, taken pre-1.0). Without it, a 2026-05 code-review artefact
appears to have reviewed a `TASK-` that did not exist under that name at the time.
Read every `TASK-*` id in an artefact dated before 2026-07-14 as its `FR-*` original.

## Rule

```
FR-<MODULE>-<NNN>   ->   TASK-<MODULE>-<NNN>
FR-<NNN>            ->   TASK-<NNN>
FR-<NNN>-T-<MM>     ->   TASK-<NNN>-S-<MM>     (subtask; -T- would read as 'task')
```

## The 507 ids

| was | is | module |
|---|---|---|
| `FR-AI-001` | `TASK-AI-001` | ai |
| `FR-AI-002` | `TASK-AI-002` | ai |
| `FR-AI-003` | `TASK-AI-003` | ai |
| `FR-AI-004` | `TASK-AI-004` | ai |
| `FR-AI-005` | `TASK-AI-005` | ai |
| `FR-AI-006` | `TASK-AI-006` | ai |
| `FR-AI-007` | `TASK-AI-007` | ai |
| `FR-AI-008` | `TASK-AI-008` | ai |
| `FR-AI-009` | `TASK-AI-009` | ai |
| `FR-AI-010` | `TASK-AI-010` | ai |
| `FR-AI-011` | `TASK-AI-011` | ai |
| `FR-AI-012` | `TASK-AI-012` | ai |
| `FR-AI-013` | `TASK-AI-013` | ai |
| `FR-AI-014` | `TASK-AI-014` | ai |
| `FR-AI-015` | `TASK-AI-015` | ai |
| `FR-AI-016` | `TASK-AI-016` | ai |
| `FR-AI-017` | `TASK-AI-017` | ai |
| `FR-AI-018` | `TASK-AI-018` | ai |
| `FR-AI-019` | `TASK-AI-019` | ai |
| `FR-AI-020` | `TASK-AI-020` | ai |
| `FR-AI-021` | `TASK-AI-021` | ai |
| `FR-AI-022` | `TASK-AI-022` | ai |
| `FR-AI-104` | `TASK-AI-104` | ai |
| `FR-AI-105` | `TASK-AI-105` | ai |
| `FR-APP-001` | `TASK-APP-001` | app |
| `FR-APP-003` | `TASK-APP-003` | app |
| `FR-APP-004` | `TASK-APP-004` | app |
| `FR-APP-005` | `TASK-APP-005` | app |
| `FR-APP-006` | `TASK-APP-006` | app |
| `FR-AUTH-001` | `TASK-AUTH-001` | auth |
| `FR-AUTH-002` | `TASK-AUTH-002` | auth |
| `FR-AUTH-003` | `TASK-AUTH-003` | auth |
| `FR-AUTH-004` | `TASK-AUTH-004` | auth |
| `FR-AUTH-005` | `TASK-AUTH-005` | auth |
| `FR-AUTH-006` | `TASK-AUTH-006` | auth |
| `FR-AUTH-101` | `TASK-AUTH-101` | auth |
| `FR-AUTH-102` | `TASK-AUTH-102` | auth |
| `FR-AUTH-103` | `TASK-AUTH-103` | auth |
| `FR-AUTH-104` | `TASK-AUTH-104` | auth |
| `FR-AUTH-105` | `TASK-AUTH-105` | auth |
| `FR-AUTH-106` | `TASK-AUTH-106` | auth |
| `FR-AUTH-107` | `TASK-AUTH-107` | auth |
| `FR-AUTH-108` | `TASK-AUTH-108` | auth |
| `FR-AUTH-109` | `TASK-AUTH-109` | auth |
| `FR-AUTH-110` | `TASK-AUTH-110` | auth |
| `FR-AUTH-111` | `TASK-AUTH-111` | auth |
| `FR-CHAT-101` | `TASK-CHAT-101` | chat |
| `FR-CHAT-102` | `TASK-CHAT-102` | chat |
| `FR-CHAT-103` | `TASK-CHAT-103` | chat |
| `FR-CHAT-104` | `TASK-CHAT-104` | chat |
| `FR-CHAT-105` | `TASK-CHAT-105` | chat |
| `FR-CHAT-106` | `TASK-CHAT-106` | chat |
| `FR-CHAT-201` | `TASK-CHAT-201` | chat |
| `FR-CHAT-202` | `TASK-CHAT-202` | chat |
| `FR-CHAT-203` | `TASK-CHAT-203` | chat |
| `FR-CHAT-204` | `TASK-CHAT-204` | chat |
| `FR-CHAT-205` | `TASK-CHAT-205` | chat |
| `FR-CHAT-206` | `TASK-CHAT-206` | chat |
| `FR-CHAT-207` | `TASK-CHAT-207` | chat |
| `FR-CHAT-208` | `TASK-CHAT-208` | chat |
| `FR-CHAT-209` | `TASK-CHAT-209` | chat |
| `FR-CHAT-210` | `TASK-CHAT-210` | chat |
| `FR-CHAT-211` | `TASK-CHAT-211` | chat |
| `FR-CHAT-212` | `TASK-CHAT-212` | chat |
| `FR-CHAT-213` | `TASK-CHAT-213` | chat |
| `FR-CHAT-214` | `TASK-CHAT-214` | chat |
| `FR-CHAT-215` | `TASK-CHAT-215` | chat |
| `FR-CHAT-216` | `TASK-CHAT-216` | chat |
| `FR-CHAT-217` | `TASK-CHAT-217` | chat |
| `FR-CHAT-218` | `TASK-CHAT-218` | chat |
| `FR-CHAT-219` | `TASK-CHAT-219` | chat |
| `FR-CHAT-220` | `TASK-CHAT-220` | chat |
| `FR-CHAT-221` | `TASK-CHAT-221` | chat |
| `FR-CHAT-222` | `TASK-CHAT-222` | chat |
| `FR-CHAT-223` | `TASK-CHAT-223` | chat |
| `FR-CHAT-224` | `TASK-CHAT-224` | chat |
| `FR-CHAT-225` | `TASK-CHAT-225` | chat |
| `FR-CHAT-226` | `TASK-CHAT-226` | chat |
| `FR-CHAT-227` | `TASK-CHAT-227` | chat |
| `FR-CHAT-228` | `TASK-CHAT-228` | chat |
| `FR-CHAT-229` | `TASK-CHAT-229` | chat |
| `FR-CHAT-230` | `TASK-CHAT-230` | chat |
| `FR-CHAT-231` | `TASK-CHAT-231` | chat |
| `FR-CHAT-232` | `TASK-CHAT-232` | chat |
| `FR-CHAT-233` | `TASK-CHAT-233` | chat |
| `FR-CHAT-234` | `TASK-CHAT-234` | chat |
| `FR-CHAT-235` | `TASK-CHAT-235` | chat |
| `FR-CHAT-236` | `TASK-CHAT-236` | chat |
| `FR-CHAT-237` | `TASK-CHAT-237` | chat |
| `FR-CHAT-238` | `TASK-CHAT-238` | chat |
| `FR-CHAT-239` | `TASK-CHAT-239` | chat |
| `FR-CHAT-240` | `TASK-CHAT-240` | chat |
| `FR-CHAT-241` | `TASK-CHAT-241` | chat |
| `FR-CHAT-242` | `TASK-CHAT-242` | chat |
| `FR-CHAT-243` | `TASK-CHAT-243` | chat |
| `FR-CHAT-244` | `TASK-CHAT-244` | chat |
| `FR-CHAT-245` | `TASK-CHAT-245` | chat |
| `FR-CHAT-246` | `TASK-CHAT-246` | chat |
| `FR-CHAT-247` | `TASK-CHAT-247` | chat |
| `FR-CHAT-248` | `TASK-CHAT-248` | chat |
| `FR-CHAT-249` | `TASK-CHAT-249` | chat |
| `FR-CHAT-250` | `TASK-CHAT-250` | chat |
| `FR-CHAT-251` | `TASK-CHAT-251` | chat |
| `FR-CHAT-252` | `TASK-CHAT-252` | chat |
| `FR-CHAT-253` | `TASK-CHAT-253` | chat |
| `FR-CHAT-254` | `TASK-CHAT-254` | chat |
| `FR-CHAT-255` | `TASK-CHAT-255` | chat |
| `FR-CHAT-256` | `TASK-CHAT-256` | chat |
| `FR-CHAT-257` | `TASK-CHAT-257` | chat |
| `FR-CHAT-258` | `TASK-CHAT-258` | chat |
| `FR-CHAT-259` | `TASK-CHAT-259` | chat |
| `FR-CHAT-260` | `TASK-CHAT-260` | chat |
| `FR-CHAT-261` | `TASK-CHAT-261` | chat |
| `FR-CHAT-262` | `TASK-CHAT-262` | chat |
| `FR-CHAT-263` | `TASK-CHAT-263` | chat |
| `FR-CHAT-264` | `TASK-CHAT-264` | chat |
| `FR-CHAT-265` | `TASK-CHAT-265` | chat |
| `FR-CHAT-266` | `TASK-CHAT-266` | chat |
| `FR-CHAT-267` | `TASK-CHAT-267` | chat |
| `FR-CHAT-268` | `TASK-CHAT-268` | chat |
| `FR-CHAT-269` | `TASK-CHAT-269` | chat |
| `FR-CRM-001` | `TASK-CRM-001` | crm |
| `FR-CRM-002` | `TASK-CRM-002` | crm |
| `FR-CRM-003` | `TASK-CRM-003` | crm |
| `FR-CRM-004` | `TASK-CRM-004` | crm |
| `FR-CRM-005` | `TASK-CRM-005` | crm |
| `FR-CRM-006` | `TASK-CRM-006` | crm |
| `FR-CRM-007` | `TASK-CRM-007` | crm |
| `FR-CRM-008` | `TASK-CRM-008` | crm |
| `FR-CRM-009` | `TASK-CRM-009` | crm |
| `FR-CRM-010` | `TASK-CRM-010` | crm |
| `FR-CUO-101` | `TASK-CUO-101` | cuo |
| `FR-CUO-102` | `TASK-CUO-102` | cuo |
| `FR-CUO-103` | `TASK-CUO-103` | cuo |
| `FR-CUO-104` | `TASK-CUO-104` | cuo |
| `FR-CUO-105` | `TASK-CUO-105` | cuo |
| `FR-CUO-106` | `TASK-CUO-106` | cuo |
| `FR-CUO-200` | `TASK-CUO-200` | cuo |
| `FR-CUO-201` | `TASK-CUO-201` | cuo |
| `FR-CUO-202` | `TASK-CUO-202` | cuo |
| `FR-CUO-203` | `TASK-CUO-203` | cuo |
| `FR-CUO-204` | `TASK-CUO-204` | cuo |
| `FR-CUO-205` | `TASK-CUO-205` | cuo |
| `FR-CUO-206` | `TASK-CUO-206` | cuo |
| `FR-CUO-207` | `TASK-CUO-207` | cuo |
| `FR-CUO-208` | `TASK-CUO-208` | cuo |
| `FR-CUO-209` | `TASK-CUO-209` | cuo |
| `FR-DOC-001` | `TASK-DOC-001` | doc |
| `FR-DOC-002` | `TASK-DOC-002` | doc |
| `FR-DOC-003` | `TASK-DOC-003` | doc |
| `FR-DOC-004` | `TASK-DOC-004` | doc |
| `FR-DOC-005` | `TASK-DOC-005` | doc |
| `FR-DOC-006` | `TASK-DOC-006` | doc |
| `FR-DOC-007` | `TASK-DOC-007` | doc |
| `FR-DOC-008` | `TASK-DOC-008` | doc |
| `FR-DOC-009` | `TASK-DOC-009` | doc |
| `FR-DOC-010` | `TASK-DOC-010` | doc |
| `FR-DOC-011` | `TASK-DOC-011` | doc |
| `FR-DOCS-001` | `TASK-DOCS-001` | docs |
| `FR-DOCS-002` | `TASK-DOCS-002` | docs |
| `FR-DOCS-003` | `TASK-DOCS-003` | docs |
| `FR-DOCS-004` | `TASK-DOCS-004` | docs |
| `FR-DOCS-005` | `TASK-DOCS-005` | docs |
| `FR-DOCS-006` | `TASK-DOCS-006` | docs |
| `FR-DOCS-007` | `TASK-DOCS-007` | docs |
| `FR-EMAIL-001` | `TASK-EMAIL-001` | email |
| `FR-EMAIL-002` | `TASK-EMAIL-002` | email |
| `FR-EMAIL-003` | `TASK-EMAIL-003` | email |
| `FR-EMAIL-004` | `TASK-EMAIL-004` | email |
| `FR-EMAIL-005` | `TASK-EMAIL-005` | email |
| `FR-EMAIL-006` | `TASK-EMAIL-006` | email |
| `FR-EMAIL-007` | `TASK-EMAIL-007` | email |
| `FR-EMAIL-008` | `TASK-EMAIL-008` | email |
| `FR-EMAIL-009` | `TASK-EMAIL-009` | email |
| `FR-EMAIL-010` | `TASK-EMAIL-010` | email |
| `FR-EMAIL-011` | `TASK-EMAIL-011` | email |
| `FR-ESOP-001` | `TASK-ESOP-001` | esop |
| `FR-ESOP-002` | `TASK-ESOP-002` | esop |
| `FR-ESOP-003` | `TASK-ESOP-003` | esop |
| `FR-ESOP-004` | `TASK-ESOP-004` | esop |
| `FR-ESOP-005` | `TASK-ESOP-005` | esop |
| `FR-ESOP-006` | `TASK-ESOP-006` | esop |
| `FR-ESOP-007` | `TASK-ESOP-007` | esop |
| `FR-EVAL-001` | `TASK-EVAL-001` | eval |
| `FR-EVAL-002` | `TASK-EVAL-002` | eval |
| `FR-EVAL-003` | `TASK-EVAL-003` | eval |
| `FR-EVAL-004` | `TASK-EVAL-004` | eval |
| `FR-HR-001` | `TASK-HR-001` | hr |
| `FR-HR-002` | `TASK-HR-002` | hr |
| `FR-HR-003` | `TASK-HR-003` | hr |
| `FR-HR-004` | `TASK-HR-004` | hr |
| `FR-HR-005` | `TASK-HR-005` | hr |
| `FR-HR-006` | `TASK-HR-006` | hr |
| `FR-HR-007` | `TASK-HR-007` | hr |
| `FR-HR-008` | `TASK-HR-008` | hr |
| `FR-HR-009` | `TASK-HR-009` | hr |
| `FR-IMP-001` | `TASK-IMP-001` | improvement |
| `FR-IMP-002` | `TASK-IMP-002` | improvement |
| `FR-IMP-003` | `TASK-IMP-003` | improvement |
| `FR-IMP-004` | `TASK-IMP-004` | improvement |
| `FR-IMP-005` | `TASK-IMP-005` | improvement |
| `FR-IMP-006` | `TASK-IMP-006` | improvement |
| `FR-IMP-007` | `TASK-IMP-007` | improvement |
| `FR-IMP-008` | `TASK-IMP-008` | improvement |
| `FR-IMP-009` | `TASK-IMP-009` | improvement |
| `FR-IMP-010` | `TASK-IMP-010` | improvement |
| `FR-IMP-011` | `TASK-IMP-011` | improvement |
| `FR-IMP-012` | `TASK-IMP-012` | improvement |
| `FR-IMP-013` | `TASK-IMP-013` | improvement |
| `FR-IMP-014` | `TASK-IMP-014` | improvement |
| `FR-IMP-015` | `TASK-IMP-015` | improvement |
| `FR-IMP-016` | `TASK-IMP-016` | improvement |
| `FR-IMP-017` | `TASK-IMP-017` | improvement |
| `FR-IMP-018` | `TASK-IMP-018` | improvement |
| `FR-IMP-019` | `TASK-IMP-019` | improvement |
| `FR-IMP-020` | `TASK-IMP-020` | improvement |
| `FR-IMP-021` | `TASK-IMP-021` | improvement |
| `FR-IMP-022` | `TASK-IMP-022` | improvement |
| `FR-IMP-023` | `TASK-IMP-023` | improvement |
| `FR-IMP-024` | `TASK-IMP-024` | improvement |
| `FR-IMP-025` | `TASK-IMP-025` | improvement |
| `FR-IMP-026` | `TASK-IMP-026` | improvement |
| `FR-IMP-027` | `TASK-IMP-027` | improvement |
| `FR-IMP-028` | `TASK-IMP-028` | improvement |
| `FR-IMP-029` | `TASK-IMP-029` | improvement |
| `FR-IMP-030` | `TASK-IMP-030` | improvement |
| `FR-IMP-031` | `TASK-IMP-031` | improvement |
| `FR-IMP-032` | `TASK-IMP-032` | improvement |
| `FR-IMP-033` | `TASK-IMP-033` | improvement |
| `FR-IMP-034` | `TASK-IMP-034` | improvement |
| `FR-IMP-035` | `TASK-IMP-035` | improvement |
| `FR-IMP-036` | `TASK-IMP-036` | improvement |
| `FR-IMP-037` | `TASK-IMP-037` | improvement |
| `FR-IMP-038` | `TASK-IMP-038` | improvement |
| `FR-IMP-039` | `TASK-IMP-039` | improvement |
| `FR-IMP-040` | `TASK-IMP-040` | improvement |
| `FR-IMP-041` | `TASK-IMP-041` | improvement |
| `FR-IMP-042` | `TASK-IMP-042` | improvement |
| `FR-IMP-043` | `TASK-IMP-043` | improvement |
| `FR-IMP-044` | `TASK-IMP-044` | improvement |
| `FR-IMP-045` | `TASK-IMP-045` | improvement |
| `FR-IMP-046` | `TASK-IMP-046` | improvement |
| `FR-IMP-047` | `TASK-IMP-047` | improvement |
| `FR-IMP-048` | `TASK-IMP-048` | improvement |
| `FR-IMP-049` | `TASK-IMP-049` | improvement |
| `FR-IMP-050` | `TASK-IMP-050` | improvement |
| `FR-IMP-051` | `TASK-IMP-051` | improvement |
| `FR-IMP-052` | `TASK-IMP-052` | improvement |
| `FR-IMP-053` | `TASK-IMP-053` | improvement |
| `FR-IMP-054` | `TASK-IMP-054` | improvement |
| `FR-IMP-055` | `TASK-IMP-055` | improvement |
| `FR-IMP-056` | `TASK-IMP-056` | improvement |
| `FR-IMP-057` | `TASK-IMP-057` | improvement |
| `FR-IMP-058` | `TASK-IMP-058` | improvement |
| `FR-IMP-059` | `TASK-IMP-059` | improvement |
| `FR-IMP-060` | `TASK-IMP-060` | improvement |
| `FR-IMP-061` | `TASK-IMP-061` | improvement |
| `FR-IMP-062` | `TASK-IMP-062` | improvement |
| `FR-IMP-063` | `TASK-IMP-063` | improvement |
| `FR-IMP-064` | `TASK-IMP-064` | improvement |
| `FR-IMP-065` | `TASK-IMP-065` | improvement |
| `FR-IMP-066` | `TASK-IMP-066` | improvement |
| `FR-IMP-067` | `TASK-IMP-067` | improvement |
| `FR-IMP-068` | `TASK-IMP-068` | improvement |
| `FR-IMP-069` | `TASK-IMP-069` | improvement |
| `FR-IMP-070` | `TASK-IMP-070` | improvement |
| `FR-IMP-071` | `TASK-IMP-071` | improvement |
| `FR-IMP-072` | `TASK-IMP-072` | improvement |
| `FR-IMP-073` | `TASK-IMP-073` | improvement |
| `FR-IMP-074` | `TASK-IMP-074` | improvement |
| `FR-IMP-075` | `TASK-IMP-075` | improvement |
| `FR-IMP-076` | `TASK-IMP-076` | improvement |
| `FR-IMP-077` | `TASK-IMP-077` | improvement |
| `FR-IMP-078` | `TASK-IMP-078` | improvement |
| `FR-IMP-079` | `TASK-IMP-079` | improvement |
| `FR-IMP-080` | `TASK-IMP-080` | improvement |
| `FR-IMP-081` | `TASK-IMP-081` | improvement |
| `FR-INV-001` | `TASK-INV-001` | inv |
| `FR-INV-002` | `TASK-INV-002` | inv |
| `FR-INV-003` | `TASK-INV-003` | inv |
| `FR-INV-004` | `TASK-INV-004` | inv |
| `FR-INV-005` | `TASK-INV-005` | inv |
| `FR-INV-006` | `TASK-INV-006` | inv |
| `FR-INV-007` | `TASK-INV-007` | inv |
| `FR-INV-008` | `TASK-INV-008` | inv |
| `FR-INV-009` | `TASK-INV-009` | inv |
| `FR-INV-010` | `TASK-INV-010` | inv |
| `FR-INV-011` | `TASK-INV-011` | inv |
| `FR-KB-001` | `TASK-KB-001` | kb |
| `FR-KB-002` | `TASK-KB-002` | kb |
| `FR-KB-003` | `TASK-KB-003` | kb |
| `FR-KB-004` | `TASK-KB-004` | kb |
| `FR-KB-005` | `TASK-KB-005` | kb |
| `FR-KB-006` | `TASK-KB-006` | kb |
| `FR-KB-007` | `TASK-KB-007` | kb |
| `FR-KB-008` | `TASK-KB-008` | kb |
| `FR-KB-009` | `TASK-KB-009` | kb |
| `FR-LEARN-001` | `TASK-LEARN-001` | learn |
| `FR-LEARN-002` | `TASK-LEARN-002` | learn |
| `FR-LEARN-003` | `TASK-LEARN-003` | learn |
| `FR-LEARN-004` | `TASK-LEARN-004` | learn |
| `FR-LEARN-005` | `TASK-LEARN-005` | learn |
| `FR-LEARN-006` | `TASK-LEARN-006` | learn |
| `FR-LEARN-007` | `TASK-LEARN-007` | learn |
| `FR-MCP-001` | `TASK-MCP-001` | mcp |
| `FR-MCP-002` | `TASK-MCP-002` | mcp |
| `FR-MCP-003` | `TASK-MCP-003` | mcp |
| `FR-MCP-004` | `TASK-MCP-004` | mcp |
| `FR-MCP-005` | `TASK-MCP-005` | mcp |
| `FR-MCP-006` | `TASK-MCP-006` | mcp |
| `FR-MCP-007` | `TASK-MCP-007` | mcp |
| `FR-MCP-008` | `TASK-MCP-008` | mcp |
| `FR-MEMORY-101` | `TASK-MEMORY-101` | memory |
| `FR-MEMORY-102` | `TASK-MEMORY-102` | memory |
| `FR-MEMORY-103` | `TASK-MEMORY-103` | memory |
| `FR-MEMORY-104` | `TASK-MEMORY-104` | memory |
| `FR-MEMORY-105` | `TASK-MEMORY-105` | memory |
| `FR-MEMORY-106` | `TASK-MEMORY-106` | memory |
| `FR-MEMORY-107` | `TASK-MEMORY-107` | memory |
| `FR-MEMORY-108` | `TASK-MEMORY-108` | memory |
| `FR-MEMORY-109` | `TASK-MEMORY-109` | memory |
| `FR-MEMORY-110` | `TASK-MEMORY-110` | memory |
| `FR-MEMORY-111` | `TASK-MEMORY-111` | memory |
| `FR-MEMORY-112` | `TASK-MEMORY-112` | memory |
| `FR-MEMORY-113` | `TASK-MEMORY-113` | memory |
| `FR-MEMORY-114` | `TASK-MEMORY-114` | memory |
| `FR-MEMORY-115` | `TASK-MEMORY-115` | memory |
| `FR-MEMORY-116` | `TASK-MEMORY-116` | memory |
| `FR-MEMORY-117` | `TASK-MEMORY-117` | memory |
| `FR-MEMORY-118` | `TASK-MEMORY-118` | memory |
| `FR-MEMORY-119` | `TASK-MEMORY-119` | memory |
| `FR-MEMORY-120` | `TASK-MEMORY-120` | memory |
| `FR-MEMORY-121` | `TASK-MEMORY-121` | memory |
| `FR-MEMORY-122` | `TASK-MEMORY-122` | memory |
| `FR-MEMORY-123` | `TASK-MEMORY-123` | memory |
| `FR-MEMORY-124` | `TASK-MEMORY-124` | memory |
| `FR-MEMORY-201` | `TASK-MEMORY-201` | memory |
| `FR-MEMORY-202` | `TASK-MEMORY-202` | memory |
| `FR-MEMORY-203` | `TASK-MEMORY-203` | memory |
| `FR-MEMORY-204` | `TASK-MEMORY-204` | memory |
| `FR-MEMORY-205` | `TASK-MEMORY-205` | memory |
| `FR-MEMORY-206` | `TASK-MEMORY-206` | memory |
| `FR-MEMORY-207` | `TASK-MEMORY-207` | memory |
| `FR-MEMORY-208` | `TASK-MEMORY-208` | memory |
| `FR-MEMORY-209` | `TASK-MEMORY-209` | memory |
| `FR-MEMORY-210` | `TASK-MEMORY-210` | memory |
| `FR-MEMORY-211` | `TASK-MEMORY-211` | memory |
| `FR-MEMORY-212` | `TASK-MEMORY-212` | memory |
| `FR-MEMORY-213` | `TASK-MEMORY-213` | memory |
| `FR-MEMORY-214` | `TASK-MEMORY-214` | memory |
| `FR-MEMORY-215` | `TASK-MEMORY-215` | memory |
| `FR-MEMORY-216` | `TASK-MEMORY-216` | memory |
| `FR-MEMORY-217` | `TASK-MEMORY-217` | memory |
| `FR-MEMORY-218` | `TASK-MEMORY-218` | memory |
| `FR-MEMORY-219` | `TASK-MEMORY-219` | memory |
| `FR-MEMORY-220` | `TASK-MEMORY-220` | memory |
| `FR-MEMORY-221` | `TASK-MEMORY-221` | memory |
| `FR-MEMORY-222` | `TASK-MEMORY-222` | memory |
| `FR-MEMORY-223` | `TASK-MEMORY-223` | memory |
| `FR-MEMORY-224` | `TASK-MEMORY-224` | memory |
| `FR-MEMORY-225` | `TASK-MEMORY-225` | memory |
| `FR-MEMORY-226` | `TASK-MEMORY-226` | memory |
| `FR-MEMORY-227` | `TASK-MEMORY-227` | memory |
| `FR-MEMORY-228` | `TASK-MEMORY-228` | memory |
| `FR-MEMORY-229` | `TASK-MEMORY-229` | memory |
| `FR-MEMORY-230` | `TASK-MEMORY-230` | memory |
| `FR-MEMORY-231` | `TASK-MEMORY-231` | memory |
| `FR-MEMORY-232` | `TASK-MEMORY-232` | memory |
| `FR-MEMORY-233` | `TASK-MEMORY-233` | memory |
| `FR-MEMORY-234` | `TASK-MEMORY-234` | memory |
| `FR-MEMORY-235` | `TASK-MEMORY-235` | memory |
| `FR-MEMORY-236` | `TASK-MEMORY-236` | memory |
| `FR-MEMORY-237` | `TASK-MEMORY-237` | memory |
| `FR-MEMORY-238` | `TASK-MEMORY-238` | memory |
| `FR-MEMORY-239` | `TASK-MEMORY-239` | memory |
| `FR-MEMORY-240` | `TASK-MEMORY-240` | memory |
| `FR-MEMORY-241` | `TASK-MEMORY-241` | memory |
| `FR-MEMORY-242` | `TASK-MEMORY-242` | memory |
| `FR-MEMORY-243` | `TASK-MEMORY-243` | memory |
| `FR-MEMORY-244` | `TASK-MEMORY-244` | memory |
| `FR-MEMORY-245` | `TASK-MEMORY-245` | memory |
| `FR-MEMORY-246` | `TASK-MEMORY-246` | memory |
| `FR-MEMORY-247` | `TASK-MEMORY-247` | memory |
| `FR-MEMORY-248` | `TASK-MEMORY-248` | memory |
| `FR-MEMORY-249` | `TASK-MEMORY-249` | memory |
| `FR-MEMORY-250` | `TASK-MEMORY-250` | memory |
| `FR-MEMORY-251` | `TASK-MEMORY-251` | memory |
| `FR-MEMORY-252` | `TASK-MEMORY-252` | memory |
| `FR-MEMORY-253` | `TASK-MEMORY-253` | memory |
| `FR-MEMORY-254` | `TASK-MEMORY-254` | memory |
| `FR-MEMORY-255` | `TASK-MEMORY-255` | memory |
| `FR-MEMORY-256` | `TASK-MEMORY-256` | memory |
| `FR-MEMORY-257` | `TASK-MEMORY-257` | memory |
| `FR-MEMORY-258` | `TASK-MEMORY-258` | memory |
| `FR-MEMORY-259` | `TASK-MEMORY-259` | memory |
| `FR-MEMORY-260` | `TASK-MEMORY-260` | memory |
| `FR-MEMORY-261` | `TASK-MEMORY-261` | memory |
| `FR-OBS-001` | `TASK-OBS-001` | obs |
| `FR-OBS-002` | `TASK-OBS-002` | obs |
| `FR-OBS-003` | `TASK-OBS-003` | obs |
| `FR-OBS-004` | `TASK-OBS-004` | obs |
| `FR-OBS-005` | `TASK-OBS-005` | obs |
| `FR-OBS-006` | `TASK-OBS-006` | obs |
| `FR-OBS-007` | `TASK-OBS-007` | obs |
| `FR-OBS-008` | `TASK-OBS-008` | obs |
| `FR-OBS-009` | `TASK-OBS-009` | obs |
| `FR-OKR-001` | `TASK-OKR-001` | okr |
| `FR-OKR-002` | `TASK-OKR-002` | okr |
| `FR-OKR-003` | `TASK-OKR-003` | okr |
| `FR-OKR-004` | `TASK-OKR-004` | okr |
| `FR-OKR-005` | `TASK-OKR-005` | okr |
| `FR-OKR-006` | `TASK-OKR-006` | okr |
| `FR-OKR-007` | `TASK-OKR-007` | okr |
| `FR-PLUGIN-001` | `TASK-PLUGIN-001` | plugin |
| `FR-PLUGIN-002` | `TASK-PLUGIN-002` | plugin |
| `FR-PLUGIN-003` | `TASK-PLUGIN-003` | plugin |
| `FR-PLUGIN-004` | `TASK-PLUGIN-004` | plugin |
| `FR-PLUGIN-005` | `TASK-PLUGIN-005` | plugin |
| `FR-PLUGIN-006` | `TASK-PLUGIN-006` | plugin |
| `FR-PLUGIN-007` | `TASK-PLUGIN-007` | plugin |
| `FR-PLUGIN-008` | `TASK-PLUGIN-008` | plugin |
| `FR-PORTAL-001` | `TASK-PORTAL-001` | portal |
| `FR-PORTAL-002` | `TASK-PORTAL-002` | portal |
| `FR-PORTAL-003` | `TASK-PORTAL-003` | portal |
| `FR-PORTAL-004` | `TASK-PORTAL-004` | portal |
| `FR-PORTAL-005` | `TASK-PORTAL-005` | portal |
| `FR-PORTAL-006` | `TASK-PORTAL-006` | portal |
| `FR-PORTAL-007` | `TASK-PORTAL-007` | portal |
| `FR-PORTAL-008` | `TASK-PORTAL-008` | portal |
| `FR-PROJ-001` | `TASK-PROJ-001` | proj |
| `FR-PROJ-002` | `TASK-PROJ-002` | proj |
| `FR-PROJ-003` | `TASK-PROJ-003` | proj |
| `FR-PROJ-004` | `TASK-PROJ-004` | proj |
| `FR-PROJ-005` | `TASK-PROJ-005` | proj |
| `FR-PROJ-006` | `TASK-PROJ-006` | proj |
| `FR-PROJ-007` | `TASK-PROJ-007` | proj |
| `FR-PROJ-008` | `TASK-PROJ-008` | proj |
| `FR-PROJ-009` | `TASK-PROJ-009` | proj |
| `FR-PROJ-010` | `TASK-PROJ-010` | proj |
| `FR-PROJ-011` | `TASK-PROJ-011` | proj |
| `FR-PROJ-012` | `TASK-PROJ-012` | proj |
| `FR-PROJ-013` | `TASK-PROJ-013` | proj |
| `FR-PROJ-014` | `TASK-PROJ-014` | proj |
| `FR-PROJ-015` | `TASK-PROJ-015` | proj |
| `FR-PROJ-016` | `TASK-PROJ-016` | proj |
| `FR-PROJ-017` | `TASK-PROJ-017` | proj |
| `FR-PROJ-018` | `TASK-PROJ-018` | proj |
| `FR-RES-001` | `TASK-RES-001` | res |
| `FR-RES-002` | `TASK-RES-002` | res |
| `FR-RES-003` | `TASK-RES-003` | res |
| `FR-RES-004` | `TASK-RES-004` | res |
| `FR-RES-005` | `TASK-RES-005` | res |
| `FR-REW-001` | `TASK-REW-001` | rew |
| `FR-REW-002` | `TASK-REW-002` | rew |
| `FR-REW-003` | `TASK-REW-003` | rew |
| `FR-REW-004` | `TASK-REW-004` | rew |
| `FR-REW-005` | `TASK-REW-005` | rew |
| `FR-REW-006` | `TASK-REW-006` | rew |
| `FR-REW-007` | `TASK-REW-007` | rew |
| `FR-REW-008` | `TASK-REW-008` | rew |
| `FR-REW-009` | `TASK-REW-009` | rew |
| `FR-REW-010` | `TASK-REW-010` | rew |
| `FR-SKILL-101` | `TASK-SKILL-101` | skill |
| `FR-SKILL-102` | `TASK-SKILL-102` | skill |
| `FR-SKILL-103` | `TASK-SKILL-103` | skill |
| `FR-SKILL-104` | `TASK-SKILL-104` | skill |
| `FR-SKILL-105` | `TASK-SKILL-105` | skill |
| `FR-SKILL-106` | `TASK-SKILL-106` | skill |
| `FR-SKILL-107` | `TASK-SKILL-107` | skill |
| `FR-SKILL-108` | `TASK-SKILL-108` | skill |
| `FR-SKILL-109` | `TASK-SKILL-109` | skill |
| `FR-SKILL-110` | `TASK-SKILL-110` | skill |
| `FR-SKILL-111` | `TASK-SKILL-111` | skill |
| `FR-SKILL-112` | `TASK-SKILL-112` | skill |
| `FR-SKILL-113` | `TASK-SKILL-113` | skill |
| `FR-SKILL-114` | `TASK-SKILL-114` | skill |
| `FR-SKILL-115` | `TASK-SKILL-115` | skill |
| `FR-SKILL-116` | `TASK-SKILL-116` | skill |
| `FR-SKILL-117` | `TASK-SKILL-117` | skill |
| `FR-SKILL-118` | `TASK-SKILL-118` | skill |
| `FR-SKILL-119` | `TASK-SKILL-119` | skill |
| `FR-SKILL-120` | `TASK-SKILL-120` | skill |
| `FR-SKILL-201` | `TASK-SKILL-201` | skill |
| `FR-TPL-001` | `TASK-TPL-001` | templates |
| `FR-TEN-001` | `TASK-TEN-001` | ten |
| `FR-TEN-002` | `TASK-TEN-002` | ten |
| `FR-TEN-003` | `TASK-TEN-003` | ten |
| `FR-TEN-004` | `TASK-TEN-004` | ten |
| `FR-TEN-005` | `TASK-TEN-005` | ten |
| `FR-TEN-101` | `TASK-TEN-101` | ten |
| `FR-TEN-102` | `TASK-TEN-102` | ten |
| `FR-TEN-103` | `TASK-TEN-103` | ten |
| `FR-TEN-104` | `TASK-TEN-104` | ten |
| `FR-TEN-105` | `TASK-TEN-105` | ten |
| `FR-TEN-106` | `TASK-TEN-106` | ten |
| `FR-TEN-107` | `TASK-TEN-107` | ten |
| `FR-TEN-201` | `TASK-TEN-201` | ten |
| `FR-TEN-202` | `TASK-TEN-202` | ten |
| `FR-TIME-001` | `TASK-TIME-001` | time |
| `FR-TIME-002` | `TASK-TIME-002` | time |
| `FR-TIME-003` | `TASK-TIME-003` | time |
| `FR-TIME-004` | `TASK-TIME-004` | time |
| `FR-TIME-005` | `TASK-TIME-005` | time |
| `FR-TIME-006` | `TASK-TIME-006` | time |
| `FR-TIME-007` | `TASK-TIME-007` | time |
| `FR-TIME-008` | `TASK-TIME-008` | time |
| `FR-TIME-009` | `TASK-TIME-009` | time |

---

# Second wave — init to install

Date: 2026-07-16.

The first wave renamed the *atom* (feature-request to task). This wave finishes the
*verb*. `init.sh` had already become `install.sh`, `version.sh` and
`lib/status-page.sh`, but the name `init` survived exactly where a content codemod
does not look: a directory, a binary, an npm package name, and every doc that told a
user what to type.

## Rule

```
tools/cyberos-init/          ->  tools/cyberos-install/
cli/bin/cyberos-init.mjs     ->  cli/bin/cyberos-install.mjs
npm name: cyberos-init       ->  cyberos-install
npx cyberos-init             ->  npx cyberos-install
log prefix "cyberos-init:"   ->  "cyberos-install:"

init.sh <repo>               ->  install.sh <repo>
init.sh --check <repo>       ->  version.sh <repo>
init.sh --page               ->  lib/status-page.sh
```

Mapped by shape, not blanket: `--check` and `--page` are separate commands now, so a
flat `init.sh -> install.sh` would have minted two more broken pointers.

## Why now

`cyberos-init` was the npm package name and the `npx` command — externally visible
surface. It was never published (registry 404; no publish workflow) and there are no
external consumers pre-1.0. Renaming it before the first release costs nothing.
Renaming it after is a breaking change for every consumer.

## Archived evidence was rewritten (D3 again)

430 sites across 124 files; 45 of those files are archived task specs, `.workflow`
evidence, review docs, and the changelog. Unlike an `FR-` id, these are **path**
references — a path that no longer resolves is simply broken, so tracking the move
is the accurate act rather than a falsification. Read any `tools/cyberos-install/...`
path in an artefact dated before 2026-07-16 as `tools/cyberos-init/...`.

## What the gate learned

This wave survived the first one because `.pre-commit-hooks/no-legacy-fr-vocabulary.sh`
could not see it. Two defects, both fixed:

- `--diff-filter=ACM` omits `R`. A pure `git mv` reports R100, so a rename-only commit
  was invisible to the gate — the one operation this epoch most needed policed
  (git mv != re.sub) was the one it was blind to.
- The gate grepped each staged file's body and never its path, so
  `plugin/skills/ship-feature-requests/SKILL.md` passed clean: renamed contents, stale
  directory. The build shipped it beside an empty `plugin/skills/ship-tasks/`, and the
  flagship skill could not load in any channel.
