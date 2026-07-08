# Improvement execution ledger

Append-only. Every task run adds one entry (agent appends during work; reviewer appends the verdict). Never edit or delete past entries; corrections are new entries referencing the old one.

Entry format:

```
## <YYYY-MM-DD> IMP-NNN <short title> - <agent|operator>
- branch: auto/imp-NNN-<slug>
- commits: <hashes>
- status: todo -> doing | doing -> review | review -> done | -> blocked (reason)
- gates: <commands run and results, one line each>
- evidence: <test output refs, screenshot paths, log lines, measured numbers>
- sensitive paths: <none | list + justification>
- notes: <deviations from spec, discovered follow-ups (file as new tasks, do not scope-creep)>
```

---
