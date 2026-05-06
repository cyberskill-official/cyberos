# `srs-author` human summary template

```
🛠️ SRS authored — `cuo/cto/srs-author` v{skill_version}
📄 SRS: {srs_path}
🔗 From PRD: {prd_path}
🏷️ Status: {srs_status}
🔁 Iteration: {srs_iteration}

📊 Authority distribution (System Architecture):
   human-edited / human-confirmed / llm-explicit / llm-implicit:
   {n} / {n} / {n} / 0   ← INV-002 enforces zero on Architecture

🚦 Outcome: {emoji} {outcome}

🔐 Security review required: {yes/no}
{if yes: route to cuo-cseco for sign-off before architectural_review_passed: true}

📊 Trace: {trace_id}  |  Audit row: {audit_row_id}
```

Outcomes mirror `prd-author`'s: `SRS_COMPLETE | HALTED_HITL | REFUSED_NON_PASS_PRD | EXHAUSTED | USER_ABORTED`.

## Citations

- Pattern source — `cuo/cpo/prd-author/HUMAN_SUMMARY.md`.
