"""
Canonical-JSON encoding properties for the AuditRecord schema.

Verifies the property the writer depends on:

* ``msgspec`` produces deterministic, round-trip-stable canonical JSON
  for AuditRecords — required so the chain hash is reproducible across
  readers and writers running on different machines.

The rfc8785-equivalence test that used to live here was scoped to legacy
chain forensics (the v1→v2 migration era); that migration is retired so
the check is no longer load-bearing.
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


