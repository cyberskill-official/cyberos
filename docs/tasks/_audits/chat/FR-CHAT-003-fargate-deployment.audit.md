---
task_id: TASK-CHAT-003
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 14
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per task-audit skill §0; ISS-007..014 added)
---

## §1 — Verdict summary

TASK-CHAT-003 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 26 §1 clauses (Terraform module, per-tenant stack, 3 tiers, SG isolation, VPC endpoints, AUTH/memory PrivateLink, KMS encryption, tagging, post-apply audit, OTel, idempotency, destroy semantics, CloudWatch alarms, image-digest pinning, ALB+TLS+stickiness, Secrets-Manager-backed RDS creds, weekly PITR, custom Postgres parameter group, custom Redis parameter group, least-privilege IAM, memory inventory rows, state-backend convention, module outputs, dry_run flag, module-version output, data-residency enforcement). 18 §2 rationale paragraphs (added 12 for new clauses). §3 contains full HCL for variables (with new variables), ECS, RDS, Redis, Networking (with VPC endpoint declarations and SG cross-refs), ALB, secrets, parameter_groups, IAM least-privilege task & exec roles, observability with 7 CloudWatch alarms, outputs, version, post-apply audit hook, residency-cross-validation precondition. 41 ACs (added 25). §5 contains 8 verification scripts: static (fmt/validate/tflint/checkov/tfsec), plan-each-tier with resource-count assertions, variable-validation (image-digest + residency), idempotency + reproducible plan, SG egress inspection, IAM simulation, weekly PITR workflow, LocalStack apply, ALB TLS policy. §6 deepens with 10 wiring subsections (state backend, tier-upgrade runbook, drift detection, image promotion, KMS lifecycle, memory-writer reachability, cross-tenant backup isolation, dependency ordering, module versioning, apply-time secret seeding). §8 lists 6 example payloads. §10 lists 32 failure rows. §11 lists 22 implementation notes covering ARM Graviton rationale, ALB header smuggling defence, stickiness cookie choice, tier-specific Redis cluster mode, ECS scaling decisions, param-group ordering, replication-slot overprovisioning, lock-table consolidation, ACM lifecycle, ALB vs NLB, residency variable rationale, dry_run scope, reproducible-plan importance, secrets array vs env var, app_cookie vs lb_cookie.

## §2 — Findings (all resolved)

### ISS-001 — Shared vs per-tenant
Shared = noisy neighbour. Resolved: §1 #1-2 + DEC-440 per-tenant; AC #7.

### ISS-002 — Tier definitions
Without tiers, one-size-fits-all. Resolved: §1 #3 + locals for tier specs; AC #3 #4 #5.

### ISS-003 — Egress isolation
Open egress = security risk. Resolved: §1 #4 + #5 SG + VPC endpoints; AC #9.

### ISS-004 — Encryption depth
Without CMK, AWS staff could read. Resolved: §1 #7 + DEC tenant-specific CMK; AC #10 #11.

### ISS-005 — Idempotency
Manual apply = drift. Resolved: §1 #11 + AC #6.

### ISS-006 — Post-apply audit
Without audit row, provisioning untraceable. Resolved: §1 #9 + AC #15 + null_resource provisioner.

### ISS-007 — Image tag mutability (strict-redo pass)
Original spec referenced `cyberos/chat:${var.pinned_image_tag}` but didn't constrain the variable; `:latest` would have shipped silently if someone pasted the wrong value. Container registries allow tag overwrite; `terraform plan` would not show a diff when the same `:latest` tag points to new bytes. Resolved: §1 #14 mandates SHA-256 digest references; variable validation regex enforces; AC #14 + #17 verify; §11 explains the supply-chain rationale.

### ISS-008 — Plaintext RDS password in task definition (strict-redo pass)
Original spec referenced `MM_SQLSETTINGS_DATASOURCE` env var as containing the connection string. Embedding the password in task definition env vars leaks via `ecs:DescribeTasks` to anyone with that permission and persists in ECS state. Resolved: §1 #16 + secrets.tf + ECS `secrets` array with `valueFrom`; rotation via `aws_secretsmanager_secret_rotation`; AC #21 + #22 verify; §11 details the Mattermost-side fetch.

### ISS-009 — No ALB/TLS in original (strict-redo pass)
Original spec mentioned Fargate listening on port 8065 but never said how external traffic reaches it. Without ALB + TLS, tenants are unreachable OR exposed via plaintext + ephemeral IPs. Resolved: §1 #15 + alb.tf + TLS-13 policy + Mattermost cookie stickiness + ACM cert validation; AC #18-20 verify.

### ISS-010 — Backups untested (strict-redo pass)
Original §1 mentioned 7-day backup retention but had no validation that backups can actually be restored. Resolved: §1 #17 mandates weekly synthetic PITR test; `chat-pitr-weekly.yml` workflow; AC #23 + `chat.pitr_test_passed` memory audit row.

### ISS-011 — Postgres needs custom parameter group for downstream FRs (strict-redo pass)
TASK-CHAT-004 requires pgroonga; TASK-CHAT-005 requires wal2json + logical_replication. The original spec used the default parameter group, which would have forced reboots later. Resolved: §1 #18 + parameter_groups.tf with all required settings at apply time, including pending-reboot params so the first apply pays the reboot cost; AC #24 + #25 verify.

### ISS-012 — IAM task role overpermissive (strict-redo pass)
Original `iam.tf` description said "task role permitting only the tenant's memory socket access" but didn't enumerate the policy. Default scaffolds give `*:*` which is unsafe. Resolved: §1 #20 + iam.tf with per-resource ARN scoping + `kms:ViaService` condition; AC #27 + #28 + `iam-task-role-simulation.sh` verify with `aws iam simulate-principal-policy`.

### ISS-013 — Inventory query unsupported (strict-redo pass)
Operators answering "what's in tenant X's stack" had to run `aws resourcegroupstaggingapi` — slow + AWS-console-only. Resolved: §1 #21 + post-apply hook emits `chat.deployment_inventory` memory row with full ARN enumeration; AC #29 + example payload.

### ISS-014 — Data residency unenforced (strict-redo pass)
Vietnamese tenants subject to Decree 53/2022 require in-country data. Original spec took `aws_region` as a free string. Resolved: §1 #26 + `data_residency` variable enum + `null_resource.residency_assertion` precondition; AC #34 + #35 verify; §11 explains.

## §3 — Resolution

All 14 mechanical concerns addressed. **Score = 10/10.**

Per task-audit skill §0 master rule: spec is now perfect — depth bounded by genuine architectural surface (Terraform module touches ~15 AWS resource types, each with its own failure modes and configuration knobs), not by line targets.

---

*End of TASK-CHAT-003 audit.*
