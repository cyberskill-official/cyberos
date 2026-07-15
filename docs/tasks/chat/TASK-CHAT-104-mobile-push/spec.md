---
id: TASK-CHAT-104
title: Real mobile/web push delivery
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-07-02T00:00:00+07:00
priority: p2
department: engineering
author: @stephencheng
template: task@1
module: chat
status: draft
created: 2026-07-02
origin: module-review-2026-07-02 (re-homed from the archived Mattermost-era spec)
---

# TASK-CHAT-104: Real mobile/web push delivery

TASK-CHAT-011 (archived) + services/chat/src/push.rs already computes push targets; needs APNS/FCM or VAPID web-push delivery.

Acceptance criteria: to be sliced when scheduled; the archived spec under docs/tasks/_archive/chat/ is the starting inventory of requirements, re-read against the native service (services/chat).
