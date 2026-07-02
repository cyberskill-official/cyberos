---
id: FR-CHAT-003
title: "Per-tenant CHAT deployment — AWS Fargate + RDS Multi-AZ + Redis ElastiCache with Terraform module and per-tenant isolation"
module: CHAT
priority: MUST
status: superseded
superseded_by: FR-CHAT-101 (first-party native chat replaced the Mattermost fork wholesale; still-wanted intents re-homed as FR-CHAT-102..106)
verify: I
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_frs: [FR-CHAT-001, FR-CHAT-002, FR-CHAT-004, FR-CHAT-005, FR-OBS-001]
depends_on: [FR-CHAT-001, FR-CHAT-002]
blocks: [FR-CHAT-004, FR-CHAT-005, FR-CHAT-011]

source_pages:
  - website/docs/modules/chat.html#deployment
  - website/docs/runbooks/chat-deploy-runbook.html
source_decisions:
  - DEC-440 (per-tenant deployment — one Fargate service + one RDS + one Redis per tenant)
  - DEC-441 (Terraform module 'tenant_chat'; idempotent apply; tenant_id is the unique key)
  - DEC-442 (Multi-AZ RDS + Redis cluster mode for HA; single-AZ for trial tier)
  - DEC-443 (egress to FR-AUTH-004 JWKS + FR-MEMORY-101 MemoryWriter via VPC endpoint, not public internet)

language: terraform 1.7
service: cyberos/infra/terraform/modules/tenant_chat/
new_files:
  - infra/terraform/modules/tenant_chat/main.tf
  - infra/terraform/modules/tenant_chat/variables.tf
  - infra/terraform/modules/tenant_chat/outputs.tf
  - infra/terraform/modules/tenant_chat/ecs.tf
  - infra/terraform/modules/tenant_chat/rds.tf
  - infra/terraform/modules/tenant_chat/redis.tf
  - infra/terraform/modules/tenant_chat/networking.tf
  - infra/terraform/modules/tenant_chat/iam.tf
  - infra/terraform/modules/tenant_chat/observability.tf
  - infra/terraform/modules/tenant_chat/README.md
  - infra/terraform/examples/single-tenant-chat/main.tf
