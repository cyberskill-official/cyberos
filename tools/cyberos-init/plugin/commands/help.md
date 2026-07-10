---
description: What the CyberOS plugin does and how to use it - commands, the FR lifecycle, the two human gates, where everything lives.
---
Orient the user in the CyberOS plugin. Present, concisely and in this order:

1. What this is: CyberOS turns work into feature requests (FRs) driven through one governed lifecycle - implement -> review -> test -> done - with the human holding the two acceptance gates. The agent does the work; the human decides.

2. The commands:
   - `/init [repo]` - install CyberOS into a repo (or update it); self-hosts from the plugin if no payload is around.
   - `/ship-feature-requests` - drive the next eligible FR from `docs/feature-requests/BACKLOG.md` end to end, halting at the two human gates.
   - `/update` - compare the repo's installed CyberOS version against what is available and apply an update on request.
   - `/changelog` - show the installed version and what changed recently.
   - `/help` - this overview.

3. The two human gates (non-negotiable): reviewing -> ready_to_test and testing -> done are set by the human only, after the agent presents its review packet and test evidence. An agent must never set `done`, push, merge, or deploy.

4. Where things live in an initialised repo: doctrine at `.cyberos/cuo/` (workflow, execution discipline, status contract), gate wiring at `.cyberos/gates.env` (runner: `bash .cyberos/cuo/gates/run-gates.sh` when vendored), FRs and the single backlog at `docs/feature-requests/`, agent entry at `.cyberos/AGENT-ENTRY.md`, BRAIN store at `.cyberos/memory/store/`.

5. Learn more: the docs site at https://cyberos.cyberskill.world/docs - start with "Ship your first feature request" (cuo module -> Guides) and "Install, update and operate CyberOS in any repo" (tools -> cyberos-init).

If the current repo has no `.cyberos/`, end by suggesting `/init`.
