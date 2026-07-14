//! The alert-routing orchestration (TASK-OBS-007). Ties triage -> decide -> deliver with the §1 #11
//! fallback chain (CHAT fails -> PagerDuty; PagerDuty fails -> last-resort CHAT), then audits. Generic
//! over the client traits, so the whole flow is testable without a network.

use crate::alertmanager_webhook::Alert;
use crate::audit::{self, AuditSink};
use crate::notify::{ChatClient, PagerDutyClient};
use crate::route::{decide, Route};
use crate::triage::{Triage, TriageClient};

/// The result of routing one alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteOutcome {
    /// Where the alert was actually delivered.
    pub delivered: Route,
    /// Whether every intended delivery succeeded (false means a fallback fired or a leg failed).
    pub fully_delivered: bool,
}

/// Route one alert end to end. A `triage` error is absorbed as confidence 0 (PagerDuty). Delivery
/// honours the §1 #11 fallback chain so the alert always lands somewhere. Emits the `obs.alert_triaged`
/// audit row with the route actually taken.
pub async fn route_alert<T, C, P>(
    triage_client: &T,
    chat: &C,
    pagerduty: &P,
    sink: &dyn AuditSink,
    alert: &Alert,
    runbook_allowlist: &[String],
    request_id: &str,
) -> RouteOutcome
where
    T: TriageClient,
    C: ChatClient,
    P: PagerDutyClient,
{
    let mut triage = triage_client
        .triage(alert)
        .await
        .unwrap_or_else(|_| Triage::failed());
    // OBS-007 P2: never surface an unverified or fabricated runbook link. Keep the suggested runbook only
    // when it is an exactly allowlisted KB runbook URL; otherwise drop it (CHAT shows "Runbook: none").
    // The alert still routes and pages regardless.
    triage.suggested_runbook =
        crate::runbook::sanitize_runbook(triage.suggested_runbook.as_deref(), runbook_allowlist);
    let intended = decide(alert.severity, triage.confidence);
    let outcome = deliver(chat, pagerduty, alert, &triage, intended, request_id).await;

    sink.emit(&audit::alert_triaged(
        alert,
        triage.confidence,
        outcome.delivered,
        triage.suggested_runbook.as_deref(),
        request_id,
    ));
    outcome
}

async fn deliver<C, P>(
    chat: &C,
    pagerduty: &P,
    alert: &Alert,
    triage: &Triage,
    intended: Route,
    request_id: &str,
) -> RouteOutcome
where
    C: ChatClient,
    P: PagerDutyClient,
{
    match intended {
        Route::Both => {
            // sev-1 - page both; deliver to each independently so one failing does not block the other.
            let chat_ok = chat.post(alert, triage, request_id).await.is_ok();
            let pd_ok = pagerduty.trigger(alert, triage, request_id).await.is_ok();
            RouteOutcome {
                delivered: delivered_route(chat_ok, pd_ok),
                fully_delivered: chat_ok && pd_ok,
            }
        }
        Route::Chat => {
            if chat.post(alert, triage, request_id).await.is_ok() {
                RouteOutcome {
                    delivered: Route::Chat,
                    fully_delivered: true,
                }
            } else {
                // §1 #11 - CHAT failed -> PagerDuty fallback. A fallback is never "fully delivered".
                let _pd_ok = pagerduty.trigger(alert, triage, request_id).await.is_ok();
                RouteOutcome {
                    delivered: Route::PagerDuty,
                    fully_delivered: false,
                }
            }
        }
        Route::PagerDuty => {
            if pagerduty.trigger(alert, triage, request_id).await.is_ok() {
                RouteOutcome {
                    delivered: Route::PagerDuty,
                    fully_delivered: true,
                }
            } else {
                // §1 #11 - PagerDuty failed -> last-resort CHAT. A fallback is never "fully delivered".
                let _chat_ok = chat.post(alert, triage, request_id).await.is_ok();
                RouteOutcome {
                    delivered: Route::Chat,
                    fully_delivered: false,
                }
            }
        }
    }
}

