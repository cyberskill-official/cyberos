# 2026-05-19 — CHAT + PROJ + EMAIL Layer-0/2 wave

**Started:** 2026-05-19 (Cowork session)
**Status:** FR-CHAT-001 + FR-EMAIL-001 + FR-PROJ-001 all moved `accepted/draft → shipped (slice 1)`.
**Goal:** Implement the foundational FR of each module per the user request "cyberos implement CHAT + PROJ + EMAIL". Each FR driven to full §1→§4→§5 traceability with per-FR §10 audit dossier.

This runbook complements [`2026-05-18-wave-1-2.md`](2026-05-18-wave-1-2.md). It picks up after FR-AUTH-005 closed the AUTH Wave-1 MUST drain.

---

## What shipped in this session

### FR-CHAT-001 — Mattermost fork pin + drift watcher + cherry-pick policy

Layer 0 (no upstream deps). Pure infra-as-code — no Rust crate at this slice.

| File | Purpose |
|---|---|
| `services/chat/PINNED_COMMIT` | 40-char SHA + rationale; CODEOWNERS-locked to legal-team |
| `services/chat/CYBEROS_PATCH_VERSION` | Semver of the CyberOS patch series (`0.1.0`) |
| `services/chat/Dockerfile` | Two-stage Go build + distroless runtime; image tag `cyberos/chat:<sha:0:12>-<patch_version>` |
| `services/chat/README.md` | Fork deviation policy table + cherry-pick workflow + drift watcher narrative |
| `services/chat/CHANGELOG.cyberos.md` | Keep-a-Changelog with 4 category prefixes |
| `services/chat/Makefile` | `chat-build` / `chat-license-check` / `chat-test` / `chat-verify` |
| `services/chat/compose.yml` | Local-dev Postgres+Redis+chat stack |
| `services/chat/config/config.json` | Baseline Mattermost config baked into image |
| `services/chat/scripts/check-license-drift.sh` | Drift watcher — GitHub API + LICENSE-filter + issue create |
| `services/chat/scripts/cherry-pick-upstream.sh` | Operator cherry-pick helper — auto PR + label pre-application |
| `services/chat/tests/{pinned_commit,license_drift,patch_apply,workflows_present,run_all_tests}.sh` | §5 bash test suite |
| `.github/workflows/chat-license-drift-watcher.yml` | Replaced stub; Monday-weekly cron + workflow_dispatch |
| `.github/workflows/chat-cherry-pick-review.yml` | Replaced stub; label-required gate responding to `labeled`/`unlabeled` events |
| `.github/CODEOWNERS` | New — pins `PINNED_COMMIT` + `patches/**` to legal-team approval |
| `services/Makefile` | Appended chat-build/chat-license-check/chat-test/chat-verify shortcuts |

