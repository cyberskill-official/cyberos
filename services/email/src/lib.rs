//! `cyberos-email` — FR-EMAIL-001 slice 1.
//!
//! Mirror layer between Stalwart (the canonical mail server) and CyberOS
//! cluster state. Responsibilities:
//!
//!   * Receive webhook events from Stalwart on inbound delivery + persist
//!     metadata into `message_metadata` (S3 stores the body, encrypted via
//!     KMS, residency-pinned per tenant).
//!   * Look up per-tenant residency before writing the body to the
//!     residency-pinned S3 bucket.
//!   * Manage per-tenant DKIM keys + rotation history.
//!   * Emit 5 `email.*` memory audit row kinds (FR-EMAIL-001 §1 #13).
//!   * Expose REST health + per-message-status + cursored list handlers.
//!
//! The Stalwart server itself runs as a separate container (see
//! `docker/Dockerfile` + `docker/stalwart.toml`).
//!
//! ### Scope at slice 1
//!
//! What lands:
//!   - Migrations 0001–0004 — message + thread metadata, bounce log, DKIM
//!     keystore, residency routing.
//!   - Types module — `EmailMessage`, `EmailThread`, `BounceEvent`,
//!     `DkimKey`, `MessageStatus`, `MessageDirection`, `BounceKind`.
//!   - Stalwart inbound adapter (mock-mode for slice 1; the real Stalwart
//!     webhook plumbing arrives in FR-EMAIL-002 alongside the JWT bridge).
//!   - Residency resolver (FR-AI-016 contract; lookup against
//!     `tenant_residency`).
//!   - DKIM keystore generation + rotation.
//!   - Append-only writers for message_metadata + bounce_log.
//!   - 5 `email.*` memory audit row builders.
//!   - Health + per-message status + list handlers.
//!   - `cyberos-email-cli provision` slice-1 user-provisioning entry.
//!
//! What is intentionally deferred (per the FR's §9 + `disallowed_tools`):
//!   - Real Stalwart container wiring + JMAP/IMAP/SMTP listeners (FR-EMAIL-002).
//!   - CaMeL dual-LLM quarantine (FR-EMAIL-005).
//!   - Shared-inbox UX (FR-EMAIL-003).
//!   - DKIM/ARC/BIMI hardening (FR-EMAIL-004).
//!   - Convert-to-issue (FR-EMAIL-007).
//!   - Bulk-send approval (FR-EMAIL-010).
//!   - DSAR per-subject export (FR-EMAIL-011).
//!
//! ### RLS pattern
//!
//! All tables follow FR-AUTH-003 §10.6 — the GUC `app.current_tenant_id`
//! drives the tenant-isolation policy; the nil-UUID escape is root-tenant
//! only. The FR-EMAIL-001 spec uses `auth.tenant_id` in §1 #10; this is a
//! documented spec divergence in the audit dossier §10.6.

// Style-class clippy + hygiene lints deferred to the FR-EMAIL-NNN-clippy-
// style-cleanup hygiene wave — see services/auth/src/lib.rs for the rationale
// (CI's `RUSTFLAGS: -D warnings` escalates idiom-modernisation hints to
// blocking errors on every PR; the diffs are style nits, not correctness bugs).
#![allow(missing_docs)]
#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::doc_overindented_list_items)]
#![allow(clippy::let_and_return)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::useless_format)]
#![allow(clippy::manual_pattern_char_comparison)]
#![allow(clippy::double_ended_iterator_last)]
#![allow(dead_code)]

pub mod audit;
pub mod dkim;
pub mod errors;
pub mod handlers;
pub mod repo;
pub mod residency;
pub mod stalwart_adapter;
pub mod types;

pub use errors::EmailError;
pub use types::*;
