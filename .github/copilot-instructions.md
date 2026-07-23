This repository runs **CyberOS**. Canonical agent instructions: `.cyberos/AGENT-ENTRY.md`.

Root `AGENTS.md` is the thin CyberOS workflow spine (not the Layer-1 memory protocol). Memory protocol: `.cyberos/memory/AGENTS.md` (normative source: `modules/memory/cyberos/data/AGENTS.md`).

Work is tasks; HITL is required at the two human-acceptance gates; run gates with `bash .cyberos/cuo/gates/run-gates.sh`. Never push, deploy, or merge without an explicit operator instruction.
