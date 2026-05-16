---
fr_id: FR-OBS-006
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
---

## §1 — Verdict summary

FR-OBS-006 expanded from 138 lines to ~520. Added 6 §1 clauses (#3 flagged-tenants with hot-reload, #4 per-route latency budgets, #6 first-match precedence, #11 hot-reload mechanism, #12 buffer-depth alarm, #13 per-tenant rate override). 7 §2 rationale paragraphs. Full collector config + flag-tenant CLI hook in §3. 13 ACs. 2 bash test scripts. 10 failure modes. 6 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Flagged-tenants config mechanism unspecified
First-pass §1 #3 mentioned "from FR-AI-021 operator CLI" without file format or hot-reload. Resolved: §1 #3 + flagged_tenants.yaml + file-watch reload + AC #6 + #9.

### ISS-002 — Per-route latency budgets not specified
First-pass had global 2000ms threshold. ai-gateway calls are normally 1-3s. Resolved: §1 #4 + route_latency_budgets.yaml with per-route thresholds; AC #5.

### ISS-003 — Policy ordering / first-match precedence unspecified
What happens with error+slow trace? Counted twice or once? Resolved: §1 #6 first-match wins; AC #10 asserts.

### ISS-004 — Buffer-depth alarm missing
First-pass had `num_traces: 100000` with no alarm at saturation. Resolved: §1 #12 + `obs_sampling_buffer_depth` gauge + sev-2 at 90%.

### ISS-005 — Hot-reload mechanism not specified
Operator flags tenant via CLI but how does collector pick up without restart? Resolved: §1 #11 + file-watch extension; collector reloads within 30s.

### ISS-006 — CLI hook to FR-AI-021 missing
First-pass mentioned "from FR-AI-021 operator CLI" but no integration shown. Resolved: §3 `flag_tenant.rs` subcommand + audit row `obs.tenant_flagged_for_sampling`.

## §3 — Resolution

All 6 mechanical revisions applied. **Score = 10/10.**

---

*End of FR-OBS-006 audit.*