allowed_tools:
  - file_read: infra/terraform/**
  - file_write: infra/terraform/modules/tenant_chat/**, infra/terraform/examples/**
  - bash: cd infra/terraform/modules/tenant_chat && terraform validate
  - bash: cd infra/terraform/modules/tenant_chat && terraform fmt -check
disallowed_tools:
  - run terraform apply from this FR (CI-managed)
  - shared infrastructure across tenants (per DEC-440)

effort_hours: 6
sub_tasks:
  - "0.5h: variables.tf — tenant_id, tier (trial|standard|premium), aws_region, vpc_id"
  - "1.0h: networking.tf — security groups (only Fargate → RDS:5432, Fargate → Redis:6379)"
  - "1.5h: ecs.tf — Fargate service running cyberos/chat image; auto-scaling 1-10 tasks per tier"
  - "1.0h: rds.tf — PostgreSQL 16 with Multi-AZ (standard+); single-AZ (trial); encrypted at rest"
  - "1.0h: redis.tf — ElastiCache Redis 7; cluster mode (standard+); single-node (trial)"
  - "0.5h: iam.tf — task role permitting only the tenant's memory socket access"
  - "0.5h: observability.tf — CloudWatch log group + FR-OBS-001 collector sidecar"
  - "0.5h: outputs.tf — chat_url, rds_endpoint, redis_endpoint (sensitive)"
  - "0.5h: example single-tenant invocation"
risk_if_skipped: "Shared infra = noisy-neighbour risk; one tenant's flood degrades all. Per-tenant isolates blast radius. Single-AZ at standard tier = downtime on AZ failure. Without VPC endpoints, every JWKS fetch + memory write goes over public internet = latency + cost + attack surface. Without Terraform module, ops creates by hand → drift."
---

## §1 — Description (BCP-14 normative)

The CHAT deployment **MUST** be a Terraform module provisioning a per-tenant isolated stack on AWS. The contract:

1. **MUST** be a Terraform module at `infra/terraform/modules/tenant_chat/` invokable per tenant with variables `{tenant_id, tier, aws_region, vpc_id, memory_writer_socket_endpoint, auth_jwks_url}`.
2. **MUST** provision per tenant:
    - 1 Fargate service running cyberos/chat image (FR-CHAT-001).
    - 1 RDS PostgreSQL instance (PostgreSQL 16; ARM Graviton).
    - 1 ElastiCache Redis cluster.
    - Security groups isolating tenant's network.
3. **MUST** support three tiers via `tier` variable:
    - `trial`: 1 Fargate task; RDS db.t4g.micro single-AZ; Redis cache.t4g.micro single-node.
    - `standard`: 1-3 auto-scaled Fargate; RDS db.t4g.small Multi-AZ; Redis cache.t4g.small replicated.
    - `premium`: 1-10 auto-scaled Fargate; RDS db.m6g.large Multi-AZ; Redis cluster.r6g.large 3-shard.
4. **MUST** lock down security groups to least-privilege:
    - Fargate → RDS on port 5432 ONLY.
    - Fargate → Redis on port 6379 ONLY.
    - Fargate egress to internet → BLOCKED (use VPC endpoints).
    - Fargate ingress from ALB on port 8065 ONLY.
5. **MUST** use VPC endpoints for AWS service access (S3 backups, ECR image pull, CloudWatch logs); zero public internet egress.
6. **MUST** use VPC endpoints (PrivateLink) for FR-AUTH-004 JWKS + FR-MEMORY-101 MemoryWriter:
    - JWKS endpoint at `https://auth.internal.cyberos.world/.well-known/jwks.json` resolves via private DNS.
    - memory socket at `tcp://memory-writer.<tenant>.svc.cluster.local:9090`.
7. **MUST** encrypt at rest:
    - RDS storage encryption with tenant-specific KMS CMK.
    - Redis backup encryption with same CMK.
    - ECS task ephemeral volumes encrypted by default.
8. **MUST** apply tagging convention to all resources: `{Tenant: <tenant_id>, Service: chat, Tier: <tier>, ManagedBy: terraform}`.
9. **MUST** emit memory audit `chat.deployment_provisioned` on terraform apply success (via post-apply hook).
10. **MUST** emit OTel metrics via collector sidecar (FR-OBS-001):
    - `chat_fargate_task_count{tenant_id}` (gauge).
    - `chat_rds_connections_active{tenant_id}` (gauge).
    - `chat_redis_evicted_keys_total{tenant_id}` (counter).
11. **MUST** be idempotent: repeated `terraform apply` with same vars → no changes.
12. **MUST** support clean teardown via `terraform destroy`: deletes Fargate, RDS, Redis; preserves CloudWatch logs (90-day retention).
13. **MUST** create CloudWatch alarms:
    - RDS CPU > 80% for 5min → sev-2 page.
    - Fargate task crash loop (> 3 restarts in 5min) → sev-1.
    - Redis evictions > 100/sec → sev-2.
    - RDS replication lag (Multi-AZ replica) > 30s → sev-2.
    - RDS free storage < 20% → sev-2.
    - ALB 5xx rate > 1% over 5min → sev-1.
    - Fargate task memory > 90% for 5min → sev-2.
14. **MUST** pin the Mattermost image to an immutable SHA-256 digest, not a `:latest` tag. The `pinned_image_tag` variable MUST be a content-addressable reference (`cyberos/chat@sha256:<64-hex>`). Image promotion = re-apply with a new digest; no in-place image swap.
15. **MUST** provision an ALB in front of the Fargate service:
    - Listener on 443 with ACM cert for `<tenant>.cyberskill.world`.
    - Listener on 80 → 301 redirect to 443.
    - TLS policy `ELBSecurityPolicy-TLS13-1-2-2021-06` (TLS 1.2 floor; 1.3 preferred).
    - Target group health check on `/api/v4/system/ping` (Mattermost native).
    - Stickiness via Mattermost session cookie for WebSocket affinity.
16. **MUST** manage Mattermost RDS credentials via AWS Secrets Manager, rotated every 30 days. The Fargate task reads the secret at boot; no plaintext password in environment variables or task definition.
17. **MUST** capture daily RDS automated backups for 7d (trial) / 14d (standard) / 30d (premium) and emit a synthetic point-in-time-restore test once per week per tier. PITR test rehydrates to a sentinel timestamp and asserts post-restore row count matches a pre-snapshot.
18. **MUST** maintain a custom RDS parameter group with these settings (deviation from default):
    - `log_min_duration_statement = 100` (log queries > 100ms; FR-OBS-005 consumes).
    - `log_statement = 'mod'` (log DDL + mutations).
    - `pg_stat_statements.track = top` (FR-OBS-005 query-fingerprint source).
    - `shared_preload_libraries = pg_stat_statements,pgroonga,wal2json` (pgroonga for FR-CHAT-004; wal2json for FR-CHAT-005 logical replication).
    - `rds.logical_replication = 1` (FR-CHAT-005 requires).
19. **MUST** maintain a custom Redis parameter group with `maxmemory-policy = allkeys-lru` (Mattermost session cache; LRU eviction on memory pressure is preferable to OOM).
20. **MUST** restrict IAM task role to least privilege:
    - `secretsmanager:GetSecretValue` on the tenant's RDS secret ARN ONLY.
    - `kms:Decrypt` on the tenant's CMK ONLY.
    - `logs:CreateLogStream` + `logs:PutLogEvents` on the tenant's log group ONLY.
    - NO `s3:*`, NO `iam:*`, NO `ec2:*`. The task-execution role (separate from task role) carries the ECR/CloudWatch boilerplate.
21. **MUST** publish the resource inventory of every apply to memory as `chat.deployment_inventory` rows. Each row enumerates: the resource ARNs, the resource counts per type, the AWS account+region, the tier, the Terraform module version SHA. Operators query memory to answer "what's in tenant X's stack" without `aws` CLI access.
22. **MUST** declare a Terraform state-backend convention: the module is invoked from a per-tenant workspace; state lives in `s3://cyberos-terraform-state/<aws-account>/<tenant_id>/chat/terraform.tfstate` with DynamoDB lock table `cyberos-terraform-lock`. State is per-tenant; cross-tenant blast radius from a corrupted state is zero.
23. **MUST** expose outputs from the module so callers (cross-module) can wire dependent resources:
    - `chat_url` (e.g. `https://t-<tenant-shortid>.cyberskill.world`).
    - `rds_endpoint` (sensitive=true; consumed by FR-CHAT-005 logical replication).
    - `redis_endpoint` (sensitive=true).
    - `ecs_cluster_arn`, `ecs_service_arn`.
    - `task_role_arn` (consumed by FR-AUTH-005 cross-account role chain).
    - `cloudwatch_log_group_name`.
24. **MUST** support a `dry_run` variable that, when true, asserts `terraform plan` shows no changes; used by the post-deploy drift detector (FR-OBS-007 consumes).
25. **MUST** include a `module_version` output sourced from a `version.tf` constant; the module is semver-versioned; every apply records the version it provisioned (consumed by `chat.deployment_provisioned` payload).
26. **MUST** support a `data_residency` variable enum `{vn-only, sg-allowed, global}` that constrains the AWS region selection: `vn-only` only permits `ap-southeast-1`; `sg-allowed` adds `ap-southeast-1` + `ap-southeast-2`; `global` opens all supported regions. Mismatch between variable and aws_region MUST fail validation.

---

## §2 — Why this design (rationale for humans)

**Why per-tenant (DEC-440)?** Noisy neighbour isolation; compliance isolation (data residency per tenant). Cost is per-tenant overhead but predictable.

**Why three tiers (§1 #3)?** Trial = cheap eval; standard = SLA-backed prod; premium = HA + scale. Each tier is one Terraform variable; no code branching.

**Why VPC endpoints (DEC-443, §1 #5-6)?** Public-internet egress = NAT cost ($45/mo per tenant) + security exposure. VPC endpoints route within AWS backbone — cheaper + faster + no public path.

**Why per-tenant KMS CMK (§1 #7)?** Tenant-controlled key revocation: revoking CMK renders backups + storage useless to even AWS staff. Compliance-grade isolation.

**Why tagging (§1 #8)?** Cost attribution + automated cleanup; AWS Billing dashboard pivots on tags.

**Why post-apply memory audit (§1 #9)?** Operators investigating "when was tenant X's chat provisioned" query memory. Single source for ops trail.

**Why CloudWatch alarms (§1 #13)?** OBS layer (FR-OBS-007) handles app-level alerts; AWS-native alarms catch infra-level. Two layers for resilience.

**Why pinned image digest, not tag (§1 #14)?** Mutable tags (`:latest`, `:v1.0`) can be silently re-pushed in ECR, swapping the running image without a Terraform diff. SHA-256 digests are content-addressable — the resource graph hashes the image identity, so `terraform plan` correctly shows a change when the image moves. This is required for supply-chain attestation: the Terraform plan output enumerates the exact image bytes deployed.

**Why ALB-fronted Fargate (§1 #15)?** Fargate tasks have ephemeral IPs and roll on scaling. ALB provides a stable DNS name + TLS termination + sticky-session affinity for Mattermost's WebSocket (a fresh connection per scale event would log everyone out). Stickiness is cookie-based, not IP-based, because Mattermost clients sit behind corporate NATs that share IPs.

**Why Secrets Manager for RDS creds (§1 #16)?** Environment variables in task definitions are stored in ECS state — visible to anyone with `ecs:DescribeTasks`. Secrets Manager scopes the read to the task IAM role + audits every fetch in CloudTrail. Rotation runs without restart via the AWS-provided RDS rotation Lambda.

**Why weekly synthetic PITR test (§1 #17)?** Backups that are never restored are aspirational, not real. A weekly automated restore + row-count assertion confirms the backup→restore pipeline still works; ops finds out about it Monday morning, not at 03:00 during an incident. We pay a small RDS-snapshot-restore fee weekly per tenant for this guarantee.

**Why custom parameter group (§1 #18)?** Default Postgres parameter group is read-only in RDS, so any tuning requires a custom group. We need it anyway for `pgroonga` + `wal2json` shared libraries (FR-CHAT-004 and FR-CHAT-005 depend on these). Doing it once, here, prevents downstream FRs from each maintaining their own override.

**Why Redis `allkeys-lru` (§1 #19)?** Mattermost uses Redis for session cache + presence; running out of memory with `noeviction` (the default) returns errors to Mattermost which then degrades to in-process cache (correct but slow). LRU eviction gracefully drops cold sessions, preserving warm sessions — degradation is invisible to users.

**Why least-privilege IAM (§1 #20)?** The task role is the only credential a compromised container has. Restricting to per-tenant ARNs means an SSRF or RCE in one tenant cannot pivot to another's data. This is enforced by ARN-scoped IAM policies that reject wildcards.

**Why `chat.deployment_inventory` row (§1 #21)?** Operators currently run `aws resourcegroupstaggingapi` to enumerate a tenant's stack — slow + requires AWS console access. memory-side inventory rows make it queryable from anywhere a memory sync flows. Compliance audits ("what data do you process") consume these rows.

**Why per-tenant Terraform workspace (§1 #22)?** Single workspace = single state file = one corrupted apply blocks all tenants. Per-tenant workspaces isolate state corruption to one tenant. The DynamoDB lock table is shared because the cost of per-tenant tables exceeds the operational benefit (DynamoDB items are partitioned by key; lock contention is per-key, so a shared table behaves like per-tenant tables for locking purposes).

**Why module-version output (§1 #25)?** When we change the module (e.g. add a new tier), the post-apply memory row records the module version that provisioned each tenant. Downstream incidents ("which tenants are on module v1.4 vs v1.5") become a memory query, not a Terraform-state crawl.

**Why data_residency variable (§1 #26)?** Vietnamese tenants subject to Decree 53/2022 require in-country data. Enforcing region in code prevents an operator from accidentally provisioning a `vn-only` tenant in `us-east-1`. Validation is at variable-validation time (terraform plan), not apply time — fail fast.

---

## §3 — API contract (Terraform module sketches)

### variables.tf

```hcl
variable "tenant_id" {
  description = "Tenant UUID; used in resource naming + tags"
  type        = string
}

variable "tier" {
  description = "Service tier"
  type        = string
  validation {
    condition     = contains(["trial", "standard", "premium"], var.tier)
    error_message = "tier must be trial | standard | premium"
  }
}

variable "aws_region"                    { type = string }
variable "vpc_id"                        { type = string }
variable "memory_writer_socket_endpoint"  { type = string }
variable "auth_jwks_url"                 { type = string }
variable "kms_cmk_arn"                   { type = string }
```

### ecs.tf

```hcl
locals {
  task_specs = {
    trial    = { cpu = "256",  memory = "512",  min = 1, max = 1  }
    standard = { cpu = "512",  memory = "1024", min = 1, max = 3  }
    premium  = { cpu = "1024", memory = "2048", min = 1, max = 10 }
  }
  spec = local.task_specs[var.tier]
}

resource "aws_ecs_cluster" "chat" {
  name = "cyberos-chat-${var.tenant_id}"
  tags = local.common_tags
}

resource "aws_ecs_task_definition" "chat" {
  family                   = "cyberos-chat-${var.tenant_id}"
  cpu                      = local.spec.cpu
  memory                   = local.spec.memory
  network_mode             = "awsvpc"
  requires_compatibilities = ["FARGATE"]
  execution_role_arn       = aws_iam_role.task_execution.arn
  task_role_arn            = aws_iam_role.task.arn

  container_definitions = jsonencode([
    {
      name      = "chat"
      image     = "cyberos/chat:${var.pinned_image_tag}"
      essential = true
      portMappings = [{ containerPort = 8065, protocol = "tcp" }]
      environment = [
        { name = "MM_SQLSETTINGS_DRIVERNAME", value = "postgres" },
        { name = "MM_SQLSETTINGS_DATASOURCE", value = local.rds_dsn },
        { name = "MM_CACHESETTINGS_REDIS",    value = aws_elasticache_replication_group.chat.primary_endpoint_address },
        { name = "CYBEROS_AUTH_JWKS_URL",      value = var.auth_jwks_url },
        { name = "CYBEROS_MEMORY_WRITER_SOCK",  value = var.memory_writer_socket_endpoint },
        { name = "CYBEROS_TENANT_ID",          value = var.tenant_id },
      ]
      logConfiguration = {
        logDriver = "awslogs"
        options = {
          awslogs-group         = aws_cloudwatch_log_group.chat.name
          awslogs-region        = var.aws_region
          awslogs-stream-prefix = "chat"
        }
      }
    },
    {
      name      = "obs-collector"
      image     = "cyberos/obs-collector:latest"
      essential = false
    }
  ])
  tags = local.common_tags
}

resource "aws_ecs_service" "chat" {
  name            = "chat"
  cluster         = aws_ecs_cluster.chat.id
  task_definition = aws_ecs_task_definition.chat.arn
  desired_count   = local.spec.min
  launch_type     = "FARGATE"
  network_configuration {
    subnets          = data.aws_subnets.private.ids
    security_groups  = [aws_security_group.chat_fargate.id]
    assign_public_ip = false
  }
  load_balancer {
    target_group_arn = aws_lb_target_group.chat.arn
    container_name   = "chat"
    container_port   = 8065
  }
  tags = local.common_tags
}

resource "aws_appautoscaling_target" "chat" {
  service_namespace  = "ecs"
  resource_id        = "service/${aws_ecs_cluster.chat.name}/${aws_ecs_service.chat.name}"
  scalable_dimension = "ecs:service:DesiredCount"
  min_capacity       = local.spec.min
  max_capacity       = local.spec.max
}
```

### rds.tf

```hcl
locals {
  rds_specs = {
    trial    = { instance_class = "db.t4g.micro",  multi_az = false }
    standard = { instance_class = "db.t4g.small",  multi_az = true  }
    premium  = { instance_class = "db.m6g.large",  multi_az = true  }
  }
}

resource "aws_db_instance" "chat" {
  identifier          = "cyberos-chat-${var.tenant_id}"
  engine              = "postgres"
  engine_version      = "16.3"
  instance_class      = local.rds_specs[var.tier].instance_class
  allocated_storage   = 50
  storage_encrypted   = true
  kms_key_id          = var.kms_cmk_arn
  multi_az            = local.rds_specs[var.tier].multi_az
  vpc_security_group_ids = [aws_security_group.chat_rds.id]
  db_subnet_group_name   = aws_db_subnet_group.chat.name
  backup_retention_period = 7
  deletion_protection = var.tier == "premium"
  skip_final_snapshot = var.tier == "trial"
  tags                = local.common_tags
}
```

### redis.tf

```hcl
locals {
  redis_specs = {
    trial    = { node_type = "cache.t4g.micro", num_cache_clusters = 1 }
    standard = { node_type = "cache.t4g.small", num_cache_clusters = 2 }
    premium  = { node_type = "cache.r6g.large", num_cache_clusters = 3, num_node_groups = 3 }
  }
}

resource "aws_elasticache_replication_group" "chat" {
  replication_group_id       = "cyberos-chat-${var.tenant_id}"
  description                = "CHAT cache for tenant ${var.tenant_id}"
  node_type                  = local.redis_specs[var.tier].node_type
  num_cache_clusters         = local.redis_specs[var.tier].num_cache_clusters
  parameter_group_name       = "default.redis7"
  port                       = 6379
  subnet_group_name          = aws_elasticache_subnet_group.chat.name
  security_group_ids         = [aws_security_group.chat_redis.id]
  at_rest_encryption_enabled = true
  transit_encryption_enabled = true
  kms_key_id                 = var.kms_cmk_arn
  tags                       = local.common_tags
}
```

### networking.tf

```hcl
resource "aws_security_group" "chat_fargate" {
  name   = "cyberos-chat-fargate-${var.tenant_id}"
  vpc_id = var.vpc_id
  # Ingress from ALB only
  ingress { from_port=8065 to_port=8065 protocol="tcp" security_groups=[aws_security_group.chat_alb.id] }
  # Egress to RDS + Redis only (no public)
  egress  { from_port=5432 to_port=5432 protocol="tcp" security_groups=[aws_security_group.chat_rds.id] }
  egress  { from_port=6379 to_port=6379 protocol="tcp" security_groups=[aws_security_group.chat_redis.id] }
  # Egress to VPC endpoints
  egress  { from_port=443 to_port=443 protocol="tcp" prefix_list_ids=[data.aws_vpc_endpoint.s3.prefix_list_id] }
  tags = local.common_tags
}

resource "aws_security_group" "chat_rds" {
  name   = "cyberos-chat-rds-${var.tenant_id}"
  vpc_id = var.vpc_id
  # Ingress from Fargate only — referenced via SG ID (NOT CIDR).
  ingress { from_port=5432 to_port=5432 protocol="tcp" security_groups=[aws_security_group.chat_fargate.id] }
  # No egress required for RDS.
  tags = local.common_tags
}

resource "aws_security_group" "chat_redis" {
  name   = "cyberos-chat-redis-${var.tenant_id}"
  vpc_id = var.vpc_id
  ingress { from_port=6379 to_port=6379 protocol="tcp" security_groups=[aws_security_group.chat_fargate.id] }
  tags = local.common_tags
}

resource "aws_security_group" "chat_alb" {
  name   = "cyberos-chat-alb-${var.tenant_id}"
  vpc_id = var.vpc_id
  ingress { from_port=443 to_port=443 protocol="tcp" cidr_blocks=["0.0.0.0/0"] }
  ingress { from_port=80  to_port=80  protocol="tcp" cidr_blocks=["0.0.0.0/0"] }
  egress  { from_port=8065 to_port=8065 protocol="tcp" security_groups=[aws_security_group.chat_fargate.id] }
  tags = local.common_tags
}

# VPC endpoints — keep traffic on AWS backbone.
resource "aws_vpc_endpoint" "ecr_api" {
  vpc_id              = var.vpc_id
  service_name        = "com.amazonaws.${var.aws_region}.ecr.api"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = data.aws_subnets.private.ids
  security_group_ids  = [aws_security_group.chat_fargate.id]
  private_dns_enabled = true
  tags                = local.common_tags
}

resource "aws_vpc_endpoint" "ecr_dkr" {
  vpc_id              = var.vpc_id
  service_name        = "com.amazonaws.${var.aws_region}.ecr.dkr"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = data.aws_subnets.private.ids
  security_group_ids  = [aws_security_group.chat_fargate.id]
  private_dns_enabled = true
  tags                = local.common_tags
}

resource "aws_vpc_endpoint" "logs" {
  vpc_id              = var.vpc_id
  service_name        = "com.amazonaws.${var.aws_region}.logs"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = data.aws_subnets.private.ids
  security_group_ids  = [aws_security_group.chat_fargate.id]
  private_dns_enabled = true
  tags                = local.common_tags
}

resource "aws_vpc_endpoint" "secretsmanager" {
  vpc_id              = var.vpc_id
  service_name        = "com.amazonaws.${var.aws_region}.secretsmanager"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = data.aws_subnets.private.ids
  security_group_ids  = [aws_security_group.chat_fargate.id]
  private_dns_enabled = true
  tags                = local.common_tags
}

# Cross-account PrivateLink to FR-AUTH-004 JWKS service.
resource "aws_vpc_endpoint" "auth_jwks" {
  vpc_id              = var.vpc_id
  service_name        = var.auth_jwks_vpc_endpoint_service_name # e.g. com.amazonaws.vpce.ap-southeast-1.vpce-svc-0abc...
  vpc_endpoint_type   = "Interface"
  subnet_ids          = data.aws_subnets.private.ids
  security_group_ids  = [aws_security_group.chat_fargate.id]
  private_dns_enabled = true
  tags                = local.common_tags
}

# S3 gateway endpoint for ECR layer cache.
resource "aws_vpc_endpoint" "s3" {
  vpc_id            = var.vpc_id
  service_name      = "com.amazonaws.${var.aws_region}.s3"
  vpc_endpoint_type = "Gateway"
  route_table_ids   = data.aws_route_tables.private.ids
  tags              = local.common_tags
}
```

### alb.tf

```hcl
resource "aws_lb" "chat" {
  name               = "cyberos-chat-${substr(var.tenant_id, 0, 8)}"
  internal           = false
  load_balancer_type = "application"
  subnets            = data.aws_subnets.public.ids
  security_groups    = [aws_security_group.chat_alb.id]
  enable_deletion_protection = var.tier == "premium"
  drop_invalid_header_fields = true
  tags               = local.common_tags
}

resource "aws_lb_target_group" "chat" {
  name        = "cyberos-chat-${substr(var.tenant_id, 0, 8)}"
  port        = 8065
  protocol    = "HTTP"
  target_type = "ip"
  vpc_id      = var.vpc_id

  health_check {
    path                = "/api/v4/system/ping"
    interval            = 15
    timeout             = 5
    healthy_threshold   = 2
    unhealthy_threshold = 3
    matcher             = "200"
  }

  stickiness {
    type            = "app_cookie"
    cookie_name     = "MMAUTHTOKEN"
    cookie_duration = 86400
    enabled         = true
  }
  tags = local.common_tags
}

resource "aws_lb_listener" "https" {
  load_balancer_arn = aws_lb.chat.arn
  port              = 443
  protocol          = "HTTPS"
  ssl_policy        = "ELBSecurityPolicy-TLS13-1-2-2021-06"
  certificate_arn   = aws_acm_certificate_validation.chat.certificate_arn

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.chat.arn
  }
}

resource "aws_lb_listener" "http_redirect" {
  load_balancer_arn = aws_lb.chat.arn
  port              = 80
  protocol          = "HTTP"
  default_action {
    type = "redirect"
    redirect { port = "443" protocol = "HTTPS" status_code = "HTTP_301" }
  }
}

resource "aws_acm_certificate" "chat" {
  domain_name       = "t-${substr(var.tenant_id, 0, 8)}.${var.cyberos_apex_domain}"
  validation_method = "DNS"
  tags              = local.common_tags
  lifecycle { create_before_destroy = true }
}

resource "aws_route53_record" "chat" {
  zone_id = var.cyberos_route53_zone_id
  name    = aws_acm_certificate.chat.domain_name
  type    = "A"
  alias {
    name                   = aws_lb.chat.dns_name
    zone_id                = aws_lb.chat.zone_id
    evaluate_target_health = true
  }
}
```

### secrets.tf

```hcl
resource "aws_secretsmanager_secret" "rds_password" {
  name        = "cyberos/chat/${var.tenant_id}/rds-password"
  kms_key_id  = var.kms_cmk_arn
  description = "PostgreSQL master password — managed by AWS RDS rotation Lambda"
  tags        = local.common_tags
}

resource "aws_secretsmanager_secret_rotation" "rds_password" {
  secret_id           = aws_secretsmanager_secret.rds_password.id
  rotation_lambda_arn = data.aws_lambda_function.rds_rotation.arn
  rotation_rules { automatically_after_days = 30 }
}

# The Mattermost container reads the password via Secrets Manager fetch at boot.
# Task definition references the secret ARN, NOT the value.
locals {
  mattermost_secrets = [
    {
      name      = "MM_SQLSETTINGS_DATASOURCE"
      valueFrom = "${aws_secretsmanager_secret.rds_password.arn}:dsn::"
    }
  ]
}
```

### parameter_groups.tf

```hcl
resource "aws_db_parameter_group" "chat" {
  name        = "cyberos-chat-${var.tenant_id}"
  family      = "postgres16"
  description = "CyberOS chat Postgres params — pgroonga, wal2json, query logging"

  parameter { name = "log_min_duration_statement"   value = "100" }
  parameter { name = "log_statement"                value = "mod" }
  parameter { name = "log_connections"              value = "1" }
  parameter { name = "log_disconnections"           value = "1" }
  parameter { name = "pg_stat_statements.track"     value = "top" }
  parameter { name = "shared_preload_libraries"
              value = "pg_stat_statements,pgroonga,wal2json"
              apply_method = "pending-reboot" }
  parameter { name = "rds.logical_replication"      value = "1"
              apply_method = "pending-reboot" }
  parameter { name = "max_wal_senders"              value = "10"
              apply_method = "pending-reboot" }
  parameter { name = "max_replication_slots"        value = "10"
              apply_method = "pending-reboot" }

  tags = local.common_tags
}

resource "aws_elasticache_parameter_group" "chat" {
  name        = "cyberos-chat-${var.tenant_id}"
  family      = "redis7"
  description = "CyberOS chat Redis params — LRU eviction"

  parameter { name = "maxmemory-policy" value = "allkeys-lru" }
  parameter { name = "timeout"          value = "300" }
}
```

### iam.tf — least-privilege task role

```hcl
# Task-EXECUTION role — pulls image, writes logs, fetches secret.
resource "aws_iam_role" "task_execution" {
  name = "cyberos-chat-exec-${var.tenant_id}"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Principal = { Service = "ecs-tasks.amazonaws.com" }
      Action = "sts:AssumeRole"
      Condition = {
        StringEquals = { "aws:SourceAccount" = data.aws_caller_identity.current.account_id }
        ArnLike      = { "aws:SourceArn"     = "arn:aws:ecs:${var.aws_region}:${data.aws_caller_identity.current.account_id}:*" }
      }
    }]
  })
  tags = local.common_tags
}

resource "aws_iam_role_policy" "task_execution" {
  role = aws_iam_role.task_execution.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = ["ecr:GetAuthorizationToken"]
        Resource = "*"
      },
      {
        Effect = "Allow"
        Action = ["ecr:BatchCheckLayerAvailability", "ecr:GetDownloadUrlForLayer", "ecr:BatchGetImage"]
        Resource = "arn:aws:ecr:${var.aws_region}:${data.aws_caller_identity.current.account_id}:repository/cyberos/chat"
      },
      {
        Effect = "Allow"
        Action = ["logs:CreateLogStream", "logs:PutLogEvents"]
        Resource = "${aws_cloudwatch_log_group.chat.arn}:*"
      },
      {
        Effect = "Allow"
        Action = ["secretsmanager:GetSecretValue"]
        Resource = aws_secretsmanager_secret.rds_password.arn
      },
      {
        Effect = "Allow"
        Action = ["kms:Decrypt"]
        Resource = var.kms_cmk_arn
        Condition = {
          StringEquals = {
            "kms:ViaService" = "secretsmanager.${var.aws_region}.amazonaws.com"
          }
        }
      }
    ]
  })
}

# Task role — what the running container can do. Intentionally narrow.
resource "aws_iam_role" "task" {
  name = "cyberos-chat-task-${var.tenant_id}"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Principal = { Service = "ecs-tasks.amazonaws.com" }
      Action = "sts:AssumeRole"
    }]
  })
  tags = local.common_tags
}

resource "aws_iam_role_policy" "task" {
  role = aws_iam_role.task.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        # memory audit emission (via cross-account socket; STS not required because socket is private)
        Sid    = "MemoryWriterSocket"
        Effect = "Allow"
        Action = ["sts:GetCallerIdentity"]
        Resource = "*"
      }
    ]
  })
}
```

### observability.tf

```hcl
resource "aws_cloudwatch_log_group" "chat" {
  name              = "/cyberos/chat/${var.tenant_id}"
  retention_in_days = 90
  kms_key_id        = var.kms_cmk_arn
  tags              = local.common_tags
}

resource "aws_cloudwatch_metric_alarm" "rds_cpu" {
  alarm_name          = "cyberos-chat-${var.tenant_id}-rds-cpu"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 5
  metric_name         = "CPUUtilization"
  namespace           = "AWS/RDS"
  period              = 60
  statistic           = "Average"
  threshold           = 80
  dimensions          = { DBInstanceIdentifier = aws_db_instance.chat.id }
  alarm_actions       = [var.sns_sev2_topic_arn]
  tags                = local.common_tags
}

resource "aws_cloudwatch_metric_alarm" "rds_free_storage" {
  alarm_name          = "cyberos-chat-${var.tenant_id}-rds-storage"
  comparison_operator = "LessThanThreshold"
  evaluation_periods  = 3
  metric_name         = "FreeStorageSpace"
  namespace           = "AWS/RDS"
  period              = 300
  statistic           = "Average"
  threshold           = aws_db_instance.chat.allocated_storage * 1024 * 1024 * 1024 * 0.2 # 20% of allocated
  dimensions          = { DBInstanceIdentifier = aws_db_instance.chat.id }
  alarm_actions       = [var.sns_sev2_topic_arn]
}

resource "aws_cloudwatch_metric_alarm" "rds_replica_lag" {
  count               = local.rds_specs[var.tier].multi_az ? 1 : 0
  alarm_name          = "cyberos-chat-${var.tenant_id}-rds-replica-lag"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 3
  metric_name         = "ReplicaLag"
  namespace           = "AWS/RDS"
  period              = 60
  statistic           = "Maximum"
  threshold           = 30
  dimensions          = { DBInstanceIdentifier = aws_db_instance.chat.id }
  alarm_actions       = [var.sns_sev2_topic_arn]
}

resource "aws_cloudwatch_metric_alarm" "fargate_crash_loop" {
  alarm_name          = "cyberos-chat-${var.tenant_id}-fargate-crashloop"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 1
  metric_name         = "TaskFailures"
  namespace           = "ECS/ContainerInsights"
  period              = 300
  statistic           = "Sum"
  threshold           = 3
  dimensions          = { ClusterName = aws_ecs_cluster.chat.name, ServiceName = aws_ecs_service.chat.name }
  alarm_actions       = [var.sns_sev1_topic_arn]
}

resource "aws_cloudwatch_metric_alarm" "redis_evictions" {
  alarm_name          = "cyberos-chat-${var.tenant_id}-redis-evictions"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 5
  metric_name         = "Evictions"
  namespace           = "AWS/ElastiCache"
  period              = 60
  statistic           = "Sum"
  threshold           = 100
  dimensions          = { ReplicationGroupId = aws_elasticache_replication_group.chat.id }
  alarm_actions       = [var.sns_sev2_topic_arn]
}

resource "aws_cloudwatch_metric_alarm" "fargate_memory" {
  alarm_name          = "cyberos-chat-${var.tenant_id}-fargate-memory"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 5
  metric_name         = "MemoryUtilization"
  namespace           = "AWS/ECS"
  period              = 60
  statistic           = "Average"
  threshold           = 90
  dimensions          = { ClusterName = aws_ecs_cluster.chat.name, ServiceName = aws_ecs_service.chat.name }
  alarm_actions       = [var.sns_sev2_topic_arn]
}

resource "aws_cloudwatch_metric_alarm" "alb_5xx" {
  alarm_name          = "cyberos-chat-${var.tenant_id}-alb-5xx"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 5
  metric_name         = "HTTPCode_Target_5XX_Count"
  namespace           = "AWS/ApplicationELB"
  period              = 60
  statistic           = "Sum"
  threshold           = 10
  dimensions          = { LoadBalancer = aws_lb.chat.arn_suffix }
  alarm_actions       = [var.sns_sev1_topic_arn]
}
```

### outputs.tf

```hcl
output "chat_url"                  { value = "https://${aws_acm_certificate.chat.domain_name}" }
output "rds_endpoint"              { value = aws_db_instance.chat.endpoint                    sensitive = true }
output "redis_endpoint"            { value = aws_elasticache_replication_group.chat.primary_endpoint_address sensitive = true }
output "ecs_cluster_arn"           { value = aws_ecs_cluster.chat.arn }
output "ecs_service_arn"           { value = aws_ecs_service.chat.id }
output "task_role_arn"             { value = aws_iam_role.task.arn }
output "cloudwatch_log_group_name" { value = aws_cloudwatch_log_group.chat.name }
output "kms_cmk_arn"               { value = var.kms_cmk_arn }
output "alb_arn"                   { value = aws_lb.chat.arn }
output "module_version"            { value = local.module_version }
output "rds_parameter_group_name"  { value = aws_db_parameter_group.chat.name }
output "redis_parameter_group_name"{ value = aws_elasticache_parameter_group.chat.name }
```

### version.tf

```hcl
locals {
  module_version = "1.0.0" # bumped on contract-changing edits; consumed by deployment_provisioned audit row
}
```

### post-apply memory audit hook

```hcl
# main.tf
resource "null_resource" "deployment_audit" {
  triggers = {
    tenant_id      = var.tenant_id
    tier           = var.tier
    rds_endpoint   = aws_db_instance.chat.endpoint
    redis_endpoint = aws_elasticache_replication_group.chat.primary_endpoint_address
    chat_url       = "https://${aws_acm_certificate.chat.domain_name}"
    module_version = local.module_version
  }

  provisioner "local-exec" {
    command = <<-EOT
      cyberos-memory-cli emit \
        --kind chat.deployment_provisioned \
        --tenant-id ${var.tenant_id} \
        --payload '${jsonencode({
          tier             = var.tier,
          aws_region       = var.aws_region,
          chat_url         = "https://${aws_acm_certificate.chat.domain_name}",
          ecs_cluster_arn  = aws_ecs_cluster.chat.arn,
          rds_endpoint     = aws_db_instance.chat.endpoint,
          redis_endpoint   = aws_elasticache_replication_group.chat.primary_endpoint_address,
          module_version   = local.module_version,
          terraform_run_id = terraform.workspace,
          provisioned_at_ns = timestamp()
        })}'

      cyberos-memory-cli emit \
        --kind chat.deployment_inventory \
        --tenant-id ${var.tenant_id} \
        --payload '${jsonencode({
          ecs_service_arn  = aws_ecs_service.chat.id,
          rds_instance_arn = aws_db_instance.chat.arn,
          redis_cluster_arn= aws_elasticache_replication_group.chat.arn,
          alb_arn          = aws_lb.chat.arn,
          log_group        = aws_cloudwatch_log_group.chat.name,
          task_role_arn    = aws_iam_role.task.arn,
          kms_cmk_arn      = var.kms_cmk_arn,
          security_groups  = [
            aws_security_group.chat_fargate.id,
            aws_security_group.chat_rds.id,
            aws_security_group.chat_redis.id,
            aws_security_group.chat_alb.id,
          ],
          module_version   = local.module_version
        })}'
    EOT
  }
}
```

### variables.tf — additional fields

```hcl
variable "pinned_image_tag" {
  description = "Mattermost image; MUST be a SHA-256 digest reference (e.g. 'cyberos/chat@sha256:abc...')"
  type        = string
  validation {
    condition     = can(regex("^cyberos/chat@sha256:[a-f0-9]{64}$", var.pinned_image_tag))
    error_message = "pinned_image_tag MUST be a sha256-digest reference (no :latest, no :v1.0)"
  }
}

variable "cyberos_apex_domain" {
  description = "DNS apex for tenant subdomains (e.g. 'cyberskill.world')"
  type        = string
}

variable "cyberos_route53_zone_id" {
  description = "Route53 hosted zone ID for cyberos_apex_domain"
  type        = string
}

variable "auth_jwks_vpc_endpoint_service_name" {
  description = "PrivateLink service name for the AUTH-004 JWKS service"
  type        = string
}

variable "sns_sev1_topic_arn" {
  description = "SNS topic ARN that fans out to PagerDuty sev-1"
  type        = string
}

variable "sns_sev2_topic_arn" {
  description = "SNS topic ARN that fans out to PagerDuty sev-2"
  type        = string
}

variable "data_residency" {
  description = "Data-residency constraint enum"
  type        = string
  validation {
    condition     = contains(["vn-only", "sg-allowed", "global"], var.data_residency)
    error_message = "data_residency must be vn-only | sg-allowed | global"
  }
}

variable "dry_run" {
  description = "If true, asserts plan shows no changes; consumed by drift detector"
  type        = bool
  default     = false
}
```

### main.tf — region-residency cross-validation

```hcl
locals {
  allowed_regions_by_residency = {
    vn-only    = ["ap-southeast-1"]
    sg-allowed = ["ap-southeast-1", "ap-southeast-2"]
    global     = ["ap-southeast-1", "ap-southeast-2", "us-east-1", "eu-west-1"]
  }
}

# Fails plan if data_residency disallows aws_region.
resource "null_resource" "residency_assertion" {
  lifecycle {
    precondition {
      condition     = contains(local.allowed_regions_by_residency[var.data_residency], var.aws_region)
      error_message = "data_residency=${var.data_residency} does not permit aws_region=${var.aws_region}"
    }
  }
}
```

### common tags

```hcl
locals {
  common_tags = {
    Tenant         = var.tenant_id
    Service        = "chat"
    Tier           = var.tier
    ManagedBy      = "terraform"
    Module         = "tenant_chat"
    ModuleVersion  = local.module_version
    DataResidency  = var.data_residency
    CostCenter     = "cyberos-chat"
    Owner          = "platform-team"
  }
}
```

---

## §4 — Acceptance criteria

1. **Module validates** — `terraform validate` clean.
2. **Module formats** — `terraform fmt -check` clean.
3. **Trial tier provisions** — apply with tier=trial → 1 Fargate, single-AZ RDS, single-node Redis.
4. **Standard tier provisions Multi-AZ** — apply with tier=standard → RDS Multi-AZ; Redis 2-node.
5. **Premium tier provisions 3-shard Redis** — apply with tier=premium → cluster-mode Redis.
6. **Idempotent re-apply** — second apply with same vars → no changes.
7. **Tenant ID in resource names** — every resource name contains tenant_id.
8. **Tagging applied** — all resources have Tenant + Tier tags.
9. **Fargate egress restricted** — security group egress rules only to RDS/Redis/VPC endpoints.
10. **RDS encrypted with CMK** — `storage_encrypted=true` + kms_key_id matches input.
11. **Redis at-rest + transit encryption** — both flags true.
12. **CloudWatch alarms created** — 3 alarms per tenant.
13. **Auto-scaling configured** — appautoscaling target exists at standard+ tiers.
14. **Terraform destroy cleans up** — `terraform destroy` removes all resources; CloudWatch logs retain.
15. **memory audit post-apply** — apply complete → `chat.deployment_provisioned` row emitted.
16. **JWKS URL resolves via VPC endpoint** — DNS lookup returns private IP.
17. **Image-digest pinning enforced** — variable validation rejects `cyberos/chat:latest`; only `cyberos/chat@sha256:...` accepted (AC #14).
18. **ALB has TLS 1.2 floor** — listener ssl_policy is `ELBSecurityPolicy-TLS13-1-2-2021-06`; AC #15.
19. **HTTP→HTTPS redirect** — port 80 listener returns 301 to https://; AC #15.
20. **Sticky-session cookie configured** — target group stickiness type `app_cookie` with cookie name `MMAUTHTOKEN`; AC #15.
21. **RDS password in Secrets Manager** — task definition references `secretsmanager:GetSecretValue` for the password, NOT a plaintext env var; AC #16.
22. **Secrets rotation scheduled** — `aws_secretsmanager_secret_rotation` with 30-day cadence; AC #16.
23. **Weekly PITR test passes** — CI workflow restores last week's snapshot to a sentinel cluster, asserts row count matches a pre-snapshot tag; AC #17.
24. **Custom RDS parameter group active** — `terraform state show aws_db_instance.chat` confirms `parameter_group_name = aws_db_parameter_group.chat.name`; AC #18.
25. **shared_preload_libraries includes pgroonga + wal2json** — parameter group contains both names; AC #18.
26. **Redis allkeys-lru policy active** — `terraform state show aws_elasticache_replication_group.chat` confirms parameter_group with `maxmemory-policy=allkeys-lru`; AC #19.
27. **Task role denies wildcard actions** — `iam:SimulatePrincipalPolicy` against `s3:GetObject *` returns DENY for the task role; AC #20.
28. **Task role permits only tenant-scoped secret** — simulation against the tenant's secret returns ALLOW; another tenant's secret returns DENY; AC #20.
29. **`chat.deployment_inventory` memory row emitted** — post-apply hook produces row with full resource ARN enumeration; AC #21.
30. **Terraform workspace per tenant** — `terraform workspace list` shows the tenant workspace; state lives in tenant-scoped S3 prefix; AC #22.
31. **Module outputs all wired** — every output in `outputs.tf` returns a non-empty value post-apply; AC #23.
32. **Dry-run mode catches drift** — running with `dry_run=true` after `terraform apply` returns exit 0 with `No changes`; AC #24.
33. **Module-version tag present on every resource** — `aws_resourcegroupstaggingapi` query for `ModuleVersion=1.0.0` returns ≥ 30 resources for one tenant; AC #25.
34. **Data-residency vn-only blocks us-east-1** — `terraform plan` with `data_residency=vn-only aws_region=us-east-1` fails precondition; AC #26.
35. **Data-residency sg-allowed permits ap-southeast-2** — same plan with `aws_region=ap-southeast-2` succeeds; AC #26.
36. **ECR pull goes via VPC endpoint** — VPC flow logs show ECR API traffic with `dstaddr` in 169.254/16 PrivateLink range, not public ECR IPs.
37. **No 0.0.0.0/0 egress on Fargate SG** — security-group inspection asserts there is no `0.0.0.0/0` egress rule on `chat_fargate`; AC #9 strengthened.
38. **CloudTrail records RDS-secret-read events** — after a task launch, CloudTrail shows `GetSecretValue` with the task IAM role principal.
39. **Premium tier has deletion_protection** — `aws_db_instance.chat.deletion_protection = true` when `tier == "premium"`.
40. **Trial tier `skip_final_snapshot` honoured** — `aws_db_instance.chat.skip_final_snapshot = true` only when `tier == "trial"`.
41. **Reproducible plan** — two consecutive `terraform plan` invocations against the same source + variables produce identical plan JSON.

---

## §5 — Verification

Tests live in `infra/terraform/modules/tenant_chat/tests/`. They form three layers: static (validate + fmt + tflint), plan-time (each-tier plan + variable validation), and apply-time (LocalStack + smoke tests).

### Static layer — runs on every PR

```bash
#!/usr/bin/env bash
# tests/static.sh
set -euo pipefail
cd infra/terraform/modules/tenant_chat

terraform init -backend=false
terraform fmt -check -recursive
terraform validate

# tflint catches AWS-specific anti-patterns.
tflint --module --recursive

# Checkov for compliance: CIS AWS Foundations, NIST, AWS Best Practices.
checkov -d . --framework terraform --check CKV_AWS_*

# tfsec for security-specific lints.
tfsec . --no-color --soft-fail-rules AWS010 # ALB drop_invalid_header_fields is satisfied elsewhere
```

### Plan layer — runs per tier in CI

```bash
#!/usr/bin/env bash
# tests/plan-each-tier.sh
set -euo pipefail

for tier in trial standard premium; do
  echo "=== Planning tier=$tier ==="
  terraform plan \
    -var "tenant_id=00000000-0000-0000-0000-00000000$(printf '%04d' "$RANDOM")" \
    -var "tier=$tier" \
    -var "aws_region=ap-southeast-1" \
    -var "vpc_id=vpc-test" \
    -var "kms_cmk_arn=arn:aws:kms:ap-southeast-1:000000000000:key/abc" \
    -var "pinned_image_tag=cyberos/chat@sha256:0000000000000000000000000000000000000000000000000000000000000000" \
    -var "cyberos_apex_domain=cyberskill.world" \
    -var "cyberos_route53_zone_id=Z000000000000000000000" \
    -var "auth_jwks_vpc_endpoint_service_name=com.amazonaws.vpce.ap-southeast-1.vpce-svc-test" \
    -var "sns_sev1_topic_arn=arn:aws:sns:ap-southeast-1:000000000000:sev1" \
    -var "sns_sev2_topic_arn=arn:aws:sns:ap-southeast-1:000000000000:sev2" \
    -var "data_residency=vn-only" \
    -var "memory_writer_socket_endpoint=tcp://memory.test:9090" \
    -var "auth_jwks_url=https://auth.test/.well-known/jwks.json" \
    -out "plan-$tier.tfplan"

  # Per-tier resource count assertion.
  ACTUAL=$(terraform show -json "plan-$tier.tfplan" | jq '.resource_changes | length')
  EXPECTED_MIN=$(case $tier in trial) echo 25;; standard) echo 30;; premium) echo 35;; esac)
  [[ "$ACTUAL" -ge "$EXPECTED_MIN" ]] || { echo "tier=$tier resource count $ACTUAL < $EXPECTED_MIN"; exit 1; }
done
```

### Variable-validation tests — AC #14, #26, #34, #35

```bash
#!/usr/bin/env bash
# tests/variable-validation.sh
set -euo pipefail

# AC #14 — :latest must be rejected.
if terraform plan -var 'pinned_image_tag=cyberos/chat:latest' -var '...' 2>&1 | grep -q 'sha256-digest'; then
  echo "PASS: pinned_image_tag validation rejects :latest"
else
  echo "FAIL: pinned_image_tag validation did not reject :latest"; exit 1
fi

# AC #34 — vn-only + us-east-1 must fail precondition.
if terraform plan -var 'data_residency=vn-only' -var 'aws_region=us-east-1' -var '...' 2>&1 | grep -q 'does not permit'; then
  echo "PASS: residency precondition rejects vn-only + us-east-1"
else
  echo "FAIL"; exit 1
fi

# AC #35 — sg-allowed + ap-southeast-2 must succeed.
terraform plan -var 'data_residency=sg-allowed' -var 'aws_region=ap-southeast-2' -var '...' >/dev/null
echo "PASS: residency permits sg-allowed + ap-southeast-2"
```

### Idempotency — AC #6, #32, #41

```bash
#!/usr/bin/env bash
# tests/idempotency.sh
set -euo pipefail

terraform apply -auto-approve
terraform plan -detailed-exitcode -no-color > /tmp/plan2.txt
EXIT=$?
# detailed-exitcode: 0 = no changes, 2 = changes present
[[ $EXIT -eq 0 ]] || { echo "FAIL: re-apply shows changes"; cat /tmp/plan2.txt; exit 1; }
echo "PASS: re-apply is idempotent"

# AC #41 — plan output is reproducible.
terraform plan -out=/tmp/plan-a.tfplan
terraform plan -out=/tmp/plan-b.tfplan
JSON_A=$(terraform show -json /tmp/plan-a.tfplan | jq 'del(.timestamp)')
JSON_B=$(terraform show -json /tmp/plan-b.tfplan | jq 'del(.timestamp)')
diff <(echo "$JSON_A") <(echo "$JSON_B") && echo "PASS: plan is reproducible"
```

### Security-group inspection — AC #9, #37

```python
#!/usr/bin/env python3
# tests/sg-egress-rule-check.py
"""Asserts the Fargate SG has no 0.0.0.0/0 egress rule."""
import json, subprocess, sys

state = json.loads(subprocess.check_output(["terraform", "show", "-json"]))
sgs = [r for r in state["values"]["root_module"]["resources"]
       if r["type"] == "aws_security_group" and "chat_fargate" in r["name"]]
assert len(sgs) == 1, f"expected 1 Fargate SG, got {len(sgs)}"
egress = sgs[0]["values"]["egress"]
for rule in egress:
    assert rule.get("cidr_blocks") != ["0.0.0.0/0"], \
        f"Fargate SG has 0.0.0.0/0 egress rule: {rule}"
print("PASS: Fargate SG has no 0.0.0.0/0 egress")
```

### IAM policy simulation — AC #27, #28

```bash
#!/usr/bin/env bash
# tests/iam-task-role-simulation.sh
set -euo pipefail
TASK_ROLE_ARN=$(terraform output -raw task_role_arn)
TENANT_SECRET_ARN=$(terraform output -raw rds_secret_arn)
OTHER_SECRET_ARN="arn:aws:secretsmanager:ap-southeast-1:111111111111:secret:other-tenant"

# AC #28: own secret = ALLOW
result_own=$(aws iam simulate-principal-policy \
  --policy-source-arn "$TASK_ROLE_ARN" \
  --action-names secretsmanager:GetSecretValue \
  --resource-arns "$TENANT_SECRET_ARN" \
  --query 'EvaluationResults[0].EvalDecision' --output text)
[[ "$result_own" == "allowed" ]] || { echo "FAIL: own secret denied"; exit 1; }

# AC #28: other secret = DENY
result_other=$(aws iam simulate-principal-policy \
  --policy-source-arn "$TASK_ROLE_ARN" \
  --action-names secretsmanager:GetSecretValue \
  --resource-arns "$OTHER_SECRET_ARN" \
  --query 'EvaluationResults[0].EvalDecision' --output text)
[[ "$result_other" == "explicitDeny" || "$result_other" == "implicitDeny" ]] \
  || { echo "FAIL: other tenant's secret allowed"; exit 1; }

# AC #27: wildcard s3 = DENY
result_s3=$(aws iam simulate-principal-policy \
  --policy-source-arn "$TASK_ROLE_ARN" \
  --action-names s3:GetObject \
  --resource-arns "arn:aws:s3:::any-bucket/*" \
  --query 'EvaluationResults[0].EvalDecision' --output text)
[[ "$result_s3" != "allowed" ]] || { echo "FAIL: s3 wildcard allowed"; exit 1; }
echo "PASS: IAM least-privilege verified"
```

### Weekly PITR test — AC #17, #23

```yaml
# .github/workflows/chat-pitr-weekly.yml
name: CHAT PITR weekly test
on:
  schedule: [{ cron: '0 17 * * 0' }]  # Sunday 17:00 UTC = Monday 00:00 ICT
jobs:
  pitr-restore:
    strategy:
      matrix:
        tier: [trial, standard, premium]
    steps:
      - name: Restore last week's snapshot to sentinel cluster
        run: |
          aws rds restore-db-instance-to-point-in-time \
            --source-db-instance-identifier "cyberos-chat-sentinel-${{ matrix.tier }}" \
            --target-db-instance-identifier "pitr-test-${{ matrix.tier }}-$(date +%s)" \
            --restore-time "$(date -u -d '6 hours ago' +%FT%TZ)"
      - name: Assert row count matches pre-snapshot tag
        run: |
          EXPECTED=$(cat .pitr-sentinel-row-count-${{ matrix.tier }}.txt)
          ACTUAL=$(psql -h "$PITR_HOST" -t -c "SELECT COUNT(*) FROM posts" | tr -d ' ')
          [[ "$ACTUAL" -ge "$EXPECTED" ]] || exit 1
      - name: Emit memory audit chat.pitr_test_passed
        run: cyberos-memory-cli emit --kind chat.pitr_test_passed --payload '...'
      - name: Teardown sentinel restore
        if: always()
        run: aws rds delete-db-instance --db-instance-identifier "pitr-test-${{ matrix.tier }}-..."
```

### LocalStack apply test — AC #1-13, #18, #21, #29

```bash
#!/usr/bin/env bash
# tests/localstack-apply.sh
set -euo pipefail
export AWS_ACCESS_KEY_ID=test AWS_SECRET_ACCESS_KEY=test AWS_DEFAULT_REGION=ap-southeast-1
export TF_VAR_aws_endpoint_url=http://localhost:4566

docker run -d --name localstack -p 4566:4566 \
  -e SERVICES=ecs,rds,elasticache,iam,secretsmanager,kms,logs,cloudwatch,acm,route53,elbv2,ec2 \
  localstack/localstack-pro:latest
sleep 30

cd infra/terraform/modules/tenant_chat
terraform init -backend=false
terraform apply -auto-approve -var '...'

# Smoke checks via aws CLI (against LocalStack).
aws --endpoint-url=$TF_VAR_aws_endpoint_url ecs describe-clusters \
    --clusters "cyberos-chat-$TENANT" | jq '.clusters[0].status' | grep -q ACTIVE

aws --endpoint-url=$TF_VAR_aws_endpoint_url rds describe-db-instances \
    --db-instance-identifier "cyberos-chat-$TENANT" \
    | jq '.DBInstances[0].StorageEncrypted' | grep -q true

aws --endpoint-url=$TF_VAR_aws_endpoint_url elasticache describe-replication-groups \
    --replication-group-id "cyberos-chat-$TENANT" \
    | jq '.ReplicationGroups[0].AtRestEncryptionEnabled' | grep -q true

# AC #29: memory row emitted.
cyberos-memory-cli query --kind chat.deployment_inventory \
    --tenant-id "$TENANT" --limit 1 \
    | jq '.payload.ecs_service_arn' | grep -q "cyberos-chat-$TENANT"

terraform destroy -auto-approve
docker stop localstack && docker rm localstack
```

### ALB TLS-policy assertion — AC #18

```bash
#!/usr/bin/env bash
# tests/alb-tls-policy.sh
ALB_ARN=$(terraform output -raw alb_arn)
LISTENER_ARN=$(aws elbv2 describe-listeners --load-balancer-arn "$ALB_ARN" \
    --query 'Listeners[?Port==`443`].ListenerArn' --output text)
POLICY=$(aws elbv2 describe-listeners --listener-arns "$LISTENER_ARN" \
    --query 'Listeners[0].SslPolicy' --output text)
[[ "$POLICY" == "ELBSecurityPolicy-TLS13-1-2-2021-06" ]] \
  || { echo "FAIL: TLS policy $POLICY"; exit 1; }
```

### Example invocation

```hcl
# infra/terraform/examples/single-tenant-chat/main.tf
module "tenant_chat" {
  source         = "../../modules/tenant_chat"
  tenant_id      = "00000000-0000-0000-0000-000000000001"
  tier           = "standard"
  aws_region     = "ap-southeast-1"
  data_residency = "vn-only"
  vpc_id         = "vpc-0abc"
  kms_cmk_arn    = "arn:aws:kms:ap-southeast-1:111111111111:key/abc"
  pinned_image_tag = "cyberos/chat@sha256:abc1230000000000000000000000000000000000000000000000000000000000"
  cyberos_apex_domain     = "cyberskill.world"
  cyberos_route53_zone_id = "Z0987654321"
  auth_jwks_vpc_endpoint_service_name = "com.amazonaws.vpce.ap-southeast-1.vpce-svc-0xyz"
  memory_writer_socket_endpoint = "tcp://memory-writer.demo.svc.cluster.local:9090"
  auth_jwks_url  = "https://auth.internal.cyberos.world/.well-known/jwks.json"
  sns_sev1_topic_arn = "arn:aws:sns:ap-southeast-1:111111111111:sev1"
  sns_sev2_topic_arn = "arn:aws:sns:ap-southeast-1:111111111111:sev2"
}
```

---

## §6 — Implementation skeleton

The Terraform module above is the implementation skeleton. This section names the operational wiring decisions that don't live in any single `.tf` file:

### §6.1 — State backend convention

```hcl
# Caller's terraform { backend "s3" {...} } block:
terraform {
  required_version = ">= 1.7.0"
  backend "s3" {
    bucket         = "cyberos-terraform-state"
    key            = "${data.aws_caller_identity.current.account_id}/${var.tenant_id}/chat/terraform.tfstate"
    region         = "ap-southeast-1"
    encrypt        = true
    kms_key_id     = "arn:aws:kms:ap-southeast-1:000000000000:alias/cyberos-tfstate"
    dynamodb_table = "cyberos-terraform-lock"
  }
}
```

The lock table is shared across all tenants because DynamoDB locks are keyed by LockID (the state path), so cross-tenant lock contention is impossible.

### §6.2 — Tier-upgrade runbook

In-place tier upgrade (trial → standard, standard → premium) requires careful sequencing because RDS Multi-AZ migration and Redis cluster-mode changes both incur downtime:

| Step | Action | Notes |
|---|---|---|
| 1 | `terraform plan -var tier=standard` | Inspect diff |
| 2 | Schedule maintenance window | Notify tenant ≥ 24h prior |
| 3 | Manually trigger pre-upgrade snapshot | Belt + suspenders over RDS automated backup |
| 4 | `terraform apply -var tier=standard` | RDS migrates first (~30 min downtime if t4g.micro→small); Redis migrates second (~10 min) |
| 5 | Smoke test: ALB returns 200 from `/api/v4/system/ping` | Confirm app is up |
| 6 | Emit memory audit `chat.tier_upgraded` | Operator-driven, not Terraform-driven |
| 7 | Update tenant `cyberos.tenants[tenant_id].tier` in memory | Source of truth for billing |

Premium → standard or standard → trial DOWNGRADE is operator-gated; running `terraform plan` shows resource destruction (RDS Multi-AZ replica deleted, Redis shards removed) which would lose redundancy. Downgrade workflow goes through a separate runbook.

### §6.3 — Drift-detection wiring

A scheduled GitHub Action runs `terraform plan -var dry_run=true` every 6h per tenant. If exit code is 2 (changes present), it:

1. Writes the plan output to S3 (`s3://cyberos-drift-reports/<tenant>/<timestamp>.txt`).
2. Emits memory audit `chat.deployment_drift_detected` with the diff summary.
3. Opens a Slack alert via `cyberos-slack-webhook`.

The drift is human-investigated, not auto-remediated, because some drift is legitimate (operator hot-fix; AWS region maintenance; AWS service-default change).

### §6.4 — Image-promotion workflow

The `pinned_image_tag` variable is the only knob for image updates. The CI workflow that produces a new image commits the new digest to a per-tenant `tfvars` file:

```yaml
# .github/workflows/chat-image-promote.yml
on:
  push: { branches: [main], paths: ['services/chat/Dockerfile'] }
jobs:
  build-and-tag:
    steps:
      - name: Build + push image
        run: |
          docker build -t cyberos/chat .
          DIGEST=$(docker push cyberos/chat 2>&1 | grep -oE 'sha256:[a-f0-9]{64}')
          echo "DIGEST=$DIGEST" >> $GITHUB_ENV
      - name: Update per-tenant tfvars (canary tenant first)
        run: |
          sed -i "s|pinned_image_tag.*|pinned_image_tag = \"cyberos/chat@${DIGEST}\"|" \
            infra/terraform/envs/canary/chat.tfvars
      - name: Open PR
        run: gh pr create --title "Promote chat image $DIGEST to canary"
```

Promotion to standard/premium fleets only after the canary tenant runs for 24h without alarms.

### §6.5 — KMS CMK lifecycle

The CMK is created OUTSIDE this module (per FR-AUTH-001 KMS-management FR). This module consumes the ARN. Why split: KMS CMK deletion has a 7–30 day pending window; we want it managed by a Terraform run that ONLY manages KMS so accidental tenant-stack destroy doesn't queue CMK deletion.

### §6.6 — Memory-writer socket reachability

The `memory_writer_socket_endpoint` is a private DNS name resolved via Route 53 private hosted zone. The module does NOT provision the memory writer (FR-MEMORY-101 owns that). Caller is responsible for ensuring the memory writer is reachable from the Fargate task subnet. Validation: a post-apply `local-exec` script tries `nc -z` against the socket; failure → memory audit `chat.deployment_warning` with `reason=memory_writer_unreachable`.

### §6.7 — Cross-tenant backup isolation

Each tenant's RDS snapshots use the tenant CMK. A snapshot from tenant A cannot be restored into tenant B's account because tenant B's IAM role doesn't have decrypt permission on tenant A's CMK. This is the cryptographic enforcement of tenant isolation at the backup layer.

### §6.8 — Apply-time dependency ordering

Terraform's implicit dependency graph handles most ordering, but two cases need explicit `depends_on`:

1. `aws_ecs_service.chat` MUST come after `aws_db_instance.chat` is `available` (otherwise the Mattermost boot tries to connect to RDS that's still initialising). Force via `depends_on = [aws_db_instance.chat]` even though no direct reference exists.
2. `null_resource.deployment_audit` MUST come last; force via `depends_on = [aws_ecs_service.chat, aws_cloudwatch_metric_alarm.alb_5xx, ...]`.

### §6.9 — Module versioning + breaking-change policy

`local.module_version` follows semver. Breaking changes (variable renames, resource removals) bump major; non-breaking additions bump minor; bug fixes bump patch. Callers pin to a major via `source = "git::ssh://...//modules/tenant_chat?ref=v1.x"`. Major bumps require a coordinated migration FR.

### §6.10 — Apply-time secret seeding

The Mattermost boot needs a database that's already initialised with the required schema. Two paths:

1. **Cold start (new tenant):** Mattermost boots, sees empty DB, runs its own migrations against the master Postgres connection. The Secrets Manager rotation Lambda was pre-seeded with the master password at RDS-create time.
2. **Restored from snapshot:** Schema is already present; Mattermost boots and skips migrations.

In both paths, the Fargate task fetches the password via Secrets Manager at boot and only logs `MM-DB-CONNECTED` to CloudWatch.

---

## §7 — Dependencies

- **FR-CHAT-001** — image source.
- **FR-CHAT-002** — authbridge plugin baked into image.
- **FR-CHAT-004 (downstream)** — search uses RDS.
- **FR-CHAT-005 (downstream)** — memory bridge uses Postgres logical replication.
- **FR-OBS-001** — collector sidecar.
- **FR-AUTH-004** — JWKS source.
- **FR-MEMORY-101** — memory writer socket.

---

## §8 — Example payloads

### `chat.deployment_provisioned` — post-apply success

```json
{
  "kind": "chat.deployment_provisioned",
  "ts_ns": 1747407137483000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "trace_id": "9a3b8d4c7e2f1029384756abcdef0123",
  "payload": {
    "tier": "standard",
    "aws_region": "ap-southeast-1",
    "data_residency": "vn-only",
    "chat_url": "https://t-1f8c4d6e.cyberskill.world",
    "ecs_cluster_arn": "arn:aws:ecs:ap-southeast-1:111111111111:cluster/cyberos-chat-1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
    "rds_endpoint": "<redacted>",
    "redis_endpoint": "<redacted>",
    "module_version": "1.0.0",
    "terraform_run_id": "default",
    "pinned_image_tag": "cyberos/chat@sha256:abc1230000000000000000000000000000000000000000000000000000000000",
    "provisioned_at_ns": 1747407137483000000
  }
}
```

### `chat.deployment_inventory` — full resource enumeration

```json
{
  "kind": "chat.deployment_inventory",
  "ts_ns": 1747407137485000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "ecs_service_arn":   "arn:aws:ecs:ap-southeast-1:111111111111:service/cyberos-chat-.../chat",
    "rds_instance_arn":  "arn:aws:rds:ap-southeast-1:111111111111:db:cyberos-chat-1f8c4d6e-...",
    "redis_cluster_arn": "arn:aws:elasticache:ap-southeast-1:111111111111:replicationgroup:cyberos-chat-1f8c4d6e-...",
    "alb_arn":           "arn:aws:elasticloadbalancing:ap-southeast-1:111111111111:loadbalancer/app/cyberos-chat-1f8c4d6e/abc",
    "log_group":         "/cyberos/chat/1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
    "task_role_arn":     "arn:aws:iam::111111111111:role/cyberos-chat-task-1f8c4d6e-...",
    "kms_cmk_arn":       "arn:aws:kms:ap-southeast-1:111111111111:key/abc",
    "security_groups": [
      "sg-0aaa", "sg-0bbb", "sg-0ccc", "sg-0ddd"
    ],
    "vpc_endpoint_ids": [
      "vpce-0aaa", "vpce-0bbb", "vpce-0ccc", "vpce-0ddd", "vpce-0eee"
    ],
    "cloudwatch_alarm_arns": [
      "arn:aws:cloudwatch:ap-southeast-1:111111111111:alarm:cyberos-chat-...-rds-cpu",
      "arn:aws:cloudwatch:ap-southeast-1:111111111111:alarm:cyberos-chat-...-rds-storage",
      "arn:aws:cloudwatch:ap-southeast-1:111111111111:alarm:cyberos-chat-...-rds-replica-lag",
      "arn:aws:cloudwatch:ap-southeast-1:111111111111:alarm:cyberos-chat-...-fargate-crashloop",
      "arn:aws:cloudwatch:ap-southeast-1:111111111111:alarm:cyberos-chat-...-fargate-memory",
      "arn:aws:cloudwatch:ap-southeast-1:111111111111:alarm:cyberos-chat-...-redis-evictions",
      "arn:aws:cloudwatch:ap-southeast-1:111111111111:alarm:cyberos-chat-...-alb-5xx"
    ],
    "module_version": "1.0.0",
    "terraform_apply_duration_seconds": 412
  }
}
```

### `chat.deployment_drift_detected` — scheduled drift check

```json
{
  "kind": "chat.deployment_drift_detected",
  "ts_ns": 1747407237000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "drift_summary": "1 resource changed outside Terraform: aws_db_parameter_group.chat (parameter log_min_duration_statement: 100 → 500)",
    "detected_via": "scheduled-plan",
    "plan_report_s3_url": "s3://cyberos-drift-reports/1f8c4d6e-.../2026-05-16T13:00Z.txt",
    "severity": "warning",
    "auto_remediated": false
  }
}
```

### `chat.tier_upgraded` — operator-driven tier change

```json
{
  "kind": "chat.tier_upgraded",
  "ts_ns": 1747407237100000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "from_tier": "trial",
    "to_tier": "standard",
    "operator_email": "ops@cyberskill.world",
    "downtime_seconds": 1842,
    "snapshot_before_arn": "arn:aws:rds:...:snapshot:pre-upgrade-trial-to-standard-...",
    "module_version": "1.0.0"
  }
}
```

### `chat.pitr_test_passed` — weekly PITR validation

```json
{
  "kind": "chat.pitr_test_passed",
  "ts_ns": 1747407300000000000,
  "tenant_id": "sentinel-standard",
  "payload": {
    "tier": "standard",
    "restored_snapshot_arn": "arn:aws:rds:...:snapshot:cyberos-chat-sentinel-2026-05-16",
    "restore_duration_seconds": 245,
    "row_count_pre":  104238,
    "row_count_post": 104238,
    "rds_engine_version": "16.3"
  }
}
```

### `chat.deployment_warning` — non-fatal post-apply check

```json
{
  "kind": "chat.deployment_warning",
  "ts_ns": 1747407137600000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "reason": "memory_writer_unreachable",
    "endpoint_tested": "tcp://memory-writer.demo.svc.cluster.local:9090",
    "operator_action": "verify FR-MEMORY-101 socket is up and tenant subnet has route"
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Multi-region active-active deployment — slice 4+.
- Auto-tier-upgrade based on usage — slice 4+.
- Spot instances for trial tier (cost) — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Apply fails mid-resource | Terraform state | Partial state; operator runs apply again | Idempotent retry; investigate `terraform refresh` if state drifted |
| RDS Multi-AZ upgrade in-place | downtime ~30min during failover | scheduled maintenance window | Notify tenant ≥24h prior; emit `chat.maintenance_started` audit |
| Redis cluster mode change | requires replace; ~10min | Replication group destroy + create | Operator schedules off-hours; preserve via `final_snapshot` flag |
| CMK deletion (operator error) | resources unreadable; CloudTrail `ScheduleKeyDeletion` event | Tenant data lost permanently after 7d window | Sev-1; cancel deletion via `CancelKeyDeletion` within 7d window |
| RDS quota limit hit | `LimitExceededException` from CreateDBInstance | Apply fails | Operator requests quota via AWS Support; track via `chat.deployment_blocked` audit |
| Fargate task crash loop | `fargate_crash_loop` CloudWatch alarm | Sev-1 page | Operator reads task stop reason; rolls back image digest |
| ALB target unhealthy on /api/v4/system/ping | target group health metric | New tasks fail to register; existing serve traffic | Investigate Mattermost startup logs; common cause: DB unreachable |
| RDS CPU > 80% sustained | `rds_cpu` CloudWatch alarm | Sev-2 | Scale up tier OR identify query causing CPU via FR-OBS-005 |
| Redis memory pressure | `redis_evictions` alarm | Sev-2; LRU evicts cold sessions | Scale up tier OR reduce session TTL via Mattermost setting |
| Cross-AZ replication lag > 30s | `rds_replica_lag` alarm | Sev-2; reads may be stale | Investigate WAL throughput; consider larger instance class |
| Plugin (FR-CHAT-002) hot-reload triggers Fargate task replace | task definition revision bumped | Rolling restart; <30s downtime per task | None — ALB drains |
| VPC endpoint policy too restrictive | Fargate task can't pull image; ECR API returns 403 | Sev-2; task fails to start | Operator inspects endpoint policy; widen to `*` for ECR principals in tenant account |
| KMS key rotation (automatic AWS-managed CMK rotation) | re-encrypts in place | None visible | Automatic; CloudTrail event for audit |
| Customer-managed CMK rotation | KMS rotation triggers re-encryption | RDS storage re-encrypts in background; no downtime | None — automatic |
| Terraform state file corruption | apply returns "state not parseable" | Apply fails | Restore from S3 versioning; emit `chat.tf_state_recovered` audit |
| Terraform state lock leak (apply killed mid-flight) | next apply blocks on DynamoDB lock | Apply hangs | Manually release lock: `terraform force-unlock <id>` |
| Cross-account PrivateLink endpoint not approved | `chat_authbridge` plugin returns `jwks_unavailable` | Logins fail | Operator approves endpoint connection in AUTH-004 account |
| ACM cert validation pending (Route53 record propagation) | apply times out at `aws_acm_certificate_validation` | First apply takes longer; subsequent applies skip | Wait 5-10min for DNS propagation; re-apply |
| Secrets Manager rotation Lambda fails | `secretsmanager:RotateSecret` event with `Failed` | Password not rotated; Mattermost keeps using old | Sev-2; operator inspects Lambda logs |
| RDS snapshot space exhausted (premium tier) | `rds_free_storage` alarm | Sev-2; backups may fail | Auto-scaling storage enabled at premium; operator triggers manual scale |
| ALB 5xx spike from Mattermost | `alb_5xx` alarm | Sev-1 | Investigate via CloudWatch logs; common: DB connection pool exhausted |
| memory writer socket unreachable from Fargate | post-apply `null_resource.deployment_audit` `local-exec` fails | Audit row not emitted; warning row emitted | Operator verifies FR-MEMORY-101 socket up + VPC route exists |
| Drift detector finds out-of-band change | scheduled `terraform plan` exit-code 2 | `chat.deployment_drift_detected` audit | Operator investigates; either revert or codify into Terraform |
| Tenant requests data-residency change (vn-only → global) | `terraform plan` shows region change → cluster destroy | Requires data-migration workflow; not in-place | Migration FR (deferred) |
| Module-version bump introduces breaking diff | `terraform plan` shows unexpected destroy/create | Apply fails preflight check | Operator pins to older major; coordinates migration FR |
| Two operators apply concurrently | DynamoDB lock contention | One blocks until other completes | None — Terraform handles |
| Image digest in tfvars typo'd | variable validation regex rejects | Plan fails | Operator fixes |
| Pin to wrong arch (`linux/amd64` digest on Graviton) | ECS task start returns `EssentialContainerExited` | Sev-1 crash loop | Operator pins arch-matched digest |
| AWS regional outage (ap-southeast-1) | apply errors with retries | Sev-1 | Wait out outage; consider multi-region failover (slice 4+) |
| LocalStack apply skips behaviour that real AWS rejects | tests pass; real apply fails | Caught in canary tenant before fleet | Promote slowly: canary → staging → prod |
| Route53 hosted zone disabled mid-apply | `aws_route53_record.chat` fails | Apply fails; tenant URL not resolvable | Operator restores hosted zone |
| Logical replication slot exhausted (FR-CHAT-005 consumer) | `max_replication_slots` hit | New consumers can't subscribe | Bump param group; restart RDS |
| Lambda rotation IAM permission revoked | rotation fails silently | Password stays static | Sev-3 if undetected for >30d (next rotation due); detected via `last_rotated_date` CloudWatch metric |
| Module v1.x deprecation announcement | release notes + memory audit `chat.module_deprecated` | Operators get 90d notice | Migrate to v2.x per migration FR |

---

## §11 — Implementation notes

- Tier-aware specs use Terraform locals for readability; adding a tier = one map entry.
- Fargate Spot supported via separate `tier_trial_spot` variant — slice 4+. Spot is not safe for standard/premium because evictions cause WebSocket reconnect storms.
- Backup retention 7 days at trial (cost), 14 days at standard (SLA), 30 days at premium (compliance). Longer retention via S3 export to Glacier — separate FR.
- Tags drive AWS Cost Allocation reports; finance team pivots on `Tenant` tag. The `CostCenter` tag is used for chargeback to the per-tenant invoice.
- Post-apply memory audit via `null_resource` provisioner is non-ideal (provisioners are Terraform's escape hatch, not its primary surface) but the AWS provider has no first-class "emit-after-apply" extension point. The alternative — Lambda triggered by CloudWatch Events on `terraform apply` completion — adds complexity without buying clarity. We'll revisit if Terraform 1.10+ ships native post-apply hooks.
- Auto-scaling policy: scale up at 70% CPU, scale down at 30%; cooldown 5min on both directions. The asymmetric thresholds avoid flapping. Scale-up cooldown is intentionally long enough to absorb a sudden burst (e.g. all-hands meeting login storm) without runaway scaling.
- Why ARM Graviton (db.t4g, db.m6g, cache.t4g, cache.r6g) instead of x86: 20-40% better price-performance for Postgres + Redis workloads (AWS published benchmarks; we replicated on a sandbox tenant). No application-side change required — Postgres + Redis are bit-identical across arches.
- The ALB drop_invalid_header_fields setting defends against HTTP header smuggling attacks (CVE class). Default is false; we explicitly enable.
- We chose application-cookie stickiness (`MMAUTHTOKEN`) over duration-based stickiness because Mattermost issues its own session cookie; aligning stickiness to that cookie gives a single source of truth for "which task should handle this user."
- The premium tier's Redis cluster mode (3 shards) is required for Mattermost installations >1k concurrent users. The Mattermost docs say "we recommend Redis Cluster for >500 users"; we set the threshold at premium tier (3k users) so smaller tenants don't pay for cluster overhead.
- Why we don't use Fargate task ECS managed service auto-scaling on the RDS connection count: ECS metric-based scaling reacts to instance metrics, not application metrics. We rely on CPU and memory; queue-depth-based scaling would require custom CloudWatch metric publication from inside Mattermost, which is out of scope.
- The custom RDS parameter group is required at apply time (not after-the-fact) because `shared_preload_libraries` is `apply_method = "pending-reboot"`. Forgetting it means a manual `RebootDBInstance` after FR-CHAT-004 ships pgroonga.
- `rds.logical_replication = 1` is required by FR-CHAT-005. We enable it at this FR (deployment) rather than at FR-CHAT-005 (the consumer) because changing the param requires a reboot — better to pay that cost once at provisioning than later when traffic exists.
- `max_replication_slots = 10` accommodates: 1 for FR-CHAT-005 memory replication, 1 for the read-replica, 8 spare for future consumers. Bumping this requires reboot, so we overprovision.
- Why a single shared lock table instead of per-tenant: DynamoDB pricing is per-table baseline + per-read. 100 tenants × per-tenant table = 100 × $0.10/mo baseline = $10/mo for zero functional gain (DynamoDB locks are keyed by LockID, not table). One shared table costs $0.10/mo total.
- The state-backend convention uses `<aws-account>/<tenant_id>/...` so that one Terraform state file per tenant is the natural unit. If a tenant ever spans multiple AWS accounts (e.g. compliance carve-out), the account prefix scales naturally.
- ACM certificate uses `create_before_destroy` lifecycle to avoid downtime during cert renewal — new cert provisions, ALB attaches to it, old cert deletes after.
- ALB `enable_deletion_protection` is conditional on premium tier because trial tenants get torn down regularly; the protection would be operator friction.
- We chose ALB over NLB because ALB's HTTP-aware health checks (`/api/v4/system/ping`) catch issues NLB's TCP probe misses (e.g. Mattermost server running but DB unreachable returns 503).
- The `auth_jwks_vpc_endpoint_service_name` variable is required (not optional default) because the PrivateLink endpoint service name is account-specific; defaulting would mask misconfiguration.
- A `data_residency=vn-only` tenant doesn't get a Route 53 record in the global zone — they get one in a VN-region private hosted zone. This is enforced via the cross-validation in `main.tf` precondition; downstream FR will add the actual VN private zone wiring (slice 2).
- We don't auto-tier-upgrade based on usage (deferred to slice 4+) because tier changes incur downtime; auto-upgrade in production is unsafe without operator coordination.
- The `dry_run` variable is intentionally minimal — it doesn't gate `terraform apply`, it just emits a different audit row when set. The drift detector uses it as a marker; humans MUST still be in the loop for actual applies.
- The reproducible-plan check (AC #41) is the most important guard against subtle module-version regressions — if the same inputs ever produced different plans, downstream automation would silently change behaviour across runs.
- Why we removed `valueFrom` from environment variables and put the secret reference in the task definition `secrets` array: Mattermost reads `MM_SQLSETTINGS_DATASOURCE` from environment, but ECS task definitions can populate environment from secrets via the `secrets` array. This keeps the secret out of the task-definition JSON in plaintext.
- Why an `app_cookie` stickiness with the Mattermost session cookie instead of `lb_cookie` (ALB-issued): with `lb_cookie`, a user who logs out + back in gets routed to the same task — fine for affinity but means a flapping task takes down a user's session. With `app_cookie` tied to the auth token, logout invalidates the affinity along with the session.

---

*End of FR-CHAT-003.*
