# Playground triage

A worth-it pass over the outside sources sitting in `playground/` (gitignored, Stage 0). The question
for each: does it strengthen a real CyberOS module enough to earn a Stage-1 absorption, and if so,
which one and how. Written 2026-06-24. Nothing here is absorbed yet; this is the shortlist that decides
what gets a deeper verdict note.

Read against the module roster and the honest frontier in `docs/tasks/remaining-build-plan.md`.
The CyberSkill-authored items (CYBEROS_STRATEGY.md, CyberOS-docs/, the cyberos-*.md notes,
cyberos-memory-workbench-archive/, cyberskill-vn-skills/, the agentic-memory mhtml and the
dreaming-agents mp4) are our own prior work, not outside sources, so they are out of scope here - the
memory archive already has a path forward via `cyberos import` (see CONSUMED-FROM-WORKBENCH.md).

## Tier 1 - clear module fit, deep-dive candidates

| Source | Stack | What it is | Strengthens | Why it is worth a verdict note |
|---|---|---|---|---|
| code-review-graph | python, MCP | Builds a Tree-sitter structural map of a codebase, tracks changes incrementally, and serves precise context to an agent over MCP so it reads only what changed | cuo (the code-review step) + mcp | Directly attacks a real cuo cost: re-reading the whole tree per review. And it is already an MCP server, so it federates through the gateway exactly like the obs triage tool just landed - a second real federated tool, not a demo one. |
| hermes-agent | python | Self-improving agent with a built-in learning loop: creates skills from experience, improves them during use, persists knowledge, recalls across sessions, runs scheduled automations | cuo (TASK-CUO-204 dream loop) + skill | The closest existing implementation of the self-evolution the dream loop is built for. Mine it for the propose/refine/persist patterns; it is a whole framework, so the move is pattern-borrowing, not vendoring wholesale. |
| claude-mem | node | A Claude memory system (capture, recall, compaction) | memory | Overlaps the memory module head-on. The verdict note decides whether its approach beats ours on any axis (compaction, recall ranking) or is duplicative. Honest chance the answer is "we already do this better" - which is a valid Stage-0 outcome. |

## Tier 2 - methodology and skill corpora (reference, likely not vendored)

| Source | Stack | What it is | Use |
|---|---|---|---|
| superpowers | node | A full coding-agent methodology as composable skills (spec, plan, subagent-driven TDD, YAGNI, DRY) | Reference for the cuo build chain + skill catalog. Compare against EXECUTION-DISCIPLINE.md; borrow what is sharper. |
| gstack | node+skills | A "virtual engineering team" - 23 specialist slash-command roles + 8 tools, MIT | Same category as superpowers; role definitions could inform skill-catalog entries. |
| ECC | python | Harness-native operator system: skills, instincts, memory optimization, continuous learning, security scanning | Broad and large. Mine specific subsystems (security scanning could inform the caf gate); do not absorb wholesale. |
| agentskills, anthropics-skills, andrej-karpathy-skills, mattpocock-skills, claude-cookbooks, everything-claude-code, academic-research-skills, system-prompts-and-models-of-ai-tools | mixed | Skill collections and reference corpora | Inputs to the skill module's catalog and golden-set inspiration, not module code. Treat as one corpus; harvest patterns, do not vendor. |

## Tier 3 - park (low relevance or unclear)

| Source | Stack | Why parked |
|---|---|---|
| openhuman | rust | A whole personal-AI product in early beta. Interesting that it is Rust, but it strengthens no specific current module. Revisit only if a named subsystem is wanted. |
| mini-tokyo-3d | node | A 3D transit visualization. No fit with any current module; at most a tangential dashboard-viz reference. |
| designmd | unclear | Purpose not yet established; needs a one-line look before it is even Tier 2. |

## Recommendation

Start with code-review-graph. It has the clearest module fit (cuo review cost + an MCP tool), the
absorption seam is the same federation pattern just proven with obs triage, and Python means I can read
and even prototype it here in the sandbox rather than waiting on the Mac. hermes-agent is the strongest
second, as a pattern source for the dream loop. claude-mem is worth a verdict mostly to confirm whether
it beats the memory module on anything; expect a short note either way.

Next action per pick: a Stage-0 verdict note at `docs/absorptions/<source>-absorption.md` - read in
full, state overlap/gaps/license, and make the worth-it call before any task mapping or vendoring.
