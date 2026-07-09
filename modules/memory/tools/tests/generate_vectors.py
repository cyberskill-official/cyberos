#!/usr/bin/env python3
"""
Generate the test-vector corpus for cyberos-validate.

Creates ./vectors/<NN-name>/ each containing a `.cyberos/memory/store/` store +
an `expected.json` declaring the CRITICAL findings the validator should produce.

Run: python3 generate_vectors.py
"""

from __future__ import annotations

import hashlib
import json
import os
import shutil
from pathlib import Path

HERE = Path(__file__).parent
VECTORS = HERE / "vectors"

# Canonical genesis chain
GENESIS_CHAIN = "sha256:" + "0" * 64


def write(path: Path, content: str | bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if isinstance(content, str):
        path.write_text(content, encoding="utf-8")
    else:
        path.write_bytes(content)


def manifest_skeleton(audit_chain_head: str = GENESIS_CHAIN) -> dict:
    return {
        "memory_layer": 1,
        "tenant": {"id": "test", "name": "Test", "residency": "vn"},
        "owner": {"kind": "human", "id": "tester", "display_name": "Tester"},
        "project": {
            "id": "prj_test",
            "name": "Test",
            "root_path": "/tmp/test",
            "language": "en",
            "stack": []
        },
        "scope_root": "project:prj_test",
        "timezone": "UTC",
        "created_at": "2026-05-01T00:00:00Z",
        "last_updated_at": "2026-05-01T00:00:00Z",
        "memory_count": 0,
        "audit_chain_head": audit_chain_head,
        "exclusion_rules": [],
        "scope_contract": {
            "agent_default_write_scopes": ["project", "meta", "module"],
            "agent_default_read_scopes": [
                "company", "module", "member", "client",
                "project", "persona", "meta"
            ],
            "elevated_scopes_require_human_confirmation": [
                "company", "member", "client", "persona"
            ]
        },
        "size_limits": {
            "per_file_body_kb": 10,
            "per_file_hard_kb": 30,
            "per_tenant_total_mb": 10
        },
        "languages": ["en"],
        "language_routing_default": "en",
        "signing_key_fingerprint": None,
    }


def memory_md(memory_id: str, *, scope: str = "project:prj_test",
              classification: str = "operational",
              authority: str = "human-confirmed",
              version: int = 1,
              created_at: str = "2026-05-01T00:00:00Z",
              last_updated_at: str = "2026-05-01T00:00:00Z",
              extra_fields: dict | None = None,
              body: str = "Test body.\n",
              ) -> str:
    fm = {
        "memory_id": memory_id,
        "scope": scope,
        "classification": classification,
        "authority": authority,
        "version": version,
        "created_at": created_at,
        "created_by": "subject:tester",
        "last_updated_at": last_updated_at,
        "updated_by": "subject:tester",
    }
    if extra_fields:
        fm.update(extra_fields)
    lines = ["---"]
    for k, v in fm.items():
        if isinstance(v, list):
            lines.append(f"{k}: {json.dumps(v)}")
        elif isinstance(v, bool):
            lines.append(f"{k}: {'true' if v else 'false'}")
        elif v is None:
            lines.append(f"{k}: null")
        elif isinstance(v, str):
            # Quote if needed; simple cases unquoted
            if any(c in v for c in ":#") or v.startswith("-"):
                lines.append(f"{k}: \"{v}\"")
            else:
                lines.append(f"{k}: {v}")
        else:
            lines.append(f"{k}: {v}")
    lines.append("---")
    lines.append("")
    lines.append("# " + memory_id)
    lines.append("")
    lines.append(body)
    return "\n".join(lines)


def expected(crit: list[str]) -> str:
    return json.dumps(
        {"expected_critical_codes": sorted(crit)},
        indent=2, sort_keys=True
    ) + "\n"


def reset() -> None:
    if VECTORS.exists():
        shutil.rmtree(VECTORS)
    VECTORS.mkdir(parents=True, exist_ok=True)


# ---------------------------------------------------------------------------
# Fixture authors
# ---------------------------------------------------------------------------

def fixture_01_clean_bootstrap() -> None:
    """Pristine store with one bootstrap-shaped genesis row + empty scopes."""
    fix = VECTORS / "01-clean-bootstrap"
    store = fix / ".cyberos/memory/store"
    write(store / "manifest.json",
          json.dumps(manifest_skeleton(GENESIS_CHAIN), indent=2))
    write(store / "README.md", "# .cyberos/memory/store\nDo not edit audit/.\n")
    write(store / "audit" / "2026-05.jsonl", "")
    for d in ("company", "module", "member", "client", "project", "persona",
              "memories", "memories/decisions", "memories/facts", "meta"):
        (store / d).mkdir(exist_ok=True)
        (store / d / ".keep").touch()
    write(fix / "expected.json", expected([]))


def fixture_02_chain_break() -> None:
    """Two audit rows, second's prev_chain doesn't match first's chain."""
    fix = VECTORS / "02-chain-break"
    store = fix / ".cyberos/memory/store"
    write(store / "manifest.json",
          json.dumps(manifest_skeleton("sha256:" + "1" * 64), indent=2))
    rows = [
        {"audit_id": "evt_01HSXYZ00000000000000000A1",
         "ts": "2026-05-01T00:00:00Z",
         "actor_kind": "agent", "actor_id": "test",
         "op": "create", "path": ".cyberos/memory/store/test.md",
         "prev_chain": GENESIS_CHAIN,
         "chain": "sha256:" + "a" * 64},
        {"audit_id": "evt_01HSXYZ00000000000000000A2",
         "ts": "2026-05-01T00:01:00Z",
         "actor_kind": "agent", "actor_id": "test",
         "op": "create", "path": ".cyberos/memory/store/test2.md",
         # prev_chain SHOULD be sha256:aaa... but is wrong
         "prev_chain": "sha256:" + "f" * 64,
         "chain": "sha256:" + "1" * 64},
    ]
    write(store / "audit" / "2026-05.jsonl",
          "\n".join(json.dumps(r) for r in rows) + "\n")
    write(fix / "expected.json", expected(["chain-link-mismatch"]))


def fixture_03_supersedes_cycle() -> None:
    """A → supersedes → B → supersedes → A."""
    fix = VECTORS / "03-supersedes-cycle"
    store = fix / ".cyberos/memory/store"
    write(store / "manifest.json",
          json.dumps(manifest_skeleton(GENESIS_CHAIN), indent=2))
    write(store / "audit" / "2026-05.jsonl", "")
    a = "mem_01HSAAA0000000000000000001"
    b = "mem_01HSBBB0000000000000000002"
    write(store / "memories" / "decisions" / "DEC-001-a.md",
          memory_md(a, extra_fields={"supersedes": b}))
    write(store / "memories" / "decisions" / "DEC-002-b.md",
          memory_md(b, extra_fields={"supersedes": a}))
    write(fix / "expected.json", expected(["supersedes-cycle"]))


def fixture_04_dangling_supersedes() -> None:
    """A → supersedes → nonexistent."""
    fix = VECTORS / "04-dangling-supersedes"
    store = fix / ".cyberos/memory/store"
    write(store / "manifest.json",
          json.dumps(manifest_skeleton(GENESIS_CHAIN), indent=2))
    write(store / "audit" / "2026-05.jsonl", "")
    a = "mem_01HSAAA0000000000000000001"
    ghost = "mem_01HSGHOST00000000000000000"
    write(store / "memories" / "decisions" / "DEC-001-a.md",
          memory_md(a, extra_fields={"supersedes": ghost}))
    write(fix / "expected.json", expected(["supersedes-dangling"]))


def fixture_05_malformed_memory_id() -> None:
    """memory_id is plain UUIDv4, not v7/ULID."""
    fix = VECTORS / "05-malformed-memory-id"
    store = fix / ".cyberos/memory/store"
    write(store / "manifest.json",
          json.dumps(manifest_skeleton(GENESIS_CHAIN), indent=2))
    write(store / "audit" / "2026-05.jsonl", "")
    bad = "mem_de305d54-75b4-431b-adb2-eb6b9e546014"  # v4
    write(store / "memories" / "facts" / "FACT-001-bad.md",
          memory_md(bad))
    write(fix / "expected.json", expected(["memory-id-malformed"]))


def fixture_06_frontmatter_cap_exceeded() -> None:
    """Frontmatter >4KB hard cap."""
    fix = VECTORS / "06-frontmatter-cap-exceeded"
    store = fix / ".cyberos/memory/store"
    write(store / "manifest.json",
          json.dumps(manifest_skeleton(GENESIS_CHAIN), indent=2))
    write(store / "audit" / "2026-05.jsonl", "")
    big_tags = ["tag-" + str(i) for i in range(500)]
    a = "mem_01HSAAA0000000000000000001"
    write(store / "memories" / "facts" / "FACT-001-big.md",
          memory_md(a, extra_fields={"tags": big_tags}))
    write(fix / "expected.json", expected(["frontmatter-cap-exceeded"]))


def fixture_07_tombstoned_missing_metadata() -> None:
    """tombstoned: true but no deleted_at / deleted_by / tombstone_reason.

    Per §4.6 these are required when tombstoned. Validator emits WARN
    (not CRITICAL), so expected_critical_codes is empty.
    """
    fix = VECTORS / "07-tombstoned-missing-metadata"
    store = fix / ".cyberos/memory/store"
    write(store / "manifest.json",
          json.dumps(manifest_skeleton(GENESIS_CHAIN), indent=2))
    write(store / "audit" / "2026-05.jsonl", "")
    a = "mem_01HSAAA0000000000000000001"
    write(store / "memories" / "facts" / "FACT-001-tomb.md",
          memory_md(a, extra_fields={"tombstoned": True}))
    write(fix / "expected.json", expected([]))


def fixture_08_authority_invalid() -> None:
    """authority not in §5.3 hierarchy."""
    fix = VECTORS / "08-authority-invalid"
    store = fix / ".cyberos/memory/store"
    write(store / "manifest.json",
          json.dumps(manifest_skeleton(GENESIS_CHAIN), indent=2))
    write(store / "audit" / "2026-05.jsonl", "")
    a = "mem_01HSAAA0000000000000000001"
    write(store / "memories" / "facts" / "FACT-001-bad-auth.md",
          memory_md(a, authority="omniscient-llm"))
    write(fix / "expected.json", expected(["authority-invalid"]))


def fixture_09_classification_invalid() -> None:
    """classification not in §5.4 set."""
    fix = VECTORS / "09-classification-invalid"
    store = fix / ".cyberos/memory/store"
    write(store / "manifest.json",
          json.dumps(manifest_skeleton(GENESIS_CHAIN), indent=2))
    write(store / "audit" / "2026-05.jsonl", "")
    a = "mem_01HSAAA0000000000000000001"
    write(store / "memories" / "facts" / "FACT-001.md",
          memory_md(a, classification="top-secret"))
    write(fix / "expected.json", expected(["classification-invalid"]))


def fixture_10_temporal_monotonicity() -> None:
    """created_at > last_updated_at."""
    fix = VECTORS / "10-temporal-monotonicity"
    store = fix / ".cyberos/memory/store"
    write(store / "manifest.json",
          json.dumps(manifest_skeleton(GENESIS_CHAIN), indent=2))
    write(store / "audit" / "2026-05.jsonl", "")
    a = "mem_01HSAAA0000000000000000001"
    write(store / "memories" / "facts" / "FACT-001-time.md",
          memory_md(a,
                    created_at="2026-05-10T00:00:00Z",
                    last_updated_at="2026-05-01T00:00:00Z"))
    write(fix / "expected.json", expected(["temporal-monotonicity"]))


def fixture_11_audit_head_unreachable() -> None:
    """manifest.audit_chain_head doesn't appear in the ledger."""
    fix = VECTORS / "11-audit-head-unreachable"
    store = fix / ".cyberos/memory/store"
    write(store / "manifest.json",
          json.dumps(manifest_skeleton("sha256:" + "f" * 64), indent=2))
    rows = [
        {"audit_id": "evt_01HSXYZ00000000000000000B1",
         "ts": "2026-05-01T00:00:00Z",
         "actor_kind": "agent", "actor_id": "test",
         "op": "create", "path": ".cyberos/memory/store/test.md",
         "prev_chain": GENESIS_CHAIN,
         "chain": "sha256:" + "a" * 64},
    ]
    write(store / "audit" / "2026-05.jsonl",
          "\n".join(json.dumps(r) for r in rows) + "\n")
    write(fix / "expected.json", expected(["audit-chain-head-unreachable"]))


def fixture_12_audit_row_unparseable() -> None:
    """Mid-ledger non-JSON line (sync-collision simulation)."""
    fix = VECTORS / "12-audit-row-unparseable"
    store = fix / ".cyberos/memory/store"
    write(store / "manifest.json",
          json.dumps(manifest_skeleton(GENESIS_CHAIN), indent=2))
    rows = [
        json.dumps({
            "audit_id": "evt_01HSXYZ00000000000000000C1",
            "ts": "2026-05-01T00:00:00Z",
            "actor_kind": "agent", "actor_id": "test",
            "op": "create", "path": ".cyberos/memory/store/test.md",
            "prev_chain": GENESIS_CHAIN,
            "chain": "sha256:" + "a" * 64
        }),
        "not-valid-json{",  # corrupt line
    ]
    write(store / "audit" / "2026-05.jsonl",
          "\n".join(rows) + "\n")
    write(fix / "expected.json", expected(["audit-row-unparseable"]))


def fixture_13_confidence_out_of_range() -> None:
    """provenance.confidence > 1.0."""
    fix = VECTORS / "13-confidence-out-of-range"
    store = fix / ".cyberos/memory/store"
    write(store / "manifest.json",
          json.dumps(manifest_skeleton(GENESIS_CHAIN), indent=2))
    write(store / "audit" / "2026-05.jsonl", "")
    a = "mem_01HSAAA0000000000000000001"
    body = """---
memory_id: mem_01HSAAA0000000000000000001
scope: project:prj_test
classification: operational
authority: human-confirmed
version: 1
created_at: 2026-05-01T00:00:00Z
created_by: subject:tester
last_updated_at: 2026-05-01T00:00:00Z
updated_by: subject:tester
provenance:
  source: chat
  source_ref: test
  confidence: 1.5
---

# A
"""
    write(store / "memories" / "facts" / "FACT-001-conf.md", body)
    write(fix / "expected.json", expected(["confidence-out-of-range"]))


def fixture_14_unicode_vietnamese() -> None:
    """Vietnamese diacritics in body — should pass cleanly."""
    fix = VECTORS / "14-unicode-vietnamese"
    store = fix / ".cyberos/memory/store"
    write(store / "manifest.json",
          json.dumps(manifest_skeleton(GENESIS_CHAIN), indent=2))
    write(store / "audit" / "2026-05.jsonl", "")
    a = "mem_01HSAAA0000000000000000001"
    write(store / "memories" / "facts" / "FACT-001-vi.md",
          memory_md(a, body="Hiện thực hoá ý chí. CyberSkill — turn your will into real.\n"))
    write(fix / "expected.json", expected([]))


def fixture_17_encrypted_memory_valid() -> None:
    """encrypted: true with valid §5.6.1 envelope shape — should pass."""
    fix = VECTORS / "17-encrypted-memory-valid"
    store = fix / ".cyberos/memory/store"
    write(store / "manifest.json",
          json.dumps(manifest_skeleton(GENESIS_CHAIN), indent=2))
    write(store / "audit" / "2026-05.jsonl", "")
    a = "mem_01HSAAA0000000000000000001"
    write(store / "memories" / "facts" / "FACT-001-encrypted.md",
          memory_md(a, extra_fields={
              "encrypted": True,
              "encryption": {
                  "algorithm": "xchacha20poly1305-ietf-v0",
                  "nonce": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",  # 24 zero bytes (32 b64 chars, no padding)
                  "aad": "sha256(memory_id||last_updated_at)",
              },
          }, body="<encrypted ciphertext base64>"))
    write(fix / "expected.json", expected([]))


def fixture_18_encrypted_memory_bad_nonce() -> None:
    """encrypted: true with wrong-length nonce — CRITICAL."""
    fix = VECTORS / "18-encrypted-memory-bad-nonce"
    store = fix / ".cyberos/memory/store"
    write(store / "manifest.json",
          json.dumps(manifest_skeleton(GENESIS_CHAIN), indent=2))
    write(store / "audit" / "2026-05.jsonl", "")
    a = "mem_01HSAAA0000000000000000001"
    write(store / "memories" / "facts" / "FACT-001-bad-nonce.md",
          memory_md(a, extra_fields={
              "encrypted": True,
              "encryption": {
                  "algorithm": "xchacha20poly1305-ietf-v0",
                  "nonce": "AAAA",  # only 3 bytes — too short
                  "aad": "sha256(memory_id||last_updated_at)",
              },
          }, body="<bad>"))
    write(fix / "expected.json", expected(["encryption-nonce-length"]))


def fixture_19_shamir_fingerprint_missing() -> None:
    """encryption_policy.enabled but no master_key_fingerprint — CRITICAL."""
    fix = VECTORS / "19-shamir-fingerprint-missing"
    store = fix / ".cyberos/memory/store"
    m = manifest_skeleton(GENESIS_CHAIN)
    m["encryption_policy"] = {"enabled": True, "scopes": []}
    m["shamir_fragments"] = {
        "threshold": 3, "total": 5,
        "master_key_fingerprint": None,  # missing!
        "fragments": [],
    }
    write(store / "manifest.json", json.dumps(m, indent=2))
    write(store / "audit" / "2026-05.jsonl", "")
    write(fix / "expected.json", expected(["shamir-fingerprint-missing"]))


def fixture_20_merkle_checkpoint_divergence() -> None:
    """op:consolidation_run row with merkle_root that doesn't match
    the recomputation over preceding rows — CRITICAL."""
    fix = VECTORS / "20-merkle-checkpoint-divergence"
    store = fix / ".cyberos/memory/store"
    import hashlib
    try:
        import rfc8785
        canon = rfc8785.dumps
    except ImportError:
        canon = lambda v: __import__("json").dumps(  # noqa: E731
            v, sort_keys=True, separators=(",", ":"),
            ensure_ascii=False).encode("utf-8")

    genesis = "sha256:" + "0" * 64
    rows = []
    prev_chain = genesis
    # Build 3 normal rows + one consolidation_run with WRONG merkle_root
    for i in range(3):
        body = {
            "audit_id": f"evt_019e0000-0000-71a0-9000-00000000000{i+1}",
            "ts": f"2026-05-09T00:0{i}:00Z",
            "actor_kind": "agent", "actor_id": "test", "persona": None,
            "op": "create", "scope": "meta",
            "path": f".cyberos/memory/store/test{i+1}.md",
            "memory_id": None, "prev_version": None, "new_version": None,
            "supersedes_event_id": None, "classification": None,
            "authority": None, "consent_event_id": None,
            "provenance": {"source": "manual", "source_ref": "test", "confidence": 1.0},
            "before_hash": None, "after_hash": None, "diff": "<hash-only>",
            "reason": "test", "correction_to": None,
        }
        chain = "sha256:" + hashlib.sha256(canon(body) + prev_chain.encode()).hexdigest()
        rows.append({**body, "prev_chain": prev_chain, "chain": chain})
        prev_chain = chain

    # consolidation_run with WRONG merkle_root
    cons = {
        "audit_id": "evt_019e0000-0000-71a0-9000-000000000099",
        "ts": "2026-05-09T00:10:00Z",
        "actor_kind": "agent", "actor_id": "test", "persona": None,
        "op": "consolidation_run", "scope": "meta",
        "path": ".cyberos/memory/store/manifest.json",
        "memory_id": None, "prev_version": None, "new_version": None,
        "supersedes_event_id": None, "classification": None,
        "authority": None, "consent_event_id": None,
        "provenance": {"source": "manual", "source_ref": "test", "confidence": 1.0},
        "before_hash": None, "after_hash": None, "diff": "<hash-only>",
        "reason": "test", "correction_to": None,
        "merkle_root": "sha256:deadbeef" + "0" * 56,  # WRONG
    }
    cons_chain = "sha256:" + hashlib.sha256(canon(cons) + prev_chain.encode()).hexdigest()
    rows.append({**cons, "prev_chain": prev_chain, "chain": cons_chain})

    write(store / "audit" / "2026-05.jsonl",
          "\n".join(json.dumps(r) for r in rows) + "\n")
    m = manifest_skeleton(cons_chain)
    write(store / "manifest.json", json.dumps(m, indent=2))
    write(fix / "expected.json", expected(["merkle-checkpoint-divergence"]))


def fixture_21_classification_personnel_no_consent() -> None:
    """personnel-classification memory missing consent.has_consent: true."""
    fix = VECTORS / "21-personnel-no-consent"
    store = fix / ".cyberos/memory/store"
    write(store / "manifest.json",
          json.dumps(manifest_skeleton(GENESIS_CHAIN), indent=2))
    write(store / "audit" / "2026-05.jsonl", "")
    a = "mem_01HSAAA0000000000000000001"
    # Note: validator currently doesn't enforce consent; this fixture serves as
    # a forward-compat check that we'll catch it once the rule lands.
    write(store / "memories" / "people" / "PEOPLE-001-bob.md",
          memory_md(a, classification="personnel"))
    # Currently no CRITICAL — validator hasn't implemented this rule yet
    write(fix / "expected.json", expected([]))


def fixture_16_stale_checkpoint() -> None:
    """manifest.reconciliation_checkpoint.chain doesn't match the ledger
    row at the referenced audit_id. Stage 1 §8.7 phase 4 amendment."""
    fix = VECTORS / "16-stale-checkpoint"
    store = fix / ".cyberos/memory/store"
    # Build a minimal valid ledger first
    import hashlib
    try:
        import rfc8785
        canon = rfc8785.dumps
    except ImportError:
        canon = lambda v: __import__("json").dumps(  # noqa: E731
            v, sort_keys=True, separators=(",", ":"),
            ensure_ascii=False).encode("utf-8")

    genesis = "sha256:" + "0" * 64
    body = {
        "audit_id": "evt_019e0000-0000-71a0-9000-000000000001",
        "ts": "2026-05-09T00:00:00Z",
        "actor_kind": "agent", "actor_id": "test", "persona": None,
        "op": "create", "scope": "meta",
        "path": ".cyberos/memory/store/test1.md",
        "memory_id": None, "prev_version": None, "new_version": None,
        "supersedes_event_id": None, "classification": None,
        "authority": None, "consent_event_id": None,
        "provenance": {"source": "manual", "source_ref": "test",
                       "confidence": 1.0},
        "before_hash": None, "after_hash": None, "diff": "<hash-only>",
        "reason": "test", "correction_to": None,
    }
    chain = "sha256:" + hashlib.sha256(
        canon(body) + genesis.encode()).hexdigest()
    row = {**body, "prev_chain": genesis, "chain": chain}
    write(store / "audit" / "2026-05.jsonl", json.dumps(row) + "\n")
    m = manifest_skeleton(chain)  # head matches real chain
    m["reconciliation_checkpoint"] = {
        "audit_id": body["audit_id"],
        "chain": "sha256:deadbeef" + "0" * 56,  # WRONG
        "ts": "2026-05-09T00:00:00Z",
    }
    write(store / "manifest.json", json.dumps(m, indent=2) + "\n")
    write(fix / "expected.json", expected(["stale-checkpoint"]))


def fixture_15_duplicate_memory_id() -> None:
    """Two files share the same memory_id."""
    fix = VECTORS / "15-duplicate-memory-id"
    store = fix / ".cyberos/memory/store"
    write(store / "manifest.json",
          json.dumps(manifest_skeleton(GENESIS_CHAIN), indent=2))
    write(store / "audit" / "2026-05.jsonl", "")
    a = "mem_01HSAAA0000000000000000001"
    write(store / "memories" / "facts" / "FACT-001.md", memory_md(a))
    write(store / "memories" / "facts" / "FACT-002.md", memory_md(a))
    write(fix / "expected.json", expected(["memory-id-duplicate"]))


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    reset()
    for name, fn in sorted(globals().items()):
        if name.startswith("fixture_") and callable(fn):
            fn()
            print(f"  ✅ {name}")
    print(f"\nGenerated {len(list(VECTORS.iterdir()))} fixtures in {VECTORS}")
