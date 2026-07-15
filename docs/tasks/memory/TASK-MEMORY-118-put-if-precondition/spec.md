---
id: TASK-MEMORY-118
title: "memory put_if — optimistic-concurrency primitive with content-hash preconditions; many-agent contention without clobbering; canonical-ops extension §3.1"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-19T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: memory
priority: p1
status: done
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng
created: 2026-05-19
shipped: null
memory_chain_hash: null
related_tasks: [TASK-MEMORY-103, TASK-MEMORY-115, TASK-MEMORY-117]
depends_on: [TASK-MEMORY-117]
blocks: []
protocol_amendment_required: "AGENTS.md §3.1 (extend canonical-op list) — `put_if(path, body, meta, precondition_body_hash)` added; approval phrase: APPROVE protocol change P21 §3.1"

source_pages:
  - playground/extracts/memory-and-dreaming.transcript.txt  # see "optimistic concurrency" segment [468..493]
source_decisions:
  - DEC-240 (put_if is an ADDITIVE primitive; existing `put` keeps unchanged semantics — back-compat preserved by construction)
  - DEC-241 (Precondition is the SHA-256 body_hash of the existing memory file as observed by the caller; `None` ≡ "must not exist" — the create-only variant)
  - "DEC-242 (Failure response is structured `{outcome: \"rejected\", reason: \"precondition_failed\", expected: <hash>, actual: <hash>}`; emits `memory.precondition_failed` aux row for diagnostics; HEAD does NOT advance)"
  - DEC-243 (put_if interacts with §3.4 content-addressed semantics: the precondition check happens BEFORE the body-hash transition decision; same redacted-body invariant preserved)

language: python 3.10
service: modules/memory/cyberos/
new_files:
  - modules/memory/tests/test_put_if_optimistic_concurrency.py
modified_files:
  - modules/memory/cyberos/core/writer.py        # add `put_if(path, body, meta, precondition_body_hash)` method
  - modules/memory/cyberos/__main__.py           # add `cyberos put-if <path> --precondition <hash>` CLI
  - modules/memory/memory.schema.json            # add `put_if` to canonical-op enum + payload shape
  - modules/memory/memory.invariants.yaml        # `put-if-precondition-form` (error)
  - AGENTS.md                                     # §3.1 amendment (extend canonical-op list)
