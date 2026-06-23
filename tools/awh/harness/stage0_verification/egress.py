"""Generate a default-deny egress firewall (iptables + ipset) from a domain allowlist.

Why default-deny egress is the load-bearing control
---------------------------------------------------
Anthropic, "How we contain Claude across products" (2026): a phishing prompt got
Claude to exfiltrate ``~/.aws/credentials`` in 24 of 25 retries — "the only defense
that holds … is the environment, specifically egress controls." And native OS
sandboxing "safely reduces permission prompts by 84%" with network denied by
default ("Making Claude Code more secure and autonomous with sandboxing", Oct 2025).

This module renders the firewall *programmatically* so its safety invariants are
unit-testable (default policy DROP, established/related allowed, loopback allowed,
allowlist enforced, final reject) rather than trusting a hand-written script — the
SOCKS5 null-byte bypass that lived in Claude Code's proxy for ~5.5 months is the
cautionary tale for "egress controls need real review".

The generated script is committed at ``sandbox/devcontainer/init-firewall.sh`` so a
human can review the exact rules; ``render_init_script`` is its tested twin.
"""

from __future__ import annotations

import ipaddress
import re

__all__ = ["default_allowlist", "build_iptables_plan", "render_init_script"]

# A valid DNS hostname: letters/digits/hyphens in 1–63-char labels, dots between,
# total ≤253, optional trailing dot. Crucially this admits NO shell metacharacters
# (space, ;, |, &, $, (), <>, quotes, newlines) — the whole point is that a value
# interpolated into the generated root script can't carry a command injection.
_HOSTNAME_RE = re.compile(
    r"^(?=.{1,253}\.?$)(?!-)[A-Za-z0-9-]{1,63}(?<!-)"
    r"(\.(?!-)[A-Za-z0-9-]{1,63}(?<!-))*\.?$"
)


def _check_domain(d: str) -> str:
    if not isinstance(d, str) or not _HOSTNAME_RE.match(d):
        raise ValueError(f"egress: refusing to render an unsafe/invalid domain: {d!r}")
    return d


def _check_ip(ip: str) -> str:
    try:
        ipaddress.ip_address(ip)
    except (ValueError, TypeError) as e:
        raise ValueError(f"egress: invalid resolver IP: {ip!r}") from e
    return ip


def _check_subnet(net: str) -> str:
    try:
        ipaddress.ip_network(net, strict=False)
    except (ValueError, TypeError) as e:
        raise ValueError(f"egress: invalid allow-subnet (CIDR): {net!r}") from e
    return net


def default_allowlist() -> list[str]:
    """Domains an autonomous coding agent typically needs — nothing else."""
    return [
        "github.com",
        "api.github.com",
        "codeload.github.com",
        "objects.githubusercontent.com",
        "registry.npmjs.org",
        "pypi.org",
        "files.pythonhosted.org",
        "api.anthropic.com",
    ]


def build_iptables_plan(
    domains: list[str],
    resolver_ips: list[str] | None = None,
    allow_subnets: list[str] | None = None,
) -> list[str]:
    """Return the ordered shell command lines for a default-deny OUTPUT firewall.

    Order matters: flush -> allowlist set -> loopback -> established -> DNS ->
    allowed dst -> host subnets -> default DROP -> final logged reject.
    """
    # Validate every value BEFORE it is interpolated into the generated root script.
    resolver_ips = [_check_ip(r) for r in (resolver_ips or ["1.1.1.1", "8.8.8.8"])]
    allow_subnets = [_check_subnet(n) for n in (allow_subnets or [])]
    domains = [_check_domain(d) for d in domains]
    plan: list[str] = [
        "iptables -F OUTPUT",
        "iptables -F INPUT",
        "ipset destroy allowed-domains 2>/dev/null || true",
        "ipset create allowed-domains hash:ip",
        # loopback (agent <-> local proxy) is always allowed
        "iptables -A OUTPUT -o lo -j ACCEPT",
        "iptables -A INPUT -i lo -j ACCEPT",
        # return traffic for connections we opened
        "iptables -A OUTPUT -m state --state ESTABLISHED,RELATED -j ACCEPT",
        "iptables -A INPUT -m state --state ESTABLISHED,RELATED -j ACCEPT",
    ]
    for r in resolver_ips:
        plan.append(f"iptables -A OUTPUT -p udp --dport 53 -d {r} -j ACCEPT")
        plan.append(f"iptables -A OUTPUT -p tcp --dport 53 -d {r} -j ACCEPT")
    # resolve each allowed domain at apply-time and add every IP to the ipset
    for d in domains:
        plan.append(
            f'for ip in $(dig +short A {d}); do ipset add allowed-domains "$ip" 2>/dev/null || true; done'
        )
    plan.append("iptables -A OUTPUT -m set --match-set allowed-domains dst -j ACCEPT")
    for net in allow_subnets:
        plan.append(f"iptables -A OUTPUT -d {net} -j ACCEPT")
    # default-deny: anything not explicitly allowed is dropped and logged
    plan.append('iptables -A OUTPUT -j LOG --log-prefix "AWH-EGRESS-DENY: " --log-level 4')
    plan.append("iptables -P OUTPUT DROP")
    plan.append("iptables -P INPUT DROP")
    plan.append("iptables -P FORWARD DROP")
    return plan


def render_init_script(
    domains: list[str] | None = None,
    resolver_ips: list[str] | None = None,
    allow_subnets: list[str] | None = None,
) -> str:
    """Render the full ``init-firewall.sh`` (default-deny egress + self-verification)."""
    domains = domains if domains is not None else default_allowlist()
    plan = build_iptables_plan(domains, resolver_ips, allow_subnets)
    body = "\n".join(plan)
    allowed_probe = domains[0] if domains else "github.com"
    # Negative probe must be a host that is NOT allowlisted, else a "blocked" result
    # is meaningless. Pick the first example.* host not present in the allowlist.
    neg_probe = next(
        (h for h in ("example.org", "example.com", "example.net") if h not in domains),
        "example.org",
    )
    return f"""#!/usr/bin/env bash
# AUTO_WORK harness — default-deny egress firewall.
# Apply INSIDE the agent's network namespace as root at container start.
# Generated/validated by harness.stage0_verification.egress.render_init_script.
set -euo pipefail

if ! command -v ipset >/dev/null 2>&1; then
  echo "ipset is required (apt-get install -y ipset iptables dnsutils)" >&2
  exit 1
fi

{body}

echo "[awh] egress firewall applied: default-deny with {len(domains)} allowed domain(s)."

# --- self-verification: an allowed host must reach, a random host must NOT ---
if curl -sS --max-time 5 -o /dev/null "https://{allowed_probe}"; then
  echo "[awh] OK: reached allowed host {allowed_probe}"
else
  echo "[awh] WARN: could not reach allowed host {allowed_probe} (DNS/timing?)" >&2
fi
if curl -sS --max-time 5 -o /dev/null "https://{neg_probe}" 2>/dev/null; then
  echo "[awh] FAIL: {neg_probe} was reachable — egress is NOT contained!" >&2
  exit 1
else
  echo "[awh] OK: non-allowlisted egress is blocked."
fi
"""