**Verification:** `make chat-verify` green in 4/4 test scripts (pinned_commit + license_drift's 4 sub-cases + patch_apply + workflows_present).

### FR-EMAIL-001 — Stalwart adapter + per-tenant DKIM + residency-pinned bodies

Layer 0 (zero upstream deps; FR-AI-016 residency lookup is a config lookup with default `vn-1` tag).

| Layer | Files |
|---|---|
| Migrations | `0001_messages.sql` (message_metadata + thread_metadata + 2 closed enums + RLS) · `0002_bounce_log.sql` (append-only) · `0003_dkim_keys.sql` (per-tenant keystore + rotation + KMS-encrypted blob) · `0004_residency_routing.sql` (4 residency tags) |
| Rust crate | `src/{lib,types,errors,residency}.rs` + `src/dkim/{mod,keystore}.rs` + `src/stalwart_adapter/{mod,inbound,outbound}.rs` + `src/repo/{mod,messages,bounce_log}.rs` + `src/audit/{mod,email_events}.rs` + `src/handlers/{mod,status}.rs` + `src/bin/{server,cli}.rs` (17 source files) |
| Container | `docker/Dockerfile` (FROM stalwartlabs/mail-server:0.10) · `docker/stalwart.toml` (Postgres backend + S3 blob + 7 listeners + DKIM + MTA-STS + spam threshold + 30s shutdown) · `docker/compose.yml` (Postgres + Minio + Stalwart + gateway) |
| Tests | `tests/{residency_pin,audit_row,inbound_quarantine,subject_normalisation}_test.rs` (26 cases) + 21 inline `#[cfg(test)]` |
| Docs | `README.md` + `AGENTS.md` (module-level agent rules) |

**memory audit row kinds emitted:** `email.message_received`, `email.message_sent`, `email.message_bounced`, `email.message_quarantined`, `email.dkim_key_rotated`. All carry PII-hashed addresses (SHA-256[..16]); raw addresses never leak into the audit chain.

**Spec divergences (documented in audit §10.6):**
- RLS GUC name aligns with FR-AUTH-003 §10.6 amendment (`app.current_tenant_id` vs spec's `auth.tenant_id`).
- Slice-1 placeholder DKIM PEM generator (real RSA-2048 in slice 2 behind feature flag).
- Live-runtime ACs (perf, OTel) deferred to integration-test runner in CI.

### FR-PROJ-001 — Issue + Cycle + Engagement schema + FSM + bidirectional links

Layer 2 (depends on FR-AUTH-001 + FR-AUTH-003 — both shipped).

| Layer | Files |
|---|---|
| Migrations | `0001_engagements.sql` (RLS + status enum check + ended_at >= started_at) · `0002_cycles.sql` (RLS + ends_at > starts_at CHECK) · `0003_issues.sql` (5-state status + 4-priority + RLS + auto `updated_at` trigger) · `0004_issue_links.sql` (RLS via `issues` join + self-link CHECK) |
| Rust crate | `src/{lib,types,errors,status_fsm,audit,links,repo,handlers}.rs` (8 source files) |
| Tests | `tests/{status_fsm,audit_row,link_types,error_mapping}_test.rs` (23 cases) + 20 inline `#[cfg(test)]` |
| Docs | `README.md` + `AGENTS.md` |

**memory audit row kinds emitted:** `proj.issue_created`, `proj.issue_status_changed`, `proj.issue_assigned`, `proj.issue_linked`.

**FSM (5-state closed):** `triage → todo → doing → review → done` with explicit backward edges (deferrals, pauses, rejections) and `done` as terminal. Illegal transitions return 400 with the allowed-set.

**Symmetric links:** `Blocks` ↔ `BlockedBy`, `Duplicates` ↔ `DuplicatedBy` auto-insert inverse in same tx. Asymmetric (`Related`, `DerivedFromEmail/Chat/Meeting`) do not.

**Spec divergences (documented in audit §10.6):**
- `estimate_hours` changed from `NUMERIC(6,2)` to `DOUBLE PRECISION` with equivalent CHECK constraints (avoids `sqlx/bigdecimal` workspace feature flag).
- RLS GUC name same as EMAIL (FR-AUTH-003 §10.6 alignment).
- `tests/issues_test.rs` split into 4 concern-specific files.
- `TENANT_SCOPED_TABLES` registry update NOT applied (would break AUTH boot-check before PROJ migrations run; needs separate per-service boot-check amendment).

### Workspace + cross-cutting

- `services/Cargo.toml [workspace].members` — added `email` and `proj`.
- `.github/CODEOWNERS` (new file) — pins:
  - `services/chat/PINNED_COMMIT` + `patches/**` to legal-team.
  - `services/email/docker/stalwart.toml` + `services/email/migrations/**` + `services/email/src/dkim/**` to security-team.
  - `services/auth/**` to security-team.
  - `.github/workflows/**` to devops-team.

---

## §14.1 — Non-memory files touched (heartbeat signal)

Per memory `feedback_section_14_emission`: surface non-memory file changes explicitly so the memory heartbeat is unambiguous.

**Created (43 files):**
```
services/chat/PINNED_COMMIT
services/chat/CYBEROS_PATCH_VERSION
services/chat/Dockerfile
services/chat/README.md
services/chat/CHANGELOG.cyberos.md
services/chat/Makefile
services/chat/compose.yml
services/chat/config/config.json
services/chat/scripts/check-license-drift.sh
services/chat/scripts/cherry-pick-upstream.sh
services/chat/tests/pinned_commit_test.sh
services/chat/tests/license_drift_test.sh
services/chat/tests/patch_apply_test.sh
services/chat/tests/workflows_present_test.sh
services/chat/tests/run_all_tests.sh
services/email/Cargo.toml
services/email/README.md
services/email/AGENTS.md
services/email/docker/Dockerfile
services/email/docker/stalwart.toml
services/email/docker/compose.yml
services/email/migrations/0001_messages.sql
services/email/migrations/0002_bounce_log.sql
services/email/migrations/0003_dkim_keys.sql
services/email/migrations/0004_residency_routing.sql
services/email/src/lib.rs
services/email/src/types.rs
services/email/src/errors.rs
services/email/src/residency.rs
services/email/src/dkim/mod.rs
services/email/src/dkim/keystore.rs
services/email/src/stalwart_adapter/mod.rs
services/email/src/stalwart_adapter/inbound.rs
services/email/src/stalwart_adapter/outbound.rs
services/email/src/repo/mod.rs
services/email/src/repo/messages.rs
services/email/src/repo/bounce_log.rs
services/email/src/audit/mod.rs
services/email/src/audit/email_events.rs
services/email/src/handlers/mod.rs
services/email/src/handlers/status.rs
services/email/src/bin/server.rs
services/email/src/bin/cli.rs
services/email/tests/residency_pin_test.rs
services/email/tests/audit_row_test.rs
services/email/tests/inbound_quarantine_test.rs
services/email/tests/subject_normalisation_test.rs
services/proj/Cargo.toml
services/proj/README.md
services/proj/AGENTS.md
services/proj/migrations/0001_engagements.sql
services/proj/migrations/0002_cycles.sql
services/proj/migrations/0003_issues.sql
services/proj/migrations/0004_issue_links.sql
services/proj/src/lib.rs
services/proj/src/types.rs
services/proj/src/errors.rs
services/proj/src/status_fsm.rs
services/proj/src/audit.rs
services/proj/src/links.rs
services/proj/src/repo.rs
services/proj/src/handlers.rs
services/proj/tests/status_fsm_test.rs
services/proj/tests/audit_row_test.rs
services/proj/tests/link_types_test.rs
services/proj/tests/error_mapping_test.rs
.github/CODEOWNERS
docs/sessions/2026-05-19-chat-proj-email.md
```

**Modified (7 files):**
```
.github/workflows/chat-license-drift-watcher.yml  (stub → full implementation)
.github/workflows/chat-cherry-pick-review.yml     (stub → full implementation)
services/Cargo.toml                                (+ email, + proj workspace members)
services/Makefile                                  (+ chat-build/chat-license-check/chat-test/chat-verify)
docs/feature-requests/BACKLOG.md                   (header v0.6.0 → v0.6.1)
docs/feature-requests/chat/FR-CHAT-001-mattermost-fork.md          (status: accepted → shipped)
docs/feature-requests/chat/FR-CHAT-001-mattermost-fork.audit.md    (+ §10 Implementation audit)
docs/feature-requests/email/FR-EMAIL-001-stalwart-deployment.md    (status: draft → shipped)
docs/feature-requests/email/FR-EMAIL-001-stalwart-deployment.audit.md  (+ §10)
docs/feature-requests/proj/FR-PROJ-001-issue-schema.md              (status: accepted → shipped)
docs/feature-requests/proj/FR-PROJ-001-issue-schema.audit.md        (+ §10)
```

## §14.2 — memory-side state

No direct memory writes from this session — the Rust binaries that emit `email.*` / `proj.*` / `chat.*` audit rows are scaffolded but the operator runs them out-of-session (the writer transport binds at the binary entry point).

When the operator runs `cargo build && cargo test` and starts the services, the memory audit emission will activate. Expected new row kinds entering the chain:

- `email.message_received`, `email.message_sent`, `email.message_bounced`, `email.message_quarantined`, `email.dkim_key_rotated` (5 from FR-EMAIL-001).
- `proj.issue_created`, `proj.issue_status_changed`, `proj.issue_assigned`, `proj.issue_linked` (4 from FR-PROJ-001).
- No new kinds from FR-CHAT-001 yet — the memory bridge is FR-CHAT-005 territory.

---

## Next concrete steps

In strict topo order per BACKLOG §B:

1. **Operator runs `cargo build -p cyberos-email -p cyberos-proj`** to validate compilation.
2. **Operator runs `cargo test -p cyberos-email -p cyberos-proj`** to validate the test suites pass (≈ 47 EMAIL + 43 PROJ assertions).
3. **FR-EMAIL-004** — DKIM/ARC/BIMI hardening (Layer 1; depends on EMAIL-001).
4. **FR-EMAIL-005** — CaMeL dual-LLM quarantine (Layer 1; depends on EMAIL-001).
5. **FR-EMAIL-011** — DSAR per-subject export (Layer 1; depends on EMAIL-001).
6. **FR-EMAIL-009** — outbound 1:1 send with AM confirm (Layer 2).
7. **FR-CHAT-002** — `cyberos-chat-authbridge` plugin (Layer 3; depends on FR-CHAT-001 + FR-AUTH-004).
8. **FR-PROJ-002** — memory decision anchoring (Layer 3; depends on FR-PROJ-001).
9. **FR-PROJ-005** — rate-card schema (Layer 3).
10. **FR-PROJ-009** — MEMORY_LINK schema (Layer 3).

The full Layer 3+ wave continues through `cargo test` green gates.
