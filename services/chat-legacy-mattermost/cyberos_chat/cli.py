"""Small operator CLI for CHAT control-plane helpers."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from .deployment import TenantDeploymentSpec, build_deployment_plan
from .importers import import_zalo_bundle


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="cyberos-chat")
    sub = parser.add_subparsers(dest="cmd", required=True)

    deploy = sub.add_parser("plan-deploy")
    deploy.add_argument("--tenant-id", required=True)
    deploy.add_argument("--region", required=True)
    deploy.add_argument("--image", required=True)
    deploy.add_argument("--auth-jwks-url", required=True)

    imp = sub.add_parser("import-zalo")
    imp.add_argument("bundle", type=Path)

    ns = parser.parse_args(argv)
    if ns.cmd == "plan-deploy":
        plan = build_deployment_plan(
            TenantDeploymentSpec(
                tenant_id=ns.tenant_id,
                region=ns.region,
                image=ns.image,
                auth_jwks_url=ns.auth_jwks_url,
            )
        )
        print(json.dumps(plan.__dict__, default=str, sort_keys=True))
        return 0
    if ns.cmd == "import-zalo":
        rows = import_zalo_bundle(ns.bundle)
        print(json.dumps([row.__dict__ for row in rows], ensure_ascii=False, sort_keys=True))
        return 0
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
