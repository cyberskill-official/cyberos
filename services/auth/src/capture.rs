//! TASK-MEMORY-122 §1 #3 — AUTH's capture surface (the per-module `capture.rs`, DEC-2714).
//!
//! Thin typed helpers that translate AUTH's domain events into TASK-MEMORY-121 interaction-events and record
//! them through the shared [`cyberos_capture::Capturer`]. They sit next to the existing
//! `memory_bridge::emit_token_issued` / `emit_token_failed` calls in `handlers.rs` and reuse the SAME audit
//! pool (auth + memory share a Postgres deployment, so AUTH's `state.pg` IS the brain audit DB — no new
//! connection, §1 #3).
//!
//! Everything here is best-effort and OFF by default:
//!   * Each helper takes `Option<&Capturer>`. When `None` — the default, because `CAPTURE_ENABLED` is unset
//!     and so `AppState.capturer` is `None` — the helper returns immediately and does nothing. Deploying
//!     this during a live team test changes nothing about sign-in.
//!   * When a capturer is present, the helper builds the event and calls `capture_metered`, which routes
//!     through TASK-MEMORY-121 `emit()` (consent-gated: a subject who has not acknowledged the notice yields
//!     a `Skipped` outcome and zero rows) and swallows any error. A capture failure NEVER fails or delays
//!     token issuance — the call sites already treat their audit writes this way.
//!
//! No raw credentials, emails, or IPs are ever placed in an event; AUTH passes the same privacy-safe
//! `source_ip_hash16` digest it already computes for the audit row.

use cyberos_capture::{
    Capturer, ContentRef, EventClass, InteractionEvent, Module, SourceChannel, TargetRef,
};
use uuid::Uuid;

/// Map a request's `User-Agent` (when present) to the closed `SourceChannel`. The CDS web console and the
/// browser send a normal browser UA; the desktop app and CLI/API clients identify themselves. Unknown /
/// absent falls back to `Api` (the failure-mode-inventory default for a legacy client, §10). This is what
/// makes "signed in from the web console" vs "an API client" distinguishable downstream (§1 #13).
pub fn source_channel_from_ua(user_agent: Option<&str>) -> SourceChannel {
    match user_agent {
        Some(ua) => {
            let ua = ua.to_ascii_lowercase();
            if ua.contains("cyberos-desktop") || ua.contains("tauri") {
                SourceChannel::Desktop
            } else if ua.contains("cyberos-cli") || ua.contains("authctl") || ua.contains("curl") {
                SourceChannel::Cli
            } else if ua.contains("mozilla") || ua.contains("webkit") || ua.contains("chrome") {
                SourceChannel::Web
            } else {
                SourceChannel::Api
            }
        }
        None => SourceChannel::Api,
    }
}

/// §1 #3 — `auth.signed_in` on a successful token issue. `target_ref` is the session (the jti); `content_ref`
/// is none; `attributes` carry the auth `method` (`password | oidc | passkey`) and the privacy-safe
/// `source_ip_hash16`. `trace_id` is set when a request trace (traceparent) is in scope, else null (§1 #14).
/// Best-effort + no-op when `cap` is `None`.
pub async fn emit_signed_in(
    cap: Option<&Capturer>,
    tenant: Uuid,
    subject: Uuid,
    jti: &str,
    method: &str,
    source: SourceChannel,
    source_ip_hash16: &str,
    trace_id: Option<String>,
) {
    let Some(cap) = cap else { return };
    let mut b = InteractionEvent::builder(Module::Auth, "auth.signed_in", EventClass::Auth)
        .tenant(tenant)
        .subject(subject)
        .occurred_now()
        .target(TargetRef::Session {
            id: jti.to_string(),
        })
        .content(ContentRef::None)
        .source(source)
        .attribute("method", serde_json::Value::String(method.to_string()))
        .attribute(
            "source_ip_hash16",
            serde_json::Value::String(source_ip_hash16.to_string()),
        );
    if let Some(t) = trace_id {
        b = b.trace_id(t);
    }
    match b.build() {
        Ok(ev) => cap.capture_metered(&ev).await,
        Err(e) => tracing::warn!(error = %e, "auth.signed_in event build failed (best-effort)"),
    }
}

/// §1 #3 — `auth.sign_in_failed` on a failed attempt. `subject_id` is `None` when the email did not resolve
/// (no person to attribute — mirrors `emit_token_failed`), `Some` when a known subject failed (e.g.
/// suspended/wrong-password). `content_ref` none; `attributes` carry the `reason` and `source_ip_hash16`.
///
/// Note: the consent gate only applies to events that carry a subject. A failed attempt with `subject =
/// None` is a system-actor event and is recorded even before the person acknowledges — but it carries no
/// personal content (only a reason + a per-day IP digest), and a failed attempt for a KNOWN subject is
/// still consent-gated. Best-effort + no-op when `cap` is `None`.
pub async fn emit_sign_in_failed(
    cap: Option<&Capturer>,
    tenant: Uuid,
    subject: Option<Uuid>,
    reason: &str,
    source: SourceChannel,
    source_ip_hash16: &str,
    trace_id: Option<String>,
) {
    let Some(cap) = cap else { return };
    let mut b = InteractionEvent::builder(Module::Auth, "auth.sign_in_failed", EventClass::Auth)
        .tenant(tenant)
        .occurred_now()
        .target(TargetRef::None)
        .content(ContentRef::None)
        .source(source)
        .attribute("reason", serde_json::Value::String(reason.to_string()))
        .attribute(
            "source_ip_hash16",
            serde_json::Value::String(source_ip_hash16.to_string()),
        );
    if let Some(s) = subject {
        b = b.subject(s);
    } else {
        b = b.system_actor();
    }
    if let Some(t) = trace_id {
        b = b.trace_id(t);
    }
    match b.build() {
        Ok(ev) => cap.capture_metered(&ev).await,
        Err(e) => {
            tracing::warn!(error = %e, "auth.sign_in_failed event build failed (best-effort)")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_channel_maps_user_agents() {
        assert_eq!(
            source_channel_from_ua(Some("Mozilla/5.0 (X11) Chrome/120")),
            SourceChannel::Web
        );
        assert_eq!(
            source_channel_from_ua(Some("cyberos-desktop/0.1 (tauri)")),
            SourceChannel::Desktop
        );
        assert_eq!(
            source_channel_from_ua(Some("cyberos-authctl/0.1")),
            SourceChannel::Cli
        );
        assert_eq!(
            source_channel_from_ua(Some("some-bot/1.0")),
            SourceChannel::Api
        );
        assert_eq!(source_channel_from_ua(None), SourceChannel::Api);
    }

    // The helpers are no-ops when no capturer is present (the default-OFF state). These must not panic and
    // must do nothing — proving that with CAPTURE_ENABLED unset, sign-in is completely unaffected.
    #[tokio::test]
    async fn helpers_are_noops_without_a_capturer() {
        emit_signed_in(
            None,
            Uuid::nil(),
            Uuid::nil(),
            "jti-1",
            "password",
            SourceChannel::Web,
            "deadbeefdeadbeef",
            None,
        )
        .await;
        emit_sign_in_failed(
            None,
            Uuid::nil(),
            None,
            "invalid_credentials",
            SourceChannel::Web,
            "deadbeefdeadbeef",
            None,
        )
        .await;
    }
}
