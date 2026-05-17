---
id: FR-OBS-007
title: "obs-router: Alertmanager → CUO obs.triage-alert@1 skill → CHAT (≥0.70 conf) OR PagerDuty + sev-1 always pages + ack-button + audit"
module: OBS
priority: MUST
status: accepted
verify: T
phase: P0
milestone: P0 · slice 3
slice: 3
owner: Stephen Cheng (CTO)
created: 2026-05-15
shipped: null
brain_chain_hash: null
related_frs: [FR-OBS-001, FR-OBS-003, FR-OBS-005, FR-CUO-101, FR-KB-008]
depends_on: [FR-OBS-002, FR-OBS-003]
blocks: [FR-KB-008]

source_pages:
  - website/docs/modules/obs.html#alert-routing
  - website/docs/modules/cuo.html#obs-triage-skill
source_decisions:
  - DEC-170 (CUO triage with 0.70 confidence floor; below = page on-call)
  - DEC-171 (sev-1 always pages BOTH CHAT + PagerDuty; never trust triage at highest severity)
  - DEC-172 (CHAT post includes ack-button + suggested runbook + trace_id link)
  - DEC-173 (CUO skill failure = safe fallback to PagerDuty; never silent-drop)

language: rust 1.81
service: cyberos/services/obs-router/
new_files:
  - services/obs-router/Cargo.toml
  - services/obs-router/src/main.rs
  - services/obs-router/src/alertmanager_webhook.rs
  - services/obs-router/src/cuo_triage.rs
  - services/obs-router/src/chat_post.rs
  - services/obs-router/src/pagerduty.rs
  - services/obs-router/src/severity.rs
  - services/obs-router/src/ack_handler.rs
  - services/obs-router/tests/triage_test.rs
  - services/obs-router/tests/pagerduty_fallback_test.rs
  - services/obs-router/tests/sev1_always_pages_test.rs
  - services/obs-router/tests/cuo_skill_failure_test.rs
  - skills/obs.triage-alert/SKILL.md
  - skills/obs.triage-alert/runbooks-corpus/.keep
modified_files:
  - deploy/obs/alertmanager-config.yaml                   # webhook → obs-router:7777
