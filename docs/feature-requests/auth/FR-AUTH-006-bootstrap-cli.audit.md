---
fr_id: FR-AUTH-006
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
---

## §1 — Verdict summary

FR-AUTH-006 expanded from 73 lines to ~720. Added 9 §1 clauses (#6 initial signing key in bootstrap, #8 rotate-keys subcommand, #9 sweepers subcommand, #11 production-reset triple guard, #12 standardised exit codes, #13 stdout summary, #14 OTel spans, expanded #2 with env-var fallback + masking, expanded #4 audit row payload). 8 §2 rationale paragraphs. Full Rust skeleton + cron config in §3. 20 ACs. 8 full Rust test bodies. 16 failure modes. 9 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — No initial signing key creation; FR-AUTH-004 can't issue tokens after bootstrap
First-pass created tenant 0 + root-admin only. FR-AUTH-004's token issuance reads from `signing_keys` table — without bootstrap creating one, first token request 500s. Resolved: §1 #6 normative + §3 invokes `jwks::rotation::generate_new_signing_key_in_tx`; AC #1 + AC #14 verify; bootstrap audit row carries `initial_signing_key_kid`.

### ISS-002 — No production-reset safety guard
First-pass had `--reset --confirm` but no environment awareness. Production reset wipes everything. Resolved: §1 #11 triple gate (--reset + --confirm + --force-prod-reset + interactive Y + tty check); ACs #6/7/8/9 cover each path; §10 rows + §11 note.

### ISS-003 — No sweepers (sessions, idempotency, retired keys grow unbounded)
FR-AUTH-004 + FR-AUTH-001 + FR-AUTH-005 all said "sweeper deletes after N hours" without specifying where. Resolved: §1 #9 sweepers subcommand; §3 implementation; AC #16 + #17 + cron schedule in §6.

### ISS-004 — No rotate-keys subcommand for emergency rotation
First-pass left rotation as quarterly cron only. Suspected compromise needs immediate rotation. Resolved: §1 #8 rotate-keys subcommand; AC #15 + §5 test; §11 documents quarterly cron + ad-hoc usage.

### ISS-005 — Standardised exit codes missing
First-pass §4 said "exits 1 with already initialised" — but distinct failure modes (CI scripts) need distinct codes. Resolved: §1 #12 ExitCode enum (0/1/2/3/4/5/6); §3 main.rs maps; tests assert specific codes.

### ISS-006 — Plaintext password in CLI summary risk
First-pass §6 had `println!("Bootstrap complete. Root admin: {}", email)` — echoing email is mostly fine, but the pattern of "echo what the user typed" risks future regressions echoing password. Resolved: §1 #5 explicitly forbids password echo; §1 #13 summary excludes email (subject_id only); §5 test asserts no plaintext password in stdout/BRAIN/logs.

## §3 — Resolution

All 6 mechanical revisions applied. **Score = 10/10.**

---

## §10 — Implementation audit (code-vs-spec)

> Added 2026-05-18 (session 20) by `chief-technology-officer/implement-backlog-frs` workflow.

### §10.1 — Verdict

**Implementation status:** BLOCKED. 6 spec-vs-code gaps (4 high + 1 critical + 1 medium). Biggest gap is **structural**: FR §1 #8-11 wants a unified `cyberos-auth` CLI with `bootstrap`/`rotate-keys`/`sweepers` subcommands; current code is a single-shot `cyberos-auth-bootstrap` binary.

### §10.2 — Gap list

| # | Spec ref | Gap | Severity | Effort | Status |
|---|---|---|---|---|---|
| G-001 | §1 #4 | `auth.bootstrap_completed` BRAIN row not emitted; no BRAIN bridge wired | high | 30 LOC | open |
| G-002 | §1 #7 | Idempotent re-run uses `ON CONFLICT DO UPDATE` (silent overwrite); spec wants detect-and-exit-5 (`AlreadyInitialised`) | medium | 15 LOC | open |
| G-003 | §1 #8 | `rotate-keys` subcommand absent | high | 80 LOC + restructure | open |
| G-004 | §1 #9 | `sweepers` subcommand absent (expired sessions + idempotency rows + retired keys) | high | 100 LOC | open |
| G-005 | §1 #10 | `--reset --confirm` flag absent | medium | 40 LOC | open |
| G-006 | §1 #11 | Production-environment safety guard for `--reset` absent | critical | 20 LOC | open |

**Structural observation:** G-003+G-004+G-005+G-006 require a top-level CLI router (clap-based). Suggested: keep `cyberos-auth-bootstrap` binary as transitional alias; add new binary `cyberos-auth` with subcommands; ship both side-by-side; remove the alias in a follow-up release.

### §10.3 — Audit-fix log

| ts | gap | change | tests | cargo result | commit |
|---|---|---|---|---|---|
| _empty — fix loop has not started_ | | | | | |

### §10.4 — BACKLOG.md mutations

| ts | line | from | to | mutation_kind |
|---|---|---|---|---|
| 2026-05-18T15:20:00Z | 217 | `planned` | `[BLOCKED: 6 spec gaps — see auth/.workflow/FR-AUTH-006/]` | status-cell-only |
| _pending: reference updated to .audit.md §10 (next mutation)_ | | | | |
| _pending audit-fix completion_ | 217 | (above) | `shipped + strict-audited` | status-cell-only |

### §10.5 — Working notes

**Code state at audit time:**
- `services/auth/src/bin/bootstrap.rs` (213 LOC) handles only the bootstrap path.
- Confirms root tenant via `SELECT EXISTS FROM tenants WHERE id = nil_uuid`.
- Root-admin INSERT … ON CONFLICT DO UPDATE in single tx with `SET LOCAL app.current_tenant_id` GUC.
- `bcrypt::DEFAULT_COST` (12).
- `ensure_signing_key` creates RSA-2048 if no active key (90-day TTL).
- Best-effort `subject_roles` INSERT when migration 0007 is present.

**Edge-case-matrix rows (12 total):** NULL_INPUT × 2 · BOUNDARY × 2 · MALFORMED × 2 · CONCURRENT × 2 · SECURITY × 2 (--reset prod safety + plaintext-password leak) · DEGRADATION × 2.

**Coverage-gate verify command:**
```bash
cd services && cargo +1.85.0 test --workspace bootstrap
```

---

*End of FR-AUTH-006 audit. Spec quality: PASS 10/10. Implementation: BLOCKED (6 gaps; audit-fix loop pending).*
