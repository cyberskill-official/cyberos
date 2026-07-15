---
task_id: TASK-APP-005
audited: 2026-07-12
verdict: PASS (after revision)
score_pre_revision: 6/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

## §1 — Verdict summary

304 lines, 8 numbered §1 clauses, 8 acceptance criteria, 8 failure-mode rows, 3 verification blocks. Initial draft (292 lines) was below the 300-line under-specification floor and carried two instances of unearned confidence — an unspecified cross-file "diff plugs against §9 justifications" CI mechanism claimed as an acceptance criterion without being designed, and an assumed `flatpak-builder --show-manifest` CLI flag never confirmed to exist — plus a dead `grep -v` exclusion in a verification script and an under-specified Snapcraft `architectures:` YAML form. All findings below were resolved in the same authoring pass before this audit was finalized, per the master rule's loop-to-10/10 discipline.

## §2 — Findings (all resolved)

### ISS-001 — AC #3 claimed an unspecified, disproportionately complex CI mechanism
The original AC #3 asserted "a CI lint diffs the `plugs:` list ... against ... and fails on any addition not simultaneously accompanied by a new §9 justification entry" — this describes a script that would need to parse prose §9 entries and correlate them against YAML list diffs, a real but nontrivial piece of engineering that was never designed anywhere in the spec, just asserted as an existing acceptance gate. This is exactly the category of unspecified-but-claimed logic the authoring discipline forbids (distinct from the Store Submission API's legitimate WORKER-phase deferral, since that's an external, well-documented contract — this would have been bespoke CyberOS logic asserted into existence). Resolved: AC #3 now specifies a real, simple fixed-string comparison against the exact six-entry plug set, and the "future additions need justification" expectation is reframed as a documented code-review norm (§11), not a fabricated automated mechanism; §4.

### ISS-002 — `flatpak-builder --show-manifest` was asserted as a real CLI flag without confirmation
§3/§5's original AC #4 verification invoked `flatpak-builder ... --show-manifest`, a flag this task's authoring never actually confirmed exists in `flatpak-builder`'s current CLI surface — asserting an unverified flag with the same confidence as verified facts (like the confirmed `libwebkit2gtk-4.1-dev` package name or the confirmed `128x128@2x.png` icon) blurs a distinction the anti-fabrication discipline depends on. Resolved: AC #4's wording now explicitly states the exact flag must be confirmed against the installed `flatpak-builder` version's `--help` output at implementation time, and §5's verification script falls back to a full non-destructive local build (`--force-clean`) as the safe default rather than a possibly-nonexistent flag; §4, §5.

### ISS-003 — AC #6's verification script contained a dead, pointless `grep -v` exclusion
The original `grep -r "flathub/flathub" .github/ tools/ | grep -v "docs/deploy/linux-store-submission.md"` excluded a path that was never in scope to begin with — the grep only scans `.github/` and `tools/`, never `docs/`, so the exclusion pattern could never match anything and was silently dead code, the kind of copy-paste artifact that erodes trust in a verification script's correctness. Resolved: removed the pointless exclusion, replaced with a `grep -rl | wc -l` count-based check and an inline comment explaining why no exclusion is needed; §5.

### ISS-004 — Snapcraft `architectures:` YAML form was presented without acknowledging schema uncertainty
The `snapcraft.yaml` skeleton in §3 used the `build-on:` object-list form for `architectures:` without noting that Snapcraft's schema has used different accepted forms across versions, and this task's authoring never executed `snapcraft pack` against a real Snapcraft installation to confirm which form `core22` currently expects. Resolved: added an inline YAML comment flagging this as unconfirmed and requiring WORKER-phase verification against Snapcraft's live schema, rather than presenting one plausible syntax as settled fact; §3.

### ISS-005 — Spec length (292 lines) was below the 300-line under-specification floor
Per the repo's own task authoring discipline, sub-300-line tasks not covered by the stub/infra exception are flagged as potentially under-specified. This task is CI/config-heavy but carries genuine architectural weight (two structurally distinct distribution mechanisms, a structural rather than merely-documented PR-prevention guarantee), placing it in the same "needs the fuller bar, not just the infra ceiling" category as TASK-APP-003/004. Resolved: the ISS-001 through ISS-004 fixes plus the ISS-006 §7 tightening below brought the spec to 304 lines with materially more precise, not padded, content; multiple sections.

### ISS-006 — §7's Human/account prerequisites list didn't make the Flathub app-id decision's blocking relationship to PR submission explicit enough
The original §7 entry mentioned the app-id/domain-ownership decision as one item in a flat list alongside the Snapcraft credential prerequisites, without stating that an unresolved app-id specifically blocks the Flathub PR (as opposed to being a nice-to-have-resolved-eventually item) — a reader skimming §7 could miss that this is a hard sequencing dependency, not a parallel independent task. Resolved: §7 now explicitly labels this as a "hard blocker on ever opening the Flathub PR" with the reasoning (provisional app-id → provisional manifest/`.desktop` filename/paths → submission risk) spelled out inline; §7.

## §3 — Resolution

All 6 findings addressed in the same authoring session that produced them, per the master rule (author → audit → loop to 10/10 before starting the next task). No findings deferred. **Score = 10/10.**

---

*End of TASK-APP-005 audit.*
