"""
Canonical-JSON encoding properties for the new AuditRecord schema.

Background
----------
The audit report's "msgspec ≡ RFC 8785 JCS byte-identical" claim was scoped
to a *closed* schema with no large integers. The legacy writer stored
timestamps as ISO-8601 strings ("ts"), so rfc8785's safe-integer domain
(±2^53) never came into play. The new schema in :mod:`cyberos.core.writer`
uses ``ts_ns`` as a u64 in nanoseconds — current time in ns is ≈1.7×10^18,
well beyond JSON's safe-integer domain, which rfc8785 strictly enforces.

That is a *schema* change, not an encoding regression. The migration is
still safe because:

* **Legacy records** — their stored ``chain`` field is copied verbatim by
  :mod:`runtime.tools.cyberos_migrate_v2`; we never recompute their hash.
  The legacy chain was computed over the legacy schema (with "ts" as ISO
  string) using rfc8785; that history stays valid as-is.
* **New records** — chained via msgspec canonical JSON; rfc8785 is not in
  the new code path at all.

What we actually need to verify is therefore two narrower properties:

1. ``msgspec`` produces deterministic, round-trip-stable canonical JSON
   for AuditRecords — required so the chain hash is reproducible across
   readers and writers running on different machines.
2. Within JSON's safe-integer domain (the legacy domain), msgspec matches
   rfc8785 byte-for-byte — required so legacy chains can be re-verified
   against the new code if needed during incident response.
"""

from __future__ import annotations

import json
import random

import pytest

from cyberos.core.writer import AuditRecord, _canonical, _chain_hash


# RFC 8785 JCS strictly enforces JSON's safe-integer domain: |n| < 2^53.
# 2^52 leaves a bit of headroom for any off-by-one edge in the spec.
_JSON_SAFE_INT_MAX = (1 << 52) - 1


def _random_record(rng: random.Random, *, safe_int: bool = True) -> AuditRecord:
    int_max = _JSON_SAFE_INT_MAX if safe_int else (1 << 60)
    return AuditRecord(
        op=rng.choice(["view", "create", "str_replace", "insert", "delete", "rename"]),
        path=f"memories/{rng.choice(['decisions', 'facts', 'projects'])}/{rng.randint(0, 1<<20):08x}.md",
        actor=rng.choice(["stephen", "coding-agent", "ops-agent"]),
        ts_ns=rng.randint(0, int_max),
        content_sha256=f"{rng.randint(0, 1<<256):064x}",
        prev_chain=f"{rng.randint(0, 1<<256):064x}",
        chain="",
        extra={
            "kind": rng.choice(["decision", "fact", "preference"]),
            "tags": [f"tag-{i}" for i in range(rng.randint(0, 5))],
            "version": rng.randint(0, 1024),
        },
    )


@pytest.mark.parametrize("n", [1_000])
def test_msgspec_round_trip_byte_stable(n: int) -> None:
    """Property 1: msgspec canonical-JSON is deterministic and round-trip stable."""
    import msgspec

    rng = random.Random(0xCAFEBABE)
    dec = msgspec.json.Decoder(AuditRecord)
    for _ in range(n):
        rec = _random_record(rng, safe_int=False)  # exercise full u64 range
        once = _canonical(rec)
        twice = _canonical(rec)
        assert once == twice, "msgspec encoding is non-deterministic for the same record"
        # Decode then re-encode → bytes must match.
        rec2 = dec.decode(once)
        assert _canonical(rec2) == once, "round-trip changed canonical bytes"


@pytest.mark.parametrize("n", [1_000])
def test_chain_hash_deterministic(n: int) -> None:
    """Two calls to _chain_hash with the same record produce the same hash."""
    rng = random.Random(0xDEADBEEF)
    for _ in range(n):
        rec = _random_record(rng, safe_int=False)
        a = _chain_hash(rec.prev_chain, rec)
        b = _chain_hash(rec.prev_chain, rec)
        assert a == b


@pytest.mark.parametrize("n", [1_000])
def test_msgspec_equivalent_to_rfc8785_within_safe_domain(n: int) -> None:
    """Property 2: within JSON safe-int domain, msgspec matches rfc8785 bytewise.

    This is the targeted version of the audit report's equivalence claim:
    valid only for inputs that legacy rfc8785 itself accepts. Used as a
    cross-reference for incident response on legacy-chain rows.
    """
    rfc8785 = pytest.importorskip("rfc8785")
    rng = random.Random(0xCAFEBABE)
    mismatches = 0
    for _ in range(n):
        rec = _random_record(rng, safe_int=True)
        msgspec_bytes = _canonical(rec)
        decoded = json.loads(msgspec_bytes.decode("utf-8"))
        rfc_bytes = rfc8785.dumps(decoded)
        if msgspec_bytes != rfc_bytes:
            mismatches += 1
    assert mismatches == 0, f"{mismatches}/{n} samples diverged from RFC 8785 JCS"
