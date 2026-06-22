# Dream loop operator runbook (FR-CUO-204)

The dream loop lets CyberOS propose and, eventually, auto-apply small self-improvements when it is idle.
It is now enabled in propose mode: it runs, evaluates every candidate through all the gates, and records
what it would do, but it applies nothing. Moving to auto-apply is a separate, deliberate step described
below.

## The enablement ladder

`modules/cuo/config/dream.yaml` has a `mode`:

- off - the loop never runs.
- propose - the loop runs and records, applies nothing. This is the shipped setting.
- auto - the loop may auto-apply a change, but only when every lock below is satisfied.

`enabled: true` and an unset kill switch are required for any run. The kill switch wins over everything:

```
export CYBEROS_DREAM_KILL=1   # hard stop, regardless of config
```

## Running it (propose mode)

One cycle, writing an audit trail, changing nothing:

```
cd modules/cuo
python3 -m cuo.core.dream_runner --audit-log /tmp/dream-audit.jsonl
```

It prints a report (seen / applied / halted-for-human) and appends one JSON row per action to the log.
With the shipped wiring it proposes nothing yet - the proposal source is bound separately (see below) - so
a clean run reports `seen=0`. That is the safe baseline: the runtime and all the gates are live and proven,
waiting only for a proposal feed.

## What still has to be wired before it does real work

The runner ships with an empty proposer and no real applier on purpose. Two deliberate steps remain, both
flowing through this same gated runner:

1. Bind the FR-CUO-201 refinement proposer as `propose_fn` and the FR-CUO-202 `classify_proposal` as
   `classify_fn`, so real candidates (driven by the harness self-audit signals) appear in propose runs.
   This is safe to do immediately: propose mode still applies nothing.
2. Only when you trust what propose mode surfaces, bind the FR-CUO-202 `apply_proposal` as `real_apply_fn`
   and move to auto.

## Going to auto (the four locks)

Auto-apply happens only when ALL of these hold; miss any one and the runner silently falls back to a dry
run that changes nothing:

1. `enabled: true` and the kill switch unset;
2. `mode: auto` in the config;
3. the explicit runtime opt-in: `python3 -m cuo.core.dream_runner --allow-auto-apply`;
4. you are on a dedicated dream branch (its name contains "dream", e.g. `auto/dream`).

Even then, each individual change must independently clear the three content gates: the path envelope
(allowlist and denylist below), the FR-CUO-202 low-risk classifier, and the AWH test gate. The runner never
commits, pushes, or deploys - review the applied diff and commit it yourself.

## The safety boundary (please review)

The envelope is default-deny: a target is touched only if it matches the small allowlist and matches no
denylist entry. The allowlist is just skill prompt bodies, audit rubric wording, workflow step ordering,
and goldenset thresholds. The denylist (in `dream.yaml`) blocks, among others: the loop's own machinery
(`*dream*`, `*evolution_envelope*`), auth and RBAC, the audit chain, tenant isolation and RLS, PII
redaction, cost math, secrets and keys (`*secret*`, `*credential*`, `*.pem`, `*.key`, `*.env*`, `*.live`),
deploy and CI and release, the verification harness itself (`tools/*`, `scripts/*`), schema migrations, and
the memory binlogs. Read that list before you move to auto; it is the line the loop can never cross.
