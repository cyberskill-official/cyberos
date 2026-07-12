---
fr_id: FR-CUO-207
audited: 2026-07-12
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

# FR-CUO-207 audit

## §1 - Verdict summary

Audited for detection honesty (never invent a command), override granularity, and update-safety of the scaffolded config. The per-key resolution model and the provenance line survived scrutiny; JVM coverage detection was correctly descoped rather than overpromised. Traceability closes over t01-t08 in tools/cyberos-init/tests/test_gate_autodetect.sh.

## §2 - Findings (all resolved)

### ISS-001 all-or-nothing override was the wrong grain
Real repos deviate on ONE gate. Resolved: §1 #2 per-key override; AC 4 mixed-provenance fixture.

### ISS-002 re-init clobber risk
A config the operator edited must survive updates byte-identically. Resolved: §1 #3 scaffold-once discipline (same rule as BACKLOG/AGENTS), AC 5.

### ISS-003 vendored-marker false fires
node_modules/package.json would detect Node in every JS-adjacent repo. Resolved: root-only scanning pinned (§10 #2) with a nested-marker fixture in t03's family.

### ISS-004 JVM coverage overpromise
Draft mapped a coverage command for Maven/Gradle; jacoco/kover wiring is repo-specific and a wrong default poisons the coverage gate. Resolved: deliberately undetected, config named as the sanctioned path (§9), keeping the never-guess rule intact.

### ISS-005 malformed config half-apply
A partially-read config running SOME gates is worse than failing. Resolved: §1 #7 loud fail with line number, no gate runs, AC 8.

### ISS-006 invisible command provenance
Fleet debugging dies on "which command even ran?". Resolved: §1 #4 one provenance line per gate (config|autodetect:<stack>|absent), asserted in AC 4.

## §3 - Resolution

All six findings addressed as cited. Blocks FR-CUO-208 as declared; threshold hook matches FR-SKILL-118's rubric constant. **Score = 10/10.**

*End of FR-CUO-207 audit.*