/// The route to record when `Both` was intended: which legs actually landed.
fn delivered_route(chat_ok: bool, pd_ok: bool) -> Route {
    match (chat_ok, pd_ok) {
        (true, false) => Route::Chat,
        (false, true) => Route::PagerDuty,
        // both landed, or (worst case) neither did - either way the intent was both channels.
        _ => Route::Both,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alertmanager_webhook::AlertStatus;
    use crate::audit::RecordingSink;
    use crate::notify::NotifyError;
    use crate::severity::Severity;
    use crate::triage::TriageError;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct FixedTriage {
        confidence: f64,
        fail: bool,
    }
    impl TriageClient for FixedTriage {
        async fn triage(&self, _alert: &Alert) -> Result<Triage, TriageError> {
            if self.fail {
                return Err(TriageError::Timeout);
            }
            Ok(Triage {
                confidence: self.confidence,
                summary: "s".into(),
                suggested_runbook: Some("kb/r".into()),
                suspected_cause: "c".into(),
            })
        }
    }

    #[derive(Default)]
    struct CountingChat {
        calls: AtomicU32,
        fail: bool,
    }
    impl ChatClient for CountingChat {
        async fn post(&self, _a: &Alert, _t: &Triage, _r: &str) -> Result<(), NotifyError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.fail {
                Err(NotifyError("chat down".into()))
            } else {
                Ok(())
            }
        }
    }

    #[derive(Default)]
    struct CountingPd {
        calls: AtomicU32,
        fail: bool,
    }
    impl PagerDutyClient for CountingPd {
        async fn trigger(&self, _a: &Alert, _t: &Triage, _r: &str) -> Result<(), NotifyError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.fail {
                Err(NotifyError("pd down".into()))
            } else {
                Ok(())
            }
        }
    }

    fn alert(sev: Severity) -> Alert {
        Alert {
            name: "A".into(),
            severity: sev,
            status: AlertStatus::Firing,
            fingerprint: "fp".into(),
            trace_id: Some("t".into()),
            summary: None,
        }
    }

    #[tokio::test]
    async fn confident_non_sev1_goes_to_chat_and_audits() {
        let (t, c, p, s) = (
            FixedTriage {
                confidence: 0.9,
                fail: false,
            },
            CountingChat::default(),
            CountingPd::default(),
            RecordingSink::default(),
        );
        let out = route_alert(&t, &c, &p, &s, &alert(Severity::Sev2), &[], "req").await;
        assert_eq!(
            out,
            RouteOutcome {
                delivered: Route::Chat,
                fully_delivered: true
            }
        );
        assert_eq!(c.calls.load(Ordering::SeqCst), 1);
        assert_eq!(p.calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            s.latest("obs.alert_triaged").unwrap().payload["route"],
            "chat"
        );
    }

    #[tokio::test]
    async fn sev1_pages_both() {
        let (t, c, p, s) = (
            FixedTriage {
                confidence: 0.99,
                fail: false,
            },
            CountingChat::default(),
            CountingPd::default(),
            RecordingSink::default(),
        );
        let out = route_alert(&t, &c, &p, &s, &alert(Severity::Sev1), &[], "req").await;
        assert_eq!(out.delivered, Route::Both);
        assert!(out.fully_delivered);
        assert_eq!(c.calls.load(Ordering::SeqCst), 1);
        assert_eq!(p.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn triage_failure_pages_pagerduty() {
        let (t, c, p, s) = (
            FixedTriage {
                confidence: 0.0,
                fail: true,
            },
            CountingChat::default(),
            CountingPd::default(),
            RecordingSink::default(),
        );
        let out = route_alert(&t, &c, &p, &s, &alert(Severity::Sev2), &[], "req").await;
        assert_eq!(out.delivered, Route::PagerDuty);
        assert_eq!(p.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn chat_failure_falls_back_to_pagerduty() {
        let (t, c, p, s) = (
            FixedTriage {
                confidence: 0.9,
                fail: false,
            },
            CountingChat {
                fail: true,
                ..Default::default()
            },
            CountingPd::default(),
            RecordingSink::default(),
        );
        let out = route_alert(&t, &c, &p, &s, &alert(Severity::Sev2), &[], "req").await;
        assert_eq!(out.delivered, Route::PagerDuty); // §1 #11 fallback
        assert_eq!(c.calls.load(Ordering::SeqCst), 1);
        assert_eq!(p.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn pagerduty_failure_last_resorts_to_chat() {
        let (t, c, p, s) = (
            FixedTriage {
                confidence: 0.0,
                fail: false,
            },
            CountingChat::default(),
            CountingPd {
                fail: true,
                ..Default::default()
            },
            RecordingSink::default(),
        );
        let out = route_alert(&t, &c, &p, &s, &alert(Severity::Sev3), &[], "req").await;
        assert_eq!(out.delivered, Route::Chat); // §1 #11 last resort
        assert_eq!(p.calls.load(Ordering::SeqCst), 1);
        assert_eq!(c.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn runbook_is_dropped_unless_allowlisted() {
        // FixedTriage suggests "kb/r". With an empty allowlist that runbook is unverified, so it is blanked
        // in the audit (and therefore in the CHAT post) - OBS-007 P2 fail-closed against a fabricated URL.
        let (t, c, p, s) = (
            FixedTriage {
                confidence: 0.9,
                fail: false,
            },
            CountingChat::default(),
            CountingPd::default(),
            RecordingSink::default(),
        );
        route_alert(&t, &c, &p, &s, &alert(Severity::Sev2), &[], "req").await;
        assert!(
            s.latest("obs.alert_triaged").unwrap().payload["suggested_runbook"].is_null(),
            "a non-allowlisted runbook is dropped (fail-closed)"
        );

        // The same runbook, now exactly on the allowlist -> preserved.
        let s2 = RecordingSink::default();
        let allow = vec!["kb/r".to_string()];
        route_alert(&t, &c, &p, &s2, &alert(Severity::Sev2), &allow, "req").await;
        assert_eq!(
            s2.latest("obs.alert_triaged").unwrap().payload["suggested_runbook"],
            "kb/r",
            "an exactly allowlisted runbook is kept"
        );
    }
}
