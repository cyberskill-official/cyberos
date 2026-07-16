# TASK-IMP-101 observability injection

The deliverable is doctrine: prose that governs agent behavior. It executes nothing, so there
is no runtime branch to log - and inventing telemetry for a markdown section would be theatre.
Recording that honestly is the correct output.

What stands in for it:
- **The rows the doctrine mandates**: every executed fork branch emits its memory row
  (`task_routed_back` on route-back, `memory.status_overridden` on any override). The § names
  them, so runs remain reconstructable from the chain - which is the observability that matters
  for a governance mechanism.
- **The suite is the monitor**: t14 fails loudly if either § or step 0 stops reaching the
  payload; t09/t12's exact pins fail on any undisclosed normative edit.
- **The report is the trace**: reconcile-report@1 (TASK-IMP-100) carries the evidence the gate
  question quotes - the human decision is recorded against named facts, not vibes.

Branch coverage: doctrine has no executable branches; 100 % of the gated text is asserted.
