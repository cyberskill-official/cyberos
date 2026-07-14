# EMAIL module — agent instructions

This file is read by Claude / Cursor / Codex agents working inside
`services/email/`. It is supplementary to the root `AGENTS.md` (the
CyberOS Layer-1 Memory Protocol); it does NOT override the protocol's
§0.1 precedence rules.

---

## §1 — Read order for any change

1. The FR being implemented — `docs/tasks/email/FR-EMAIL-<N>-<slug>.md`.
2. Its audit dossier — same name + `.audit.md`.
3. This file.
4. The root `AGENTS.md` if a protocol-level concern surfaces.

---

## §2 — Hard rules for EMAIL work

- **Bodies live in S3+KMS, not Postgres.** Per TASK-EMAIL-001 §1 DEC-311 and
  the `disallowed_tools` line, message bodies MUST NOT be stored on a
  Postgres-readable row. The metadata mirror in `message_metadata` carries
  `s3_body_key` + `s3_body_kms_key_id` + `body_sha256_hex` — that is the
  ONLY place body provenance lives.

- **Outbound mail MUST be DKIM-signed.** Per TASK-EMAIL-001 §1 #15. Any
  outbound path that bypasses the DKIM verification step is a spec
  violation. The `on_outbound` adapter checks `dkim_keys.status =
  'active'` before submitting; do not add a path that skips the check.

- **Cross-residency writes are fail-closed.** Per §1 #12. The Stalwart
  inbound handler asserts residency match BEFORE the S3 PUT. Adding an
  override (even for "operator can fix later") is forbidden — Decree
  53/2022 + GDPR require this be enforced by code, not by review.

- **Append-only at the SQL-grant layer.** `REVOKE UPDATE, DELETE ON
  message_metadata, bounce_log FROM cyberos_app;` is the line. Any path
  that requires editing an existing row MUST instead write a new row
  with `prior_message_id` set.

- **PII never lands in memory audit.** Raw email addresses, raw subjects,
  raw body text — none of these may appear in an `email.*` audit row.
  The `from_hash16` / `to_hash16` helpers in `src/audit/email_events.rs`
  are the only addressee representation allowed.

---

## §3 — Where to find things

| Concern | Location |
|---|---|
| Stalwart server config | `docker/stalwart.toml` |
| SQL schema | `migrations/0001_messages.sql` etc. |
| Domain types | `src/types.rs` |
| Error type | `src/errors.rs` |
| Residency resolver | `src/residency.rs` |
| DKIM keystore | `src/dkim/keystore.rs` |
| Inbound adapter | `src/stalwart_adapter/inbound.rs` |
| Outbound adapter | `src/stalwart_adapter/outbound.rs` |
| Repo layer | `src/repo/messages.rs`, `src/repo/bounce_log.rs` |
| 5 memory audit row builders | `src/audit/email_events.rs` |
| REST handlers | `src/handlers/status.rs` |
| HTTP server entry | `src/bin/server.rs` |
| Operator CLI | `src/bin/cli.rs` |

---

## §4 — Tests

```bash
# Unit tests — no DB or Stalwart required.
cd services && cargo test -p cyberos-email --lib

# Integration tests (residency_pin, audit_row, inbound_quarantine, subject_normalisation).
cd services && cargo test -p cyberos-email
```

---

## §5 — Spec divergences

Documented in `TASK-EMAIL-001-stalwart-deployment.audit.md` §10.6:

- `auth.tenant_id` (spec §1 #10) → `app.current_tenant_id` (impl,
  aligned with TASK-AUTH-003 §10.6 amendment).
