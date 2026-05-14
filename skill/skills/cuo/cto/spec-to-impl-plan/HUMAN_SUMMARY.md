# `spec-to-impl-plan` human summary template

```
🎫 Impl-plan emitted — `cuo/cto/spec-to-impl-plan` v{skill_version}
📄 Plan: {impl_plan_path}
🔗 From: {tech_spec_path or fr_path}
🏷️ Profile: {chain_profile}

📊 Tickets: {total_tickets} ({sizing_distribution})
   Estimated effort: ~{total_estimated_engineer_days} engineer-days
   Capacity check: {capacity_status}

🚦 Outcome: {emoji} {outcome}
   {if HALT_BEFORE_CREATE_TICKETS: ⚠️  Awaiting your approval before creating tickets in {proj_backend}.}
   {if TICKETS_CREATED: ✅ {N} tickets created in {proj_backend}. See ## Ticket Index in {impl_plan_path}.}
   {if MARKDOWN_ONLY: 📝 Markdown written; no tickets created (per your instruction).}

📊 Trace: {trace_id}  |  Audit row: {audit_row_id}
```

Outcomes: `IMPL_PLAN_COMPLETE | TICKETS_CREATED | HALT_BEFORE_CREATE_TICKETS | HALTED_HITL | REFUSED_NON_PASS_INPUT | MARKDOWN_ONLY | EXHAUSTED | USER_ABORTED`.