allowed_tools:
  - file_read: services/obs-router/**, skills/obs.triage-alert/**
  - file_write: services/obs-router/**, skills/obs.triage-alert/**
  - bash: cd services/obs-router && cargo test
disallowed_tools:
  - auto-resolve a sev-1 alert without human confirmation (per §1 #5)
  - bypass PagerDuty fallback on CUO failure (per §1 #11)
  - skip BRAIN audit row (per §1 #6)
  - silent-drop alert (per §1 #11 — every alert MUST route somewhere)

effort_hours: 10
sub_tasks:
  - "0.5h: Cargo.toml + main.rs"
  - "0.5h: alertmanager_webhook.rs — accept Alertmanager v2 webhook payload"
  - "1.0h: severity.rs — derive sev-1..sev-4 from alert labels (`severity: P1` etc.)"
  - "1.0h: cuo_triage.rs — invoke `obs.triage-alert@1` skill with 5s timeout"
  - "1.0h: chat_post.rs — post to CHAT channel with ack-button + runbook link"
  - "0.5h: pagerduty.rs — trigger PagerDuty incident via Events API v2"
  - "1.0h: route decision logic (sev-1 → both; conf >= 0.70 → CHAT; else PD)"
  - "0.5h: ack_handler.rs — accept ack from CHAT button; close PagerDuty if dual-routed"
  - "0.5h: trace_id preservation from alert labels"
  - "0.5h: BRAIN audit row + OTel metrics"
  - "1.0h: skills/obs.triage-alert/SKILL.md (markdown skill with RAG over runbooks)"
  - "1.5h: Tests — high-conf-CHAT + low-conf-PD + sev-1-both + CUO-failure-fallback + ack"
risk_if_skipped: "Every alert pages a human regardless of severity. On-call gets noise overload (typical SaaS at 50 tenants generates ~20 alerts/day; without triage, that's 20 pages/day). The cost-of-everything gate's whole point — let CUO triage low-stakes alerts — is lost. Without sev-1-always-pages, an over-confident triage might silence a real incident. Without ack-button, ops correlation between CHAT discussion + alert state breaks."
---

## §1 — Description (BCP-14 normative)

A Rust HTTP service `obs-router` **MUST** accept Alertmanager webhook fires and route them through CUO's `obs.triage-alert@1` skill:

1. **MUST** accept Alertmanager v2 webhook on `:7777/alert` with payload schema per Alertmanager docs.
2. **MUST** invoke CUO `obs.triage-alert@1` skill with the alert payload as input. The skill returns `{ confidence: f64, summary: String, suggested_runbook: Option<RunbookRef>, suspected_cause: String }`.
3. **MUST** route based on (severity, confidence):
    - sev-1: route to BOTH CHAT and PagerDuty regardless of confidence.
    - sev-2..sev-4 + confidence ≥ 0.70: post triage summary to CHAT (`#oncall` channel).
    - sev-2..sev-4 + confidence < 0.70: trigger PagerDuty.
4. **MUST** include in CHAT post:
    - Alert name + severity (visual badge).
    - CUO triage summary (the skill's output).
    - Suspected cause.
    - Suggested runbook (link to KB if present).
    - Trace_id link (jumps to Tempo).
    - Ack button (sends POST to `obs-router:7777/ack/<alert_id>`).
    - Escalate-to-PagerDuty button (sends POST to escalate).
5. **MUST** route sev-1 alerts to BOTH CHAT and PagerDuty (no triage trust at sev-1 — DEC-171).
6. **MUST** emit BRAIN audit row `obs.alert_triaged` per alert with payload: `alert_name`, `severity`, `cuo_confidence`, `route` (chat | pagerduty | both), `suggested_runbook`, `trace_id`, `request_id`.
7. **MUST** preserve `trace_id` from alert labels (`trace_id` exemplar from FR-OBS-005). The CHAT post + audit row carry the trace_id; investigators click → jump to Tempo.
8. **MUST** complete triage + route within 10s p95. Slow triage means the alert is invisible to ops during the lag.
9. **MUST** handle CUO skill timeout (5s budget): on timeout, route as if confidence == 0 (PagerDuty fallback). The metric `obs_router_cuo_timeouts_total` increments; sev-2 alarm if rate > 5%.
10. **MUST** handle ack from CHAT: when operator clicks ack-button, POST to `/ack/<alert_id>` MUST (a) update the CHAT post to show "acked by @user at <time>", (b) close the PagerDuty incident if dual-routed (sev-1), (c) emit `obs.alert_acked` BRAIN row.
11. **MUST NOT** silent-drop any alert. Every alert MUST route somewhere — CUO failure → PagerDuty fallback; CHAT failure → PagerDuty fallback; PagerDuty failure → log sev-1 + try CHAT as last resort.
12. **MUST** support deduplication: alerts with identical `alert_fingerprint` arriving within 5 minutes are deduplicated to a single CHAT post (with a counter "fired N times in last 5m"). PagerDuty has its own dedup; we don't double-up.
13. **MUST** authenticate Alertmanager via shared secret (`X-CyberOS-Webhook-Secret` header). Unauthenticated webhooks → 401.
14. **SHOULD** emit OTel metrics:
    - `obs_router_alerts_received_total{severity}` (counter).
    - `obs_router_alerts_routed_total{route, severity, outcome}` (counter; outcome ∈ ok | chat_failed | pagerduty_failed | cuo_failed | dropped).
    - `obs_triage_confidence` (histogram).
    - `obs_router_triage_latency_ms` (histogram; SLO p95 < 10s).
    - `obs_router_acks_total{ack_source}` (counter).
    - `obs_router_dedup_total` (counter).

---

## §2 — Why this design (rationale for humans)

**Why CUO triage at all?** Alerts are the firehose. CUO with `obs.triage-alert@1` skill (markdown + RAG over KB runbooks) reads the alert, queries similar past incidents, and produces a triage summary. ~80% of alerts have a clear root cause; CUO surfaces it; ops gets the answer in CHAT instead of getting paged at 3am.

**Why 0.70 confidence threshold (DEC-170)?** Conservative. Below 0.70, CUO is uncertain enough that page-on-call is the safe choice. Above 0.70, CUO is confident enough that CHAT triage is appropriate. The number is calibrated against the obs.triage-alert@1 skill's calibration curve; future tuning via FR-CUO-104.

**Why sev-1 always pages BOTH (DEC-171)?** Sev-1 = customer-facing outage OR security incident. The cost of missing one is too high to trust triage. Page-AND-CHAT means ops sees both immediately; CHAT post provides context while pager wakes them up.

**Why ack-button + auto-close-PagerDuty (§1 #10)?** When ops acks in CHAT, PagerDuty incident should auto-close — otherwise pager keeps escalating ("acknowledge in PagerDuty too"). The auto-close eliminates the double-action.

**Why dedup at 5m window (§1 #12)?** Many alert rules fire repeatedly while the underlying issue persists (e.g., latency spike for 30 minutes generates 30 fires). 30 CHAT posts is noise. Dedup to 1 post + counter is signal.

**Why never-silent-drop (§1 #11)?** Silent drops are catastrophic — a real incident invisible to ops. Cascade fallback (CUO → PagerDuty → CHAT-as-last-resort) ensures every alert reaches a human.

**Why webhook secret (§1 #13)?** Open webhook ingress lets attackers fake alerts. Operator dashboards become poisonable. Shared secret limits to legitimate Alertmanager.

**Why 10s p95 budget (§1 #8)?** Slow triage = alert invisible to ops. 5s for CUO + 5s for CHAT/PagerDuty post is the budget. Above 10s, ops investigates "why didn't I see the alert?"

**Why escalate-to-PagerDuty button (§1 #4)?** Sometimes triage looks good in CHAT but ops realises it's actually severe. One-click escalation to PagerDuty without re-triaging.

---

## §3 — API contract

```rust
// services/obs-router/src/alertmanager_webhook.rs
#[derive(Deserialize)]
pub struct AlertmanagerWebhook {
    pub version: String,
    pub group_key: String,
    pub status: String,                       // "firing" | "resolved"
    pub receiver: String,
    pub group_labels: HashMap<String, String>,
    pub common_labels: HashMap<String, String>,
    pub alerts: Vec<Alert>,
}

#[derive(Deserialize)]
pub struct Alert {
    pub status: String,
    pub labels: HashMap<String, String>,      // includes severity, alert_name, tenant_id, trace_id
    pub annotations: HashMap<String, String>, // includes runbook_url, summary
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub fingerprint: String,                  // dedup key
}

// services/obs-router/src/cuo_triage.rs
#[derive(Deserialize)]
pub struct TriageResult {
    pub confidence: f64,
    pub summary: String,
    pub suspected_cause: String,
    pub suggested_runbook: Option<RunbookRef>,
}

#[derive(Deserialize)]
pub struct RunbookRef {
    pub kb_article_id: String,
    pub title: String,
    pub url: String,
}

pub async fn invoke_triage_skill(alert: &Alert) -> Result<TriageResult, CuoError> {
    let timeout = Duration::from_secs(5);
    let payload = serde_json::json!({ "alert": alert });
    tokio::time::timeout(timeout, cuo_client::invoke_skill("obs.triage-alert@1", &payload))
        .await
        .map_err(|_| CuoError::Timeout)?
}

// services/obs-router/src/severity.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity { P1, P2, P3, P4 }

pub fn parse_severity(label: &str) -> Severity {
    match label.to_uppercase().as_str() {
        "P1" | "SEV-1" | "CRITICAL" => Severity::P1,
        "P2" | "SEV-2" | "ERROR"    => Severity::P2,
        "P3" | "SEV-3" | "WARNING"  => Severity::P3,
        _                            => Severity::P4,
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Route { Chat, PagerDuty, Both }

pub fn decide_route(severity: Severity, confidence: f64) -> Route {
    if severity == Severity::P1 { return Route::Both; }
    if confidence >= 0.70 { Route::Chat } else { Route::PagerDuty }
}
```

```rust
// services/obs-router/src/main.rs (handler)
pub async fn handle_alert(payload: AlertmanagerWebhook) -> Result<(), RouterError> {
    let alert = &payload.alerts[0];   // assume one per webhook (Alertmanager grouping)
    let severity = severity::parse_severity(alert.labels.get("severity").map(String::as_str).unwrap_or(""));

    // §1 #12 dedup
    if dedup::seen_within(alert.fingerprint, Duration::from_secs(300)) {
        metrics::dedup();
        return update_existing_chat_post_counter(alert).await;
    }

    let triage = match cuo_triage::invoke_triage_skill(alert).await {
        Ok(t) => t,
        Err(_) => TriageResult { confidence: 0.0, summary: "CUO unavailable; review alert manually.".into(),
                                  suspected_cause: "unknown".into(), suggested_runbook: None },
    };

    let route = severity::decide_route(severity, triage.confidence);

    let result = match route {
        Route::Chat => chat_post::post(alert, &triage, severity).await
            .map(|_| ())
            .or_else(|_| pagerduty::trigger(alert, &triage, severity)),
        Route::PagerDuty => pagerduty::trigger(alert, &triage, severity).await,
        Route::Both => {
            let chat_result = chat_post::post(alert, &triage, severity).await;
            let pd_result = pagerduty::trigger(alert, &triage, severity).await;
            chat_result.or(pd_result)
        }
    };

    if result.is_err() {
        // §1 #11 last-resort fallback
        let _ = chat_post::post_emergency(alert).await;
    }

    brain::emit(canonical::alert_triaged(alert, severity, triage.confidence, route)).await?;
    metrics::routed(route, severity, result.is_ok());
    Ok(())
}

// services/obs-router/src/chat_post.rs
pub async fn post(alert: &Alert, triage: &TriageResult, severity: Severity) -> Result<MessageId, ChatError> {
    let trace_id = alert.labels.get("trace_id").cloned();
    let blocks = json!([
        { "type": "header", "text": { "type": "plain_text", "text": format!("[{severity:?}] {}", alert.labels["alertname"]) }},
        { "type": "section", "text": { "type": "mrkdwn", "text": triage.summary }},
        { "type": "section", "text": { "type": "mrkdwn", "text": format!("*Suspected cause:* {}", triage.suspected_cause) }},
        triage.suggested_runbook.as_ref().map(|r| json!({ "type": "section", "text": { "type": "mrkdwn",
            "text": format!("*Runbook:* <{}|{}>", r.url, r.title) }})),
        trace_id.as_ref().map(|t| json!({ "type": "section", "text": { "type": "mrkdwn",
            "text": format!("*Trace:* <https://grafana.cyberos.world/trace/{t}|{t}>") }})),
        { "type": "actions", "elements": [
            { "type": "button", "text": { "type": "plain_text", "text": "Ack" },
              "action_id": "ack", "value": alert.fingerprint.clone() },
            { "type": "button", "text": { "type": "plain_text", "text": "Escalate to PD" },
              "action_id": "escalate", "value": alert.fingerprint.clone() }
        ]}
    ]);
    chat_client::post_blocks("#oncall", blocks).await
}
```

---

## §4 — Acceptance criteria

1. Alertmanager fire → 200 OK from obs-router within 10s p95.
2. High-confidence (≥0.70) sev-2 → CHAT post created in `#oncall` (no PagerDuty).
3. Low-confidence (<0.70) sev-2 → PagerDuty trigger (no CHAT).
4. Sev-1 → BOTH CHAT + PagerDuty regardless of confidence.
5. CHAT post contains: alert name, severity badge, summary, suspected cause, runbook link, trace_id link, ack button, escalate button.
6. CUO timeout → confidence 0 → PagerDuty (graceful fallback).
7. CUO failure (non-timeout) → confidence 0 → PagerDuty.
8. CHAT failure → fallback to PagerDuty.
9. PagerDuty failure → emergency CHAT post; sev-1 log.
10. BRAIN audit row `obs.alert_triaged` emitted per alert.
11. Trace_id preserved end-to-end (CHAT link works).
12. Ack button: clicked → CHAT post updates "acked by @user at <ts>"; PagerDuty incident closed if dual-routed; `obs.alert_acked` row emitted.
13. Escalate button: clicked → PagerDuty triggered post-hoc; row carries `escalated_from_chat: true`.
14. Dedup: same fingerprint within 5min → existing CHAT post counter +1; no new post.
15. Webhook secret enforced: missing → 401; correct → process.
16. p95 triage+route latency < 10s.
17. CUO timeout > 5% sustained → sev-2 alarm.

---

## §5 — Verification

```rust
#[tokio::test]
async fn high_confidence_p2_routes_to_chat_only() {
    let mock_cuo = MockCuo::returning_confidence(0.85);
    let mock_chat = MockChat::start();
    let mock_pd = MockPagerDuty::start();
    let _ = handle_alert(test_webhook(P2, "BrainSearchLatencyHigh")).await.unwrap();
    assert_eq!(mock_chat.posts().len(), 1);
    assert_eq!(mock_pd.incidents().len(), 0);
}

#[tokio::test]
async fn low_confidence_routes_to_pagerduty() {
    let _ = MockCuo::returning_confidence(0.40);
    let mock_pd = MockPagerDuty::start();
    let mock_chat = MockChat::start();
    let _ = handle_alert(test_webhook(P3, "x")).await.unwrap();
    assert_eq!(mock_pd.incidents().len(), 1);
    assert_eq!(mock_chat.posts().len(), 0);
}

#[tokio::test]
async fn sev1_routes_to_both_regardless_of_confidence() {
    let _ = MockCuo::returning_confidence(0.99);
    let mock_chat = MockChat::start();
    let mock_pd = MockPagerDuty::start();
    let _ = handle_alert(test_webhook(P1, "DBdown")).await.unwrap();
    assert_eq!(mock_chat.posts().len(), 1);
    assert_eq!(mock_pd.incidents().len(), 1);
}

#[tokio::test]
async fn cuo_timeout_falls_back_to_pagerduty() {
    let _ = MockCuo::timing_out();
    let mock_pd = MockPagerDuty::start();
    let _ = handle_alert(test_webhook(P2, "x")).await.unwrap();
    assert_eq!(mock_pd.incidents().len(), 1);
    let timeouts: u64 = otel_test_helper::counter_value("obs_router_cuo_timeouts_total", &[]);
    assert_eq!(timeouts, 1);
}

#[tokio::test]
async fn ack_button_closes_pagerduty_for_sev1() {
    let _ = handle_alert(test_webhook(P1, "x")).await.unwrap();
    let alert_id = "test_fingerprint";
    let _ = ack_handler::handle_ack(alert_id, "alice@cyberos.world").await.unwrap();
    let mock_pd = MockPagerDuty::singleton();
    let last_action = mock_pd.last_action(alert_id).await;
    assert_eq!(last_action, "resolved");
    assert!(brain_test_helper::has_row("obs.alert_acked", alert_id));
}

#[tokio::test]
async fn dedup_within_5min_window() {
    let mock_chat = MockChat::start();
    let _ = handle_alert(test_webhook_with_fingerprint("fp1", P3)).await.unwrap();
    let _ = handle_alert(test_webhook_with_fingerprint("fp1", P3)).await.unwrap();
    let _ = handle_alert(test_webhook_with_fingerprint("fp1", P3)).await.unwrap();
    assert_eq!(mock_chat.posts().len(), 1);   // single post
    let counter: u64 = otel_test_helper::counter_value("obs_router_dedup_total", &[]);
    assert_eq!(counter, 2);
}

#[tokio::test]
async fn webhook_secret_enforced() {
    let resp = post_webhook_without_secret(test_webhook(P2, "x")).await;
    assert_eq!(resp.status(), 401);
}
```

---

## §6 — Implementation skeleton

See §3.

```markdown
<!-- skills/obs.triage-alert/SKILL.md -->
---
id: obs.triage-alert
version: 1.0.0
description: Triage Alertmanager alerts using KB runbook RAG; emit confidence + summary + suggested runbook
tools: [kb_search]
output_schema: { confidence: number, summary: string, suspected_cause: string, suggested_runbook: object? }
---

# obs.triage-alert@1

Given an Alertmanager alert, search the KB runbook corpus for similar past incidents,
synthesise a triage summary, and assess confidence.

## Procedure
1. Extract alert_name + labels + annotations.
2. Search KB (`kb_search`) for runbooks matching alert_name or annotations.summary.
3. Read top-3 matched runbooks.
4. Synthesise: "what happened, what to do, which runbook applies."
5. Confidence = function of (runbook match quality, alert clarity).

## Output
JSON object per `output_schema`.
```

---

## §7 — Dependencies

- **FR-OBS-003** — RED metrics drive most alert rules.
- **FR-OBS-005** — trace_id propagation; alerts carry trace_id from exemplars.
- **FR-CUO-101** — CUO Phase-2 LLM cascade. This FR can ship before Phase-2 with a Phase-1 rule-based fallback.
- **FR-KB-008** — KB runbook corpus that the skill RAG-searches.
- Crates: `axum`, `reqwest`, `tokio`, `serde`.

---

## §8 — Example payloads

### Alertmanager webhook

```json
{
  "version": "4",
  "group_key": "...",
  "status": "firing",
  "alerts": [{
    "status": "firing",
    "labels": { "alertname": "BrainSearchLatencyHigh", "severity": "P2", "tenant_id": "550e...", "trace_id": "0af7651916cd43dd8448eb211c80319c" },
    "annotations": { "summary": "p99 > 500ms", "runbook_url": "https://kb.cyberos.world/runbooks/brain-latency" },
    "starts_at": "2026-05-15T14:00:00Z",
    "fingerprint": "abc123"
  }]
}
```

### CHAT post (high-confidence triage)

```text
[P2] BrainSearchLatencyHigh

CUO triage (confidence 0.85): Recent surge in queries against tenant org:cyberskill's KB.
Index size has grown 30% in past hour. p99 spike correlates with FR-BRAIN-101 ingest job.

Suspected cause: index-rebalancing during ingest
Runbook: <https://kb.cyberos.world/runbooks/brain-ingest-pause|Pause ingest temporarily>
Trace: <https://grafana.cyberos.world/trace/0af7651...|view in Tempo>

[Ack] [Escalate to PD]
```

### `obs.alert_triaged` audit row

```json
{
  "kind": "obs.alert_triaged",
  "payload": {
    "alert_name": "BrainSearchLatencyHigh",
    "severity": "P2",
    "cuo_confidence": 0.85,
    "route": "chat",
    "suggested_runbook": "kb.cyberos.world/runbooks/brain-ingest-pause",
    "trace_id": "0af7651916cd43dd8448eb211c80319c",
    "request_id": "obs_router_..."
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Confidence-calibration tuning (FR-CUO-104) — slice 4+.
- Per-tenant alert routing (different ops teams per tenant) — slice 5+.
- Alert auto-resolve when underlying metric recovers — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| CUO skill timeout (5s) | tokio timeout | confidence=0 → PagerDuty | Investigate CUO |
| CUO skill error | catch-all | confidence=0 → PagerDuty | Investigate CUO |
| CHAT post fails | http error | Fallback to PagerDuty | Self-resolves |
| PagerDuty unreachable | http error | Last-resort emergency CHAT post | Sev-1 log |
| Sev-1 misclassified by sender | severity check | sev-1 always pages regardless | By design |
| Webhook secret missing | auth check | 401 | Operator fixes Alertmanager config |
| Webhook secret leaked | unauthorized webhooks | sev-1 alarm | Rotate secret |
| Triage > 10s | OTel histogram | sev-3 alarm | Investigate CUO |
| Dedup window too short (legitimate re-fire missed) | metric anomaly | Investigate; tune window | Config |
| Ack button URL unreachable | callback fails | CHAT post stays "unacked" | Operator manually acks via PagerDuty |
| Escalate button → already-PD-routed alert | dedup check | No-op | By design |
| BRAIN audit emit fails | brain_writer error | Sev-1 log; route still completes | Operator investigates BRAIN |
| CUO returns confidence > 1.0 (skill bug) | clamp at parse | Treat as 1.0 | Investigate skill |
| Alertmanager webhook payload schema change | parse fails | Sev-1; Alertmanager retries | Update parser to handle new schema |
| Multiple alerts in one webhook | iterate per alert | Each routed independently | By design |
| CHAT channel `#oncall` archived/deleted | post fails | Fallback to PagerDuty | Operator restores channel |
| Suggested runbook URL invalid | rendered as broken link in CHAT | Operator fixes runbook | Standard fix |

---

## §11 — Notes

- The `obs.triage-alert@1` skill is a markdown-only skill (no executable code) with RAG over the KB runbook corpus (FR-KB-008). The skill's confidence comes from runbook-match quality.
- Sev-1 always pages — never trust triage at the highest severity. Even confidence 0.99 sev-1 routes to BOTH.
- CUO skill failure (timeout OR error) → confidence=0 → PagerDuty fallback. Never silent-drop.
- Dedup window 5min collapses repeated fires of the same alert. The counter "fired N times in last 5m" tells ops the persistence.
- Ack button + auto-close-PagerDuty eliminates the dual-action problem (ack in CHAT AND ack in PagerDuty).
- Escalate button gives ops one-click PagerDuty escalation when CHAT triage doesn't capture the severity.
- Webhook secret prevents alert poisoning. Rotation cadence quarterly.
- The 10s triage+route p95 budget is the user-experience floor. Above 10s, ops doesn't see alerts in their natural workflow.

---

*End of FR-OBS-007. Status: draft (10/10 target).*
