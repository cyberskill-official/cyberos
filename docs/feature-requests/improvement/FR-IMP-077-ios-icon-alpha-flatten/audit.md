---
fr_id: FR-IMP-077
audited: 2026-07-13
verdict: PASS (after revision)
score_pre_revision: 8/10
score_post_revision: 10/10
issues_resolved: 2
template: engineering-spec@1
---
- ISS-001: draft flattened without a grounded color choice - resolved: #fff sourced from ic_launcher_background.xml (the brand's own adaptive background), recorded in source_pages.
- ISS-002: draft kept byte-identity for iOS alongside the flatten (self-contradiction; guard would always fail) - resolved: guard reframed to the real submission invariants (present/1024/no-alpha), Android hash guard untouched.
Score = 10/10.
