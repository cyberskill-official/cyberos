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
With no proposal source it reports `seen=0` - the safe baseline.

To feed it real candidates, point it at a refinement-proposals directory (one with an `open/` subdir) and
the skill root under which `<skill_name>/SKILL.md` live:

```
python3 -m cuo.core.dream_runner \
  --proposals-dir path/to/proposals \
  --skill-root path/to/skills \
  --audit-log /tmp/dream-audit.jsonl
```

Now each open proposal is mapped to its target SKILL.md, run through the path envelope and the real
FR-CUO-202 classifier, and recorded - and still applied to nothing in propose mode. The proposal files are
read only; the feed never moves or edits them.

## What is wired, and what is left

The FR-CUO-201 proposal feed and the FR-CUO-202 classifier are now bound through the runner: pass
`--proposals-dir` and `--skill-root` and propose mode surfaces real candidates, evaluates each through every
gate, and records them. It still applies nothing.

One deliberate step remains: only when you trust what propose mode surfaces, bind the FR-CUO-202
`apply_proposal` as the real applier and move to auto (the four locks above). That binding is intentionally
not wired into the runner yet, so today nothing can auto-apply by any path.

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
