---
fr_id: FR-IMP-078
audited: 2026-07-13
verdict: PASS (after revision)
score_pre_revision: 8/10
score_post_revision: 10/10
issues_resolved: 2
template: engineering-spec@1
---
- ISS-001: draft fixed only the android lane (the reported failure) - resolved: the same 10706 is already consumed at ASC for iOS 1.0.0, so the iOS stamp step gets the flag in the same change; fixing one lane would have moved the failure, not removed it (spec risk_if_skipped).
- ISS-002: draft derived the floor from the tagged commit's timestamp (deterministic, same value across jobs) - resolved: rejected because re-tagging the SAME commit would reuse the number and collide; wall-clock minutes are collision-free across runs (spec §1 clause 4 records the trade).
Score = 10/10.
