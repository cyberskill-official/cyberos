---
fr_id: FR-CHAT-011
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 20
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; feature-request-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per feature-request-audit skill §0; ISS-007..020 added)
---

## §1 — Verdict summary

FR-CHAT-011 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 26 §1 clauses (device table, register/unregister, MM plugin trigger, webhook, privacy payload, modern auth, error handling, audit, mute respect, fan-out budget, metrics, RLS, per-(subject,channel) dedup, badge count, DnD windows, sender_subject_id in data, silent pushes, per-tenant APNS topic, @channel mention triggers mention-only, trace_id end-to-end + apns-id, per-recipient rate limit 60/min, APNS prod vs sandbox, last_delivered_at tracking, payload priority field, per-tenant title template, per-device locale). 21 §2 rationale paragraphs. §3 contains: full schema with rate_limit_state + locale + last_delivered_at + tenant settings columns, ECS apns/fcm modules with build_payload + render_title + DnD action helper + suppression cache + rate limiter, schema for cyberos_chat_tenant_settings extension. 33 ACs. §5 contains 20 named test bodies covering privacy + HTTP/2 + service-account JWT + 410-soft-delete + mute matrix + channel-mute + fan-out latency + dedup + badge + DnD strategies + silent + APNS topic + @channel mention + apns-id trace + rate-limit + sandbox endpoint + last_delivered_at + title template + locale fallback + DnD window pure-function table. §6 deepens with 9 wiring subsections (deployment, secrets, plugin transport, DnD queue, APNS topic registration, badge source, failure routing, stale device cleanup, operator CLI). §8 lists 7 example payloads (delivered + failed + APNS banner + APNS silent + FCM v1 banner + register request + DnD dropped). §10 lists 45 failure rows. §11 lists 28 implementation notes covering HTTP/2 keepalive, FCM auth crate choice, suppression cache LRU, per-tenant topic rationale, per-recipient webhook architecture, DnD queue storage choice, HTTP/2 vs MQTT, FCM v1 vs server-key, rate-limit calibration vs Apple's ceiling, suppression-window calibration, locale support scope, apns-id trace correlation, badge fetch tradeoffs, silent-push throttling, web-push deferral.

## §2 — Findings (all resolved)

### ISS-001 — Payload privacy
Lock-screen exposure of body = PDPL. Resolved: §1 #6 + DEC-520 no body.

### ISS-002 — Auth modernisation
Legacy deprecated. Resolved: §1 #7 + DEC-522 HTTP/2 JWT only.

### ISS-003 — Device lifecycle
Unregistered tokens accumulate. Resolved: §1 #8 soft-delete on 410.

### ISS-004 — Mute respect
Override = trust breach. Resolved: §1 #10 + AC #10-13.

### ISS-005 — Fan-out architecture
Plugin-direct = scale risk. Resolved: §1 #4 + #5 two-tier.

### ISS-006 — Latency budget
> 1s = stale push. Resolved: §1 #11 + AC #14.

### ISS-007 — Rapid messages cause vibration storm (strict-redo pass)
Without dedup, 5 typing-burst messages = 5 pushes = 5 vibrations. Resolved: §1 #13 + suppress::check_and_record 1s window + chat_push_suppressed_total counter + AC #19.

### ISS-008 — Badge count missing (strict-redo pass)
iOS / Android lock-screen aggregate state is the badge number; absent badge = users don't know "how many" without opening the app. Resolved: §1 #14 + MM API fetch + badge in APNS aps.badge + FCM notification_count + AC #20.

### ISS-009 — DnD windows unhandled (strict-redo pass)
Vietnamese workdays end ~22:00 ICT; pushes at 02:00 wake users. Without DnD respect = user trust breach. Resolved: §1 #15 + is_in_dnd helper + queue|drop strategy + AC #21 + #22 + DnD pure-function table.

### ISS-010 — Sender subject_id missing from data (strict-redo pass)
Client apps need to render sender profile pic; display_name alone is ambiguous. Resolved: §1 #16 + data.sender_subject_id in both APNS + FCM + AC #23.

### ISS-011 — Silent push for badge updates missing (strict-redo pass)
Sending banners every time the badge changes is intrusive; silent pushes update state without UI interruption. Resolved: §1 #17 + content-available:1 / data-only FCM + AC #24.

### ISS-012 — APNS topic per-tenant unspecified (strict-redo pass)
Multi-tenant device users couldn't manage per-tenant push permissions. Resolved: §1 #18 + topic suffix com.cyberskill.chat.<tenant> + AC #25.

### ISS-013 — @channel/@here mentions wouldn't trigger mention-only users (strict-redo pass)
Original spec said "mention" = direct @user only; Mattermost mention semantics include @channel + @here. Resolved: §1 #19 + mention regex extension + AC #26.

### ISS-014 — Trace propagation broken at APNS boundary (strict-redo pass)
APNS request has its own apns-id; without echoing our trace_id, the chain breaks. Resolved: §1 #20 + apns-id header set to trace_id + AC #27.

### ISS-015 — No per-recipient rate limit (strict-redo pass)
Misuse (a tenant's automation generates 1k msgs/min to one user) trips Apple's topic-wide rate limit, suspending push for ALL users. Resolved: §1 #21 + Postgres-backed sliding-window limiter at 60/min + chat_push_rate_limited_total + AC #28.

### ISS-016 — APNS sandbox vs production not configurable (strict-redo pass)
Dev/staging builds need sandbox endpoint; original spec hardcoded production. Resolved: §1 #22 + apns_environment config + per-tenant setting + AC #29.

### ISS-017 — No staleness tracking (strict-redo pass)
Stale devices (user reinstalled app months ago) accumulate without visibility. Resolved: §1 #23 + last_delivered_at column + idx_push_stale partial index + AC #30 + nightly cleanup CLI.

### ISS-018 — No priority differentiation (strict-redo pass)
Silent state-syncs sent at HIGH priority waste Apple's high-priority budget. Resolved: §1 #24 + priority based on is_silent + AC #31.

### ISS-019 — Per-tenant title format inflexible (strict-redo pass)
Multi-tenant users want "ACME · #engineering" not just "engineering". Resolved: §1 #25 + push_title_template column + render_title helper + AC #32.

### ISS-020 — No locale support (strict-redo pass)
VN users got English-only fallback titles. Resolved: §1 #26 + locale column on push_devices + locale-aware fallback in render_title + AC #33.

## §3 — Resolution

All 20 mechanical concerns addressed. **Score = 10/10.**

Per feature-request-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine surface (APNS + FCM v1 auth × privacy payload × per-tenant topic × DnD windows × dedup × badge × silent push × rate limit × locale × prod/sandbox), not by line targets.

---

*End of FR-CHAT-011 audit.*
