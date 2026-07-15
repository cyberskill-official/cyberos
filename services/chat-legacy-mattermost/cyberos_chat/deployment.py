"""TASK-CHAT-003 — per-tenant deployment plan validation."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any


ALLOWED_REGIONS = {"sg-1", "vn-1", "eu-1", "us-1"}
MIN_CPU_UNITS = 512
MIN_MEMORY_MIB = 1024


@dataclass(frozen=True)
class TenantDeploymentSpec:
    tenant_id: str
    region: str
    image: str
    cpu_units: int = 1024
    memory_mib: int = 2048
    desired_count: int = 2
    rds_multi_az: bool = True
    redis_enabled: bool = True
    auth_jwks_url: str = ""


@dataclass(frozen=True)
class DeploymentPlan:
    service_name: str
    fargate: dict[str, Any]
    rds: dict[str, Any]
    redis: dict[str, Any]
    environment: dict[str, str]
    healthcheck_path: str = "/api/v4/system/ping"


def build_deployment_plan(spec: TenantDeploymentSpec) -> DeploymentPlan:
    """Build a deterministic Fargate/RDS/Redis deployment plan.

    The function is intentionally cloud-SDK free so CI can validate deploy
    semantics offline before Terraform or CDK consumes the resulting dict.
    """
    if not spec.tenant_id.strip():
        raise ValueError("tenant_id is required")
    if spec.region not in ALLOWED_REGIONS:
        raise ValueError(f"unsupported region: {spec.region}")
    if spec.cpu_units < MIN_CPU_UNITS:
        raise ValueError("cpu_units below CHAT minimum")
    if spec.memory_mib < MIN_MEMORY_MIB:
        raise ValueError("memory_mib below CHAT minimum")
    if spec.desired_count < 2:
        raise ValueError("desired_count must be >= 2 for rolling deploys")
    if not spec.rds_multi_az:
        raise ValueError("RDS Multi-AZ is required")
    if not spec.redis_enabled:
        raise ValueError("Redis is required for websocket fan-out")
    if not spec.auth_jwks_url.startswith("https://"):
        raise ValueError("auth_jwks_url must be HTTPS")

    service_name = f"cyberos-chat-{spec.tenant_id}-{spec.region}"
    return DeploymentPlan(
        service_name=service_name,
        fargate={
            "cluster": f"cyberos-{spec.region}",
            "task_family": service_name,
            "image": spec.image,
            "cpu": spec.cpu_units,
            "memory": spec.memory_mib,
            "desired_count": spec.desired_count,
            "platform_version": "1.4.0",
        },
        rds={
            "engine": "postgres",
            "multi_az": True,
            "storage_encrypted": True,
            "parameter_groups": ["pgroonga"],
        },
        redis={
            "engine": "redis",
            "transit_encryption": True,
            "at_rest_encryption": True,
        },
        environment={
            "CYBEROS_TENANT_ID": spec.tenant_id,
            "CYBEROS_REGION": spec.region,
            "CYBEROS_AUTH_JWKS_URL": spec.auth_jwks_url,
            "MM_PLUGINSETTINGS_ENABLEUPLOADS": "true",
        },
    )
