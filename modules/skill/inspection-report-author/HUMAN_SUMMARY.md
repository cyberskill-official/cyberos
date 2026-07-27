# `inspection-report-author` — human summary

After each run, emit a short chat summary:

```
inspection-report-author complete
Artefact/kind: inspection-report@1
Path: <path>
Verdict / next: <pass|route-back|HITL> → <next skill or halt>
Machine floor: <tool + exit>
```

Never claim `/harden` or ship-tasks equivalence. Inspection remediation is `/harden`; backlog `class: improvement` is `/ship-tasks`.
