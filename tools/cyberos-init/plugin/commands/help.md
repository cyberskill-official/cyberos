---
description: What the CyberOS plugin does and how to use it — commands, FR lifecycle, human gates, where things live.
---
Orient the user in the CyberOS plugin. Present, concisely and in this order:

1. What this is: CyberOS turns work into feature requests (FRs) driven through one governed lifecycle — implement → review → test → done — with the human holding the two acceptance gates.

2. The commands:
   - `/install [repo]` — install CyberOS into a repo once (idempotent re-vendor).
   - `/update` — manual update check; apply on request (`update.sh --apply`). Soft checks already run on any `.cyberos` use.
   - `/status` — manual version / rules_sha report.
   - `/ship-feature-requests` — drive the next eligible FR, halting at the two human gates.
   - `/help` — this overview.
   - Uninstall (shell): `bash .cyberos/uninstall.sh`.

3. The two human gates (non-negotiable): reviewing → ready_to_test and testing → done are set by the human only. Never set `done`, push, merge, or deploy without an operator instruction.

4. Where things live after install:
   - Doctrine: `.cyberos/cuo/`
   - Gates: `bash .cyberos/cuo/gates/run-gates.sh` (`.cyberos/gates.env`)
   - Agent entry: root `AGENTS.md` → `.cyberos/AGENT-ENTRY.md` (same pattern as `CLAUDE.md` / `GEMINI.md`)
   - Memory protocol: `.cyberos/memory/AGENTS.md`; store: `.cyberos/memory/store/`
   - FRs: `docs/feature-requests/`

5. Docs: https://cyberos.cyberskill.world/docs

If the current repo has no `.cyberos/`, suggest `/install`.