allowed_tools:
  - file_read: modules/memory/**
  - file_write: modules/memory/cyberos/core/writer.py, modules/memory/cyberos/__main__.py, modules/memory/tests/**, modules/memory/memory.schema.json, modules/memory/memory.invariants.yaml, AGENTS.md
  - bash: cd modules/memory && python -m pytest tests/test_put_if_optimistic_concurrency.py -v
  - bash: cd modules/memory && python -m cyberos put-if memories/facts/x.md - --precondition <hex> < body.md
disallowed_tools:
  - mutate AGENTS.md §3.1 canonical-op list without APPROVE protocol change P21 §3.1 chat-turn
  - bypass the TASK-MEMORY-117 ACL check for put_if (per §1 #5)

effort_hours: 8
subtasks:
  - "1.0h: AGENTS.md §3.1 amendment text (extends canonical-op table; requires APPROVE chat-turn)"
  - "0.5h: memory.schema.json — extend canonical-op enum with `put_if`; add `PreconditionBodyHash` definition (string, 64-char hex, or null)"
  - "0.5h: memory.invariants.yaml — `put-if-precondition-form` (error: must be 64-char hex or null)"
  - "2.0h: cyberos/core/writer.py — `put_if(path, body, meta, precondition_body_hash) -> PutIfResult`; inside lock: compute current body_hash; compare; on match (or null when target absent) → proceed with put logic; on mismatch → emit aux row + return rejected"
  - "0.5h: __main__.py — `cyberos put-if <path> <body-source> --precondition <hex>` CLI; reads hex from arg or `--precondition-from-file`"
  - "2.0h: tests/test_put_if_optimistic_concurrency.py — 14 cases (precondition match → write succeeds, mismatch → rejected, null precondition + path absent → succeeds, null precondition + path present → rejected, two concurrent put_if races → one succeeds one rejected, ACL check still applies, audit row emitted on rejection, head unchanged on rejection)"
  - "1.0h: integration test — simulates the multi-agent SRE scenario from the Anthropic talk (5 agents racing on one memory)"
  - "0.5h: CHANGELOG entry"
risk_if_skipped: "Without put_if, the memory's only write primitive is `put` (unconditional overwrite). Two failure modes follow: (1) Concurrent agents on the same memory clobber each other silently — last writer wins regardless of intent. The talk's headline concurrency property is the design that prevents this. (2) Dream-applier's transactional integrity (TASK-MEMORY-115 §1 #14) becomes harder to reason about — without preconditions, the applier can't distinguish 'expected state' from 'silently-changed state' at write time. The 8-hour effort is modest; the integrity gain is substantial. Skipping means multi-agent memory deployments must serialise everything through a higher-level queue, which contradicts the talk's 'hundreds or thousands of agents running concurrently' framing."
---

## §1 — Description (BCP-14 normative)

The `put_if` operation **MUST** extend the canonical-op list in AGENTS.md §3.1 as an additive primitive. It is `put` with an explicit precondition that the existing on-disk body's SHA-256 matches a caller-supplied value. The contract:

1. **MUST** add `put_if` to the canonical-op enum in `memory.schema.json`. Signature: `put_if(path, body, meta, precondition_body_hash: Optional[str]) -> PutIfResult`. The two-arg path/body shape mirrors `put`; the new third arg is the precondition.
2. **MUST** define `precondition_body_hash` as either:
    - A 64-char lowercase hex SHA-256 string ≡ "the body MUST currently hash to this; this is what I saw"
    - `null` ≡ "the path MUST NOT currently exist; create-only"
3. **MUST** acquire the same `.lock` (exclusive) that `put` does. Inside the lock:
    - If `precondition_body_hash` is `null`: check the path is absent. If absent → proceed with `put`. If present → return `PutIfResult(outcome="rejected", reason="precondition_failed", expected=null, actual=<current_hash>)`.
    - If `precondition_body_hash` is a hash: read the current body, compute its SHA-256, compare. Match → proceed with `put`. Mismatch → return `PutIfResult(outcome="rejected", reason="precondition_failed", expected=<provided>, actual=<observed>)`.
4. **MUST NOT** advance HEAD on rejection. The audit chain remains unchanged. A `memory.precondition_failed` aux audit row IS emitted (HEAD advances by exactly 1 for the aux row); see §1 #7. The `put` payload itself is never written.
5. **MUST** honour the TASK-MEMORY-117 store ACL check IN ADDITION to the precondition check. ACL check runs FIRST: if ACL rejects, return `rejected` with `reason: "acl_denied"`. Only if ACL allows does the precondition check fire. Rationale: ACL is policy; precondition is concurrency control; policy must clear before concurrency control matters.
6. **MUST** preserve all existing `put` semantics on success: same canonical row emission, same `extra.body_hash` annotation, same `extra` field passthrough. From the audit chain's perspective, a successful `put_if` is indistinguishable from a `put` — only the writer's pre-step differs.
7. **MUST** emit `memory.precondition_failed` aux audit row on rejection with payload:
    ```json
    {
      "actor":       "stephen",
      "path":        "memories/sre/dispatch-1.md",
      "expected":    "abc123…" | null,
      "actual":      "def456…" | "<absent>",
      "attempt_at":  "2026-05-19T08:00:00Z"
    }
    ```
8. **MUST** add a CLI surface `cyberos put-if <path> <body-source> --precondition <hex|none>`. `--precondition none` is the null-precondition form. `--precondition-from-file <path>` reads the hex from a file (for scripted operators).
9. **MUST** return the structured result type:
    ```python
    @dataclass
    class PutIfResult:
        outcome:        Literal["written", "rejected"]
        reason:         Optional[str]     # populated when outcome == "rejected"
        expected:       Optional[str]
        actual:         Optional[str]
        committed_seq:  Optional[int]     # the HEAD seq of the `put` row on success
    ```
10. **MUST** validate `precondition_body_hash` shape at API entry: 64-char lowercase hex OR `None`. Other shapes (`"abc"`, byte strings, hash with `sha256:` prefix) raise `ValueError`.
11. **MUST** be safe under retry loops — caller pattern:
    ```python
    while True:
        current = reader.read(path)
        new_body = update(current.body)
        res = writer.put_if(path, new_body, current.meta, precondition_body_hash=current.body_hash)
        if res.outcome == "written":
            break
        # res.outcome == "rejected" — re-read and retry
    ```
    The protocol provides the primitive; the retry policy is the caller's responsibility. Document the canonical pattern in §11.
12. **MUST** require the AGENTS.md §3.1 amendment APPROVED via `APPROVE protocol change P21 §3.1` chat-turn before `put_if` is callable. Same anchor-check pattern as TASK-MEMORY-115 / 117: `Writer.put_if(...)` raises `ProtocolAmendmentMissingError` if §3.1 hasn't been extended to include `put_if`.
13. **SHOULD** be implementable in INTEROP.md consumers — the precondition check is one-line (compute SHA-256 of the file body before write). Cross-agent interop note documented in INTEROP.md.
14. **SHOULD** support a batch variant `put_if_all([(path, body, meta, precond), ...])` that succeeds-all or rejects-all transactionally. Slice-4 stretch — useful for TASK-MEMORY-115 dream-apply (which already has its own transactional path).

---

## §2 — Why this design (rationale for humans)

**Why additive, not replacement (§1 #1, DEC-240).** The simple `put` is the right primitive for most operators (script writes one memory, doesn't care about race). Forcing every caller through precondition would punish the common case. `put_if` is the explicit choice when concurrency matters.

**Why SHA-256 body_hash as precondition (§1 #2, DEC-241).** Two reasons. (a) Content-equality is the natural primitive for memory files — two files with the same content are semantically the same. (b) `meta.body_hash` is already computed + stored by the writer for every memory; reusing it costs zero extra hashing. The caller can read the current file's frontmatter, grab `body_hash`, pass it back.

**Why `null` precondition for create-only (§1 #2).** Common pattern: "create this memory only if it doesn't exist." Without a null option, callers would compute the absent-file's "hash" as some magic value, which is awkward. `null` is unambiguous.

**Why ACL check first (§1 #5).** Two reasons. (a) Policy enforcement is more important than concurrency control; deny-first is safer. (b) An ACL-denied write should report `acl_denied`, not `precondition_failed` — different operator action required. Running ACL first makes the error specific.

**Why aux row even on rejection (§1 #4, §1 #7, DEC-242).** Dream-applier (TASK-MEMORY-115) and FS watcher (TASK-MEMORY-107) need to count rejections to detect contention storms. Operators investigating "why did agent X's writes silently disappear?" need a queryable log. The aux row is cheap; visibility is high-value.

**Why preserve `put` semantics on success (§1 #6).** `put_if` shouldn't pollute the audit chain with a new row shape — downstream consumers (walker, doctor, TASK-MEMORY-115's pattern detector, TASK-MEMORY-120's history view) all already understand `put` rows. Making `put_if`'s success row identical to a `put` row keeps every consumer working without changes.

**Why §3.1 amendment required (§1 #12, DEC-242).** §3.1 currently lists exactly three canonical ops (`put`, `move`, `delete`). Adding `put_if` is a normative extension to that list; INTEROP.md consumers need to know it exists. That's a protocol change, gated by APPROVE per §0.2.

**Why batch variant deferred (§1 #14).** Dream-applier already has its own transactional path. A general-purpose batch primitive needs a careful design (rollback semantics, partial-failure reporting, lock-window length). Slice-4 work.

---

## §3 — API contract

### Method signature

```python
# modules/memory/cyberos/core/writer.py — extension
from dataclasses import dataclass
from typing import Literal, Optional


@dataclass(frozen=True)
class PutIfResult:
    outcome:       Literal["written", "rejected"]
    reason:        Optional[str] = None
    expected:      Optional[str] = None
    actual:        Optional[str] = None
    committed_seq: Optional[int] = None


class Writer:
    # ... existing methods ...

    def put_if(
        self,
        path: str,
        body: bytes | str,
        meta: dict,
        precondition_body_hash: Optional[str] = None,
    ) -> PutIfResult:
        self._require_protocol_amendment_p21()                  # §1 #12

        # Shape check (§1 #10)
        if precondition_body_hash is not None:
            if not (isinstance(precondition_body_hash, str)
                    and len(precondition_body_hash) == 64
                    and all(c in "0123456789abcdef" for c in precondition_body_hash)):
                raise ValueError(
                    f"precondition_body_hash must be 64-char lowercase hex or None; "
                    f"got {precondition_body_hash!r}"
                )

        # ACL check (§1 #5) — runs before lock to fail fast
        acl_result = self._check_acl_write(path)
        if not acl_result.allowed:
            self._emit_aux("memory.acl_denied", {"path": path, **acl_result.payload_dict()})
            return PutIfResult(outcome="rejected", reason="acl_denied")

        with self._exclusive_lock():
            # Precondition check (§1 #3)
            existing_path = self.store_path / path
            existing_body = existing_path.read_bytes() if existing_path.exists() else None
            existing_hash = (
                hashlib.sha256(existing_body).hexdigest() if existing_body else None
            )

            match precondition_body_hash, existing_hash:
                case None, None:
                    pass                            # create-only: target absent ✓
                case None, _:
                    self._emit_aux("memory.precondition_failed", {
                        "path": path, "expected": None, "actual": existing_hash,
                        "attempt_at": _now_iso(),
                    })
                    return PutIfResult(outcome="rejected", reason="precondition_failed",
                                       expected=None, actual=existing_hash)
                case _, None:
                    self._emit_aux("memory.precondition_failed", {
                        "path": path, "expected": precondition_body_hash,
                        "actual": "<absent>", "attempt_at": _now_iso(),
                    })
                    return PutIfResult(outcome="rejected", reason="precondition_failed",
                                       expected=precondition_body_hash, actual="<absent>")
                case provided, actual if provided != actual:
                    self._emit_aux("memory.precondition_failed", {
                        "path": path, "expected": provided,
                        "actual": actual, "attempt_at": _now_iso(),
                    })
                    return PutIfResult(outcome="rejected", reason="precondition_failed",
                                       expected=provided, actual=actual)
                case _:
                    pass                            # match ✓

            # Proceed with the same put logic as the existing `put(path, body, meta)`
            committed_seq = self._put_internal(path, body, meta)
            return PutIfResult(outcome="written", committed_seq=committed_seq)
```

### AGENTS.md §3.1 amendment (proposed)

```text
## §3.1 (extended by P21 — requires APPROVE chat-turn per §0.2)

An agent operating on memory state MUST express every mutation as exactly
one of FOUR canonical operations:

| op       | semantic |
|----------|----------|
| `put`    | create or replace a memory file. Idempotent given identical args. |
| `move`   | rename within `<memory-root>/`. Preserves content hash. |
| `delete` | `mode ∈ {"tombstone", "purge"}`; default `"tombstone"`. |
| `put_if` | create or replace, GATED on `precondition_body_hash` (SHA-256 of  |
|          | current body) matching. `None` precondition ≡ "must not exist".   |

§3.1.5  `put_if` MUST emit `memory.precondition_failed` aux audit row on
mismatch; HEAD does NOT advance for the rejected `put` payload but DOES
advance by 1 for the aux row.

§3.1.6  `put_if` is INDISTINGUISHABLE from `put` in the success-row shape.
Downstream consumers (walker, doctor, dream pipeline) MUST NOT special-case
`put_if`-origin `put` rows.
```

---

## §4 — Acceptance criteria

1. **Precondition match → write succeeds** — read existing file, hash, call put_if with that hash → outcome `written`; HEAD advances by 1 (put row). *(traces_to: §1 #3)*
2. **Precondition mismatch → write rejected** — pass a different hash → outcome `rejected`; reason `precondition_failed`; expected + actual fields populated; HEAD advances by 1 (aux row only). *(traces_to: §1 #3, §1 #4)*
3. **Null precondition + path absent → write succeeds** — target doesn't exist, precondition=null → outcome `written`. *(traces_to: §1 #2, §1 #3)*
4. **Null precondition + path present → rejected** — target exists, precondition=null → outcome `rejected`; expected=null, actual=<current hash>. *(traces_to: §1 #2, §1 #3)*
5. **Hash + path absent → rejected** — target doesn't exist, precondition=<some hash> → outcome `rejected`; expected=<provided>, actual=`"<absent>"`. *(traces_to: §1 #3)*
6. **Two concurrent put_if race → one wins** — simulate two threads with the same precondition; one returns `written`, the other returns `rejected`. *(traces_to: §1 #3)*
7. **HEAD doesn't advance on rejection (put row not emitted)** — only the aux row is emitted, not a put row. Inspect HEAD seq before + after rejected call. *(traces_to: §1 #4)*
8. **ACL denial reports `acl_denied`, not `precondition_failed`** — STORE.yaml denies; valid precondition supplied → outcome `rejected` with reason `acl_denied`. *(traces_to: §1 #5)*
9. **ACL check runs before precondition** — instrument both; deny-by-acl + wrong-precondition → only ACL check fires; no `memory.precondition_failed` row. *(traces_to: §1 #5)*
10. **Success row indistinguishable from put** — successful put_if emits same row shape as direct put (same `extra` keys, same canonical fields). *(traces_to: §1 #6)*
11. **Aux row payload shape** — `memory.precondition_failed` row matches §1 #7 schema (actor, path, expected, actual, attempt_at). *(traces_to: §1 #7)*
12. **CLI `put-if --precondition <hex>` works** — happy path. *(traces_to: §1 #8)*
13. **CLI `put-if --precondition none` for create-only** — alias for null. *(traces_to: §1 #8)*
14. **CLI `put-if --precondition-from-file`** — reads hex from file. *(traces_to: §1 #8)*
15. **PutIfResult shape** — typed result has all fields (outcome, reason, expected, actual, committed_seq). *(traces_to: §1 #9)*
16. **Bad precondition shape rejected** — `put_if(..., precondition_body_hash="abc")` raises ValueError; `precondition_body_hash=b"\xab\xcd…"` (bytes) raises. *(traces_to: §1 #10)*
17. **Bad precondition with uppercase rejected** — `"ABC123..."` (uppercase hex) raises (must be lowercase per spec). *(traces_to: §1 #10)*
18. **Retry-loop pattern works end-to-end** — write a memory; concurrently the file gets overwritten; retry loop re-reads + retries; eventual `written`. *(traces_to: §1 #11)*
19. **§3.1 anchor required** — AGENTS.md §3.1 missing the `put_if` extension → `Writer.put_if(...)` raises `ProtocolAmendmentMissingError`. *(traces_to: §1 #12)*
20. **§3.1 anchor present → put_if callable** — AGENTS.md amended with `put_if` row in the canonical-op table → callable. *(traces_to: §1 #12)*

---

## §5 — Verification

```python
# modules/memory/tests/test_put_if_optimistic_concurrency.py
import hashlib, threading, pytest
from cyberos.core.writer import Writer, PutIfResult


def hash_body(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def test_precondition_match_writes(seeded_memory_with_x, ensure_section_3_1):
    """AC #1"""
    current = (seeded_memory_with_x.store_path / "memories/x.md").read_text()
    h = hash_body(current)
    head_before = seeded_memory_with_x.head_seq()
    res = seeded_memory_with_x.put_if("memories/x.md", current + "\nappended",
                                      meta={}, precondition_body_hash=h)
    assert res.outcome == "written"
    assert res.committed_seq == head_before + 1


def test_precondition_mismatch_rejects(seeded_memory_with_x, ensure_section_3_1):
    """AC #2 + #7"""
    head_before = seeded_memory_with_x.head_seq()
    res = seeded_memory_with_x.put_if("memories/x.md", "new body", meta={},
                                      precondition_body_hash="0" * 64)
    assert res.outcome == "rejected"
    assert res.reason == "precondition_failed"
    # HEAD advances by 1 (aux row), not 2 (no put row)
    assert seeded_memory_with_x.head_seq() == head_before + 1


def test_null_precondition_path_absent_writes(empty_memory, ensure_section_3_1):
    """AC #3"""
    res = empty_memory.put_if("memories/new.md", "fresh body", meta={},
                              precondition_body_hash=None)
    assert res.outcome == "written"


def test_null_precondition_path_present_rejects(seeded_memory_with_x, ensure_section_3_1):
    """AC #4"""
    res = seeded_memory_with_x.put_if("memories/x.md", "trying to create over existing",
                                      meta={}, precondition_body_hash=None)
    assert res.outcome == "rejected"
    assert res.expected is None
    assert res.actual is not None


def test_hash_precondition_path_absent_rejects(empty_memory, ensure_section_3_1):
    """AC #5"""
    res = empty_memory.put_if("memories/absent.md", "body", meta={},
                              precondition_body_hash="a" * 64)
    assert res.outcome == "rejected"
    assert res.actual == "<absent>"


def test_concurrent_put_if_one_wins(seeded_memory_with_x, ensure_section_3_1):
    """AC #6"""
    current = (seeded_memory_with_x.store_path / "memories/x.md").read_text()
    h = hash_body(current)
    results = []
    barrier = threading.Barrier(2)

    def attempt(suffix):
        barrier.wait()
        res = seeded_memory_with_x.put_if("memories/x.md", current + suffix,
                                          meta={}, precondition_body_hash=h)
        results.append(res)

    t1 = threading.Thread(target=attempt, args=("\nfrom-1",))
    t2 = threading.Thread(target=attempt, args=("\nfrom-2",))
    t1.start(); t2.start(); t1.join(); t2.join()

    outcomes = sorted(r.outcome for r in results)
    assert outcomes == ["rejected", "written"]


def test_acl_denial_reported_specifically(seeded_memory_with_x, deny_acl,
                                          ensure_section_3_1):
    """AC #8 + #9"""
    h = hash_body((seeded_memory_with_x.store_path / "memories/x.md").read_text())
    res = seeded_memory_with_x.put_if("memories/x.md", "new", meta={},
                                      precondition_body_hash=h)
    assert res.outcome == "rejected"
    assert res.reason == "acl_denied"
    # No precondition_failed row emitted
    rows = seeded_memory_with_x.read_recent_audit_rows(2)
    assert not any(r["kind"] == "memory.precondition_failed" for r in rows)


def test_success_row_indistinguishable_from_put(seeded_memory_with_x, ensure_section_3_1):
    """AC #10"""
    h = hash_body((seeded_memory_with_x.store_path / "memories/x.md").read_text())
    res = seeded_memory_with_x.put_if("memories/x.md", "new body", meta={},
                                      precondition_body_hash=h)
    put_row = seeded_memory_with_x.read_audit_row(res.committed_seq)
    assert put_row["kind"] == "put"           # NOT "put_if"
    # Shape matches the existing put row exactly
    assert set(put_row.keys()) >= {"kind", "payload", "extra"}


def test_aux_row_payload_shape(seeded_memory_with_x, ensure_section_3_1):
    """AC #11"""
    head_before = seeded_memory_with_x.head_seq()
    seeded_memory_with_x.put_if("memories/x.md", "new", meta={},
                                precondition_body_hash="0" * 64)
    aux = seeded_memory_with_x.read_audit_row(head_before + 1)
    assert aux["kind"] == "memory.precondition_failed"
    for key in ("path", "expected", "actual", "attempt_at"):
        assert key in aux["payload"]


@pytest.mark.parametrize("bad", [
    "abc",                            # too short
    "x" * 64,                          # non-hex
    "ABC" + "0" * 61,                  # uppercase (per spec lowercase only)
    b"\xab" * 32,                       # bytes
    123,                                # int
])
def test_bad_precondition_shape_rejected(empty_memory, ensure_section_3_1, bad):
    """AC #16 + #17"""
    with pytest.raises(ValueError):
        empty_memory.put_if("memories/x.md", "body", meta={},
                            precondition_body_hash=bad)


def test_protocol_amendment_required(seeded_memory_without_section_3_1):
    """AC #19"""
    with pytest.raises(Exception) as exc:
        seeded_memory_without_section_3_1.put_if("memories/x.md", "x", meta={},
                                                  precondition_body_hash=None)
    assert "APPROVE protocol change P21 §3.1" in str(exc.value)


def test_retry_loop_eventual_write(seeded_memory_with_x, ensure_section_3_1,
                                    background_overwriter):
    """AC #18"""
    final = None
    for _ in range(5):
        current = (seeded_memory_with_x.store_path / "memories/x.md").read_text()
        h = hash_body(current)
        # Concurrent overwriter just touched the file again; retry might fail
        res = seeded_memory_with_x.put_if("memories/x.md", current + "\nfinal",
                                          meta={}, precondition_body_hash=h)
        if res.outcome == "written":
            final = res; break
    assert final is not None and final.outcome == "written"


def test_cli_put_if_works(seeded_memory_with_x, ensure_section_3_1, capsys):
    """AC #12"""
    import subprocess
    h = hash_body((seeded_memory_with_x.store_path / "memories/x.md").read_text())
    result = subprocess.run([
        "python", "-m", "cyberos", "--store", str(seeded_memory_with_x.store_path),
        "put-if", "memories/x.md", "-", "--precondition", h
    ], input="updated body", capture_output=True, text=True)
    assert result.returncode == 0
    assert "written" in result.stdout


def test_cli_put_if_none(seeded_memory_with_x, ensure_section_3_1):
    """AC #13"""
    import subprocess
    result = subprocess.run([
        "python", "-m", "cyberos", "--store", str(seeded_memory_with_x.store_path),
        "put-if", "memories/new.md", "-", "--precondition", "none"
    ], input="fresh body", capture_output=True, text=True)
    assert result.returncode == 0
    assert "written" in result.stdout
```

---

## §6 — Implementation skeleton

API + tests above are the skeleton. Order:

1. AGENTS.md §3.1 amendment text (DO NOT commit until APPROVE chat-turn).
2. Schema (`memory.schema.json`) — add `put_if` op.
3. Walker invariant `put-if-precondition-form`.
4. Writer extension: `put_if` method + result type.
5. CLI subcommand.
6. Tests.
7. INTEROP.md note.
8. CHANGELOG.

---

## §7 — Dependencies

- **TASK-MEMORY-117 (depends on)** — ACL check runs first. This task requires `put_if` to honour STORE.yaml ACL.
- **TASK-MEMORY-115 (related)** — dream-applier's preconditions are conceptually identical; the applier could be refactored to use `put_if` (slice-4 cleanup).
- **TASK-MEMORY-103 (related)** — multi-device sync uses optimistic concurrency at the chain level (`prev_chain`); this task adds it at the memory-file level for finer granularity.

---

## §8 — Example payloads

### Successful put_if

```text
$ cyberos put-if memories/sre/dispatch-1.md - --precondition abc123def456789a...
< updated body content
Result: {"outcome":"written","committed_seq":4319}
```

### Rejected on mismatch

```text
$ cyberos put-if memories/sre/dispatch-1.md - --precondition 0000000000000000...
< new body
Result: {"outcome":"rejected","reason":"precondition_failed","expected":"00000000...","actual":"abc123de..."}
```

### Rejected on ACL

```text
$ cyberos put-if memories/org-wide-knowledge/runbook.md - --precondition <hash> --actor scheduled-importer
Result: {"outcome":"rejected","reason":"acl_denied"}
```

### `memory.precondition_failed` aux row

```json
{
  "kind": "memory.precondition_failed",
  "payload": {
    "actor":       "stephen",
    "path":        "memories/sre/dispatch-1.md",
    "expected":    "0000000000000000000000000000000000000000000000000000000000000000",
    "actual":      "abc123def456789abcdef0123456789abcdef0123456789abcdef0123456789a",
    "attempt_at":  "2026-05-19T08:00:00Z"
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Batch `put_if_all([...])` — §1 #14; slice 4 (TASK-MEMORY-115 dream-applier already has its own batch path; a general-purpose batch primitive is a slice-4 design discussion).
- ETag-style "If-Match"/"If-None-Match" HTTP semantics on `cyberos serve` — slice 4 once the REST surface picks up multi-writer demand.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| §3.1 missing put_if | `_require_protocol_amendment_p21` | raises `ProtocolAmendmentMissingError` | Operator runs APPROVE chat-turn |
| Precondition shape invalid | shape check at API entry | ValueError | Caller fixes hash format |
| ACL denies | check before precondition | rejected w/ reason=acl_denied | Caller checks ACL or uses different actor |
| Two concurrent put_if same precondition | lock serialises | one written, other rejected | Caller retries |
| File deleted between read and put_if | precondition has hash, actual=<absent> | rejected | Caller decides: re-read + null-precondition put_if, or give up |
| File renamed between read and put_if | path doesn't exist | rejected with actual=<absent> | Caller handles |
| Bytes input vs str input | writer's normal handling | works | None |
| Empty body | accepted; hash of empty string is well-defined | works | None |
| Very large body | normal put performance | works | None |
| Concurrent put (not put_if) clobbers | put doesn't check preconditions | last writer wins (existing semantics) | Caller uses put_if instead of put |
| Concurrent read between two put_ifs | reader sees consistent state at any point | None - by design | None |
| Hash collision (cosmic ray) | cryptographically improbable | n/a | n/a |
| HEAD advances by 0 on rejection | aux row IS emitted | HEAD +1 | None — aux row is the audit |
| Caller passes uppercase hex | shape check | ValueError | Caller lowercases |
| Memory file is encrypted (task §5.4) | precondition is computed on the cipher BYTES (not plaintext) | works because both sides see the same ciphertext | None |
| File has UTF-8 BOM | hash computed on raw bytes | works (BOM is part of body) | None |
| Caller forgets `--precondition` flag | argparse required arg | CLI error | Caller adds flag |

---

## §11 — Implementation notes

- **Why precondition is on bytes, not on the canonical document.** The `body_hash` in `meta` is computed by the writer on the raw body. Reusing that hash means callers can read `meta.body_hash` and pass it straight back. Computing on canonicalised form would force callers to canonicalise, which is wasted work.
- **Why `put_if` doesn't update `meta.body_hash` differently from `put`.** It doesn't need to — the new body's hash IS the new `meta.body_hash`, regardless of how the write was triggered. Idempotence preserved.
- **Why we don't expose `put_if` as separate `kind` in the audit row.** Downstream consumers (walker, doctor, TASK-MEMORY-115 detectors, TASK-MEMORY-120 history) all already understand `put`. A new kind would force every consumer to handle two row shapes. The success row's invisibility is the load-bearing simplification.
- **The §3.1 amendment changes ONLY the canonical-op table.** It doesn't change anything about `put`, `move`, or `delete`. Strictly additive.
- **Retry-loop pattern is the canonical caller pattern.** Document it in INTEROP.md so cross-agent consumers don't reinvent it. The protocol provides the primitive; the loop is library-level.
- **`background_overwriter` test fixture** writes to the same file every 50ms during the test. Forces the retry loop to actually retry. The test asserts eventual progress — the retry budget (5 attempts) is generous enough that flaky-CI is not a real risk.
- **AC #18 is the load-bearing AC** — it asserts that the primitive composes into a working real-world retry pattern, not just that the primitive in isolation rejects correctly.
- **The `match` statement in the writer extension is Python 3.10+** — matches the module's existing requirements; the four `case` arms map exactly to the precondition × current-existence cross-product.

---

*End of TASK-MEMORY-118.*

## As built (2026-07-02)

Put-if-precondition logic shipped inside modules/memory.
