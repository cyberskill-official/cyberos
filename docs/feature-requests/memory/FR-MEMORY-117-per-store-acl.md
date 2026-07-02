---
id: FR-MEMORY-117
title: "memory per-store ACL — `STORE.yaml` per top-level subtree (memories/ meta/ company/ client/ project/ persona/ episodes/) auto-generated on migration; writer enforces ACL on every put/move/delete; reads remain unrestricted to local processes"
module: memory
priority: SHOULD
status: done
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng
created: 2026-05-19
shipped: null
memory_chain_hash: null
related_frs: [FR-MEMORY-103, FR-MEMORY-106, FR-MEMORY-112, FR-MEMORY-115]
depends_on: []
blocks: [FR-MEMORY-118]
protocol_amendment_required: "AGENTS.md §14.4 (new) — store-level ACL via STORE.yaml; consumer writers MUST honour acl; approval phrase: APPROVE protocol change P20 §14.4"

source_pages:
  - playground/extracts/memory-and-dreaming.transcript.txt  # see "permission scopes" segment [435..464]
source_decisions:
  - DEC-230 (One STORE.yaml per existing top-level dir auto-generated on first run after upgrade; operator can edit thereafter; missing STORE.yaml ≡ permissive default — back-compat preserved)
  - DEC-231 (ACL grammar: `{actor: <glob>, mode: "read" | "read-write" | "deny"}`; first-match-wins; explicit `deny` overrides upstream allow)
  - DEC-232 (Reads remain unrestricted to local processes — ACL is WRITE-side enforcement. The talk's model: read-only-on-org-wide vs read-write-on-working memory; "read-only" means writer rejects writes, not that other processes can't open the file. OS file permissions are out of scope)
  - DEC-233 (`actor: *` is the wildcard; `actor: dream-runner` and `actor: dream-applier` are reserved literal identities; custom literal actors set via `--actor <name>` on cyberos invocations)

language: python 3.10
service: modules/memory/cyberos/
new_files:
  - modules/memory/cyberos/core/store_acl.py
  - modules/memory/tests/core/test_store_acl.py
  - modules/memory/tests/core/test_store_acl.py
  - modules/memory/scripts/migrate-store-acl.py
modified_files:
  - modules/memory/cyberos/core/writer.py        # enforce ACL on every put/move/delete; lazy-load STORE.yaml per top-level dir
  - modules/memory/cyberos/core/walker.py        # invariant `store-yaml-acl-valid` rule
  - modules/memory/memory.schema.json            # `StoreAcl` + `StoreAclEntry` definitions
  - modules/memory/memory.invariants.yaml        # `store-yaml-acl-valid` (error)
  - AGENTS.md                                     # add §14.4 — store-level ACL (REQUIRES amendment via APPROVE chat-turn P20 §14.4)
allowed_tools:
  - file_read: modules/memory/**
  - file_write: modules/memory/cyberos/**, modules/memory/tests/**, modules/memory/scripts/**, modules/memory/memory.schema.json, modules/memory/memory.invariants.yaml, AGENTS.md
  - bash: cd modules/memory && python -m pytest tests/test_store_acl_*.py -v
  - bash: cd modules/memory && python scripts/migrate-store-acl.py --store /tmp/memory --dry-run
disallowed_tools:
  - bypass ACL check on writes that originate from `cyberos.core.writer.Writer` (per §1 #1)
  - mutate AGENTS.md §14.4 without an APPROVE protocol change P20 §14.4 chat-turn
  - enforce ACL on reads (per DEC-232 — write-side only)

effort_hours: 24
sub_tasks:
  - "1.0h: AGENTS.md §14.4 amendment text drafted (requires Stephen APPROVE chat-turn before merge)"
  - "1.5h: memory.schema.json — `StoreAcl` (`acl: [StoreAclEntry, ...]`, `default_mode`) + `StoreAclEntry` (`actor: string`, `mode: enum`) definitions"
  - "0.5h: memory.invariants.yaml — `store-yaml-acl-valid` (error: parse + actor glob valid)"
  - "2.0h: cyberos/core/store_acl.py — `StoreAcl` dataclass + `resolve_mode(acl, actor, path) -> 'read' | 'read-write' | 'deny'`; first-match-wins glob matching; explicit-deny precedence"
  - "3.0h: cyberos/core/writer.py — on every put/move/delete: walk up from target path to nearest STORE.yaml; if present, check ACL; if absent or path not in any subtree, permissive default; cache parsed STORE.yaml in-memory; invalidate cache on `STORE.yaml` mtime change"
  - "1.0h: walker invariant + doctor surface — `cyberos doctor` lists per-store ACL with resolved mode for the active actor"
  - "2.0h: scripts/migrate-store-acl.py — walks `<memory-root>/` for top-level dirs without STORE.yaml; writes default STORE.yaml (`acl: [{actor: '*', mode: 'read-write'}]` ≡ permissive) per DEC-230 back-compat"
  - "3.0h: modules/memory/tests/core/test_store_acl.py — 18 cases (permissive default, wildcard actor, explicit actor read-only, explicit deny overrides allow, glob actor matching, path-not-in-subtree permissive, write blocked → returns 'acl_denied' audit row, move respects both src + dst ACL, delete respects target ACL)"
  - "2.0h: modules/memory/tests/core/test_store_acl.py — 8 cases (idempotent migration, existing STORE.yaml not overwritten, dry-run prints planned writes, missing directories created on-the-fly only if --create-dirs, invalid STORE.yaml after manual edit triggers walker error)"
  - "2.0h: cyberos/__main__.py — `cyberos acl show` / `cyberos acl validate` / `cyberos acl explain <path>` operator surfaces"
  - "1.0h: INTEROP.md one-line note (gated on §14.4 amendment)"
  - "1.0h: CHANGELOG + migration runbook entry"
risk_if_skipped: "Without per-store ACL, the memory cannot represent the talk's three-store SRE example: one `org-wide-knowledge` read-only store, two read-write working stores. Every memory write goes through the same trust level. Two failure modes follow: (1) An automated process (dream-runner, scheduled importer) accidentally writes into a 'sacred' subtree (e.g. `memories/decisions/` that should be human-only). (2) Different agent instances (different `--actor` ids) can't have differentiated trust — useful when one agent is a 'reviewer' that should never write its own opinions into the canonical refinements store. The talk explicitly cites this as a load-bearing property for multi-agent systems. Skipping means FR-MEMORY-115's dream pipeline can't be safely scoped — operators would have to trust every dream apply to touch every memory. ACL-gating gives operators a 'this dream may only write to memories/refinements/' guarantee."
---

## §1 — Description (BCP-14 normative)

The store-ACL layer **MUST** sit between the canonical `cyberos.core.writer.Writer` and the filesystem. Every `put` / `move` / `delete` operation is checked against the relevant `STORE.yaml` before the row is appended. The contract:

1. **MUST** enforce ACLs on every write through `Writer`. The check runs after path validation (§3.3 of AGENTS.md) and before the audit-chain append.
2. **MUST NOT** enforce ACLs on reads. Reads are unrestricted to all local processes that can read the filesystem (DEC-232). OS file permissions are the operator's responsibility, not the protocol's.
3. **MUST** locate the active `STORE.yaml` by walking from the target path UP toward `<memory-root>/`. The first `STORE.yaml` encountered governs; further-up `STORE.yaml` files are ignored. This lets `memories/episodes/STORE.yaml` override a hypothetical `memories/STORE.yaml`. If no `STORE.yaml` is found anywhere up to the root, the default is permissive (`read-write` for all actors) — preserving back-compat for stores predating this FR.
4. **MUST** define `STORE.yaml` shape:
    ```yaml
    store_id: org-wide-knowledge
    default_mode: read        # read | read-write | deny ; applied when no acl entry matches
    acl:
      - {actor: "*",                  mode: "read"}
      - {actor: "stephen@*",          mode: "read-write"}
      - {actor: "dream-runner",       mode: "read-write"}
      - {actor: "scheduled-importer", mode: "deny"}
    ```
5. **MUST** resolve `actor` matches with glob-style matching against the active actor identity (`--actor <name>` flag on CLI, or `Writer(actor=...)` arg). First-match-wins, IN ORDER as written in the YAML. An explicit `deny` always blocks regardless of `default_mode`.
6. **MUST** support these built-in actor literals (no glob needed):
    - `*` — wildcard (matches any actor)
    - `dream-runner` — FR-MEMORY-115's dream runner identity
    - `dream-applier` — FR-MEMORY-115's dream applier identity
    - `scheduled-importer` — reserved for cron-driven `cyberos import` invocations
    - `claude-code-hook` — FR-MEMORY-109's hook identity
    - any literal string matching `^[a-z][a-z0-9_-]*$` for custom actor ids
    - glob form `user@*` or `*@example.com` matches the literal "@" delimiter
7. **MUST** reject writes that resolve to `deny` or that resolve to `read` with structured error `{outcome: "rejected", reason: "acl_denied", actor: ..., path: ..., store_id: ...}`. Rejected writes do NOT advance HEAD and do NOT emit `put`/`move`/`delete` rows — but they MUST emit a `memory.acl_denied` aux audit row capturing the attempt. The aux row's payload is the same struct as the error.
8. **MUST** handle `move(src, dst)` by checking ACL on BOTH paths — write capability on `src` (to delete the old location) AND write capability on `dst` (to create the new location). Either failing blocks the move.
9. **MUST** lazily load `STORE.yaml` on first write to its subtree and cache the parsed result in-memory keyed by `(store_dir, mtime_ns)`. On `STORE.yaml` mtime change (or content change detected by `cyberos doctor`), the cache entry is invalidated automatically on next access.
10. **MUST** ship a migration script `scripts/migrate-store-acl.py` that walks `<memory-root>/` and writes a default permissive `STORE.yaml` for every top-level dir that doesn't have one. Migration is idempotent — existing `STORE.yaml` files are never overwritten. Supports `--dry-run` (prints planned writes) and `--scope <dir>` (limit to one subtree).
11. **MUST** validate `STORE.yaml` shape against `memory.schema.json#/definitions/StoreAcl` at parse time. Invalid YAML or invalid shape → walker invariant `store-yaml-acl-valid` fails with structured error naming the file and the violation.
12. **MUST** add `cyberos acl show [--store <id>]` (lists STORE.yaml content per store), `cyberos acl validate` (re-validates all STORE.yaml files), and `cyberos acl explain <path>` (resolves the effective mode for the active actor on a given path, with the matching ACL entry highlighted).
13. **MUST** require the AGENTS.md §14.4 amendment APPROVED via `APPROVE protocol change P20 §14.4` chat-turn before runtime ACL enforcement is enabled. Same runtime-check pattern as FR-MEMORY-115's §7.7: the writer checks for the §14.4 anchor in AGENTS.md at construction; absent → ACL enforcement is in WARN-ONLY mode (rejections logged + aux row emitted, but writes proceed). Once §14.4 is anchored, full enforcement is active.
14. **MUST** emit the `memory.acl_denied` aux row even in WARN-ONLY mode so operators can audit "what would have been blocked once §14.4 lands."
15. **SHOULD** support per-store `default_mode: deny` with explicit-allow ACL entries — useful for highly-sensitive stores (e.g. a `secrets/` subtree where only one actor is permitted). Slice-4 stretch — slice-3 ships `default_mode: read` and `read-write` only.
16. **SHOULD** support `cyberos acl audit --since 24h` — surfaces all `memory.acl_denied` rows in the window with a summary of which actors attempted what. Slice-4 polish.

---

## §2 — Why this design (rationale for humans)

**Why subtree-rooted, not root-only (§1 #3).** The talk's three-store example demonstrates the load-bearing property: different subtrees need different rules. A single root-level ACL conflates the rules; one operator typo cascades across the memory. Subtree-rooted means a typo in `memories/episodes/STORE.yaml` affects episodes only. Search is upward — the closest `STORE.yaml` wins, which makes operator mental model match filesystem mental model.

**Why permissive default (§1 #3, DEC-230).** Back-compat with existing memories that predate this FR. An operator upgrading should see zero behaviour change until they edit or migrate. The migration script (§1 #10) writes the default permissive `STORE.yaml` so the explicit shape is materialised for future edits, but the semantic remains "everything works as before."

**Why first-match-wins, with explicit-deny overriding (§1 #5, DEC-231).** Iptables-style rules are operator-intuitive; ordered evaluation matches how operators read the file. Explicit `deny` is special-cased because "first I want to block X, then permit everyone else" is a common pattern that's awkward in pure first-match (it forces putting deny entries first AND repeating allow patterns).

**Why writes-only enforcement (§1 #2, DEC-232).** Two reasons. (a) Read enforcement requires kernel-level filesystem permissions, which conflate "OS user" with "agent actor" — the agent identity is a logical concept, not a unix uid. (b) The Anthropic talk frames `read-only` as "the agent's writer rejects writes," not as "the agent can't see the bytes." A separate file-permissions layer can be applied by the operator if they need uid-level isolation; that's out of scope for the memory protocol.

**Why WARN-ONLY mode pre-amendment (§1 #13).** Anti-footgun. An operator pulling the FR code but not running the APPROVE chat-turn shouldn't have their writes silently blocked overnight. WARN-ONLY mode is the "ask forgiveness, not permission" pattern adapted to protocol amendments: the code path exists, it logs what it would have done, the operator gets time to APPROVE before enforcement bites. Once §14.4 is anchored, enforcement is real.

**Why `memory.acl_denied` aux row even on WARN-ONLY (§1 #14).** The aux row is the audit signal. Operators inspecting "what would have been denied?" need a queryable log, not just stderr noise. Once enforcement bites, the aux rows are still useful — they record blocked attempts for post-hoc analysis.

**Why `move` checks both paths (§1 #8).** A `move(src, dst)` is conceptually a `delete(src) + put(dst)`. If `src` is in a read-only store, the delete half should fail; if `dst` is in a deny store, the put half should fail. Both checks catch operators who try to "smuggle" content across stores via move. The error names the failing side.

**Why mtime-keyed cache (§1 #9).** STORE.yaml files are rarely changed but frequently consulted. In-memory cache avoids re-parsing YAML on every write. mtime-based invalidation matches how `cyberos doctor` and FR-MEMORY-107 (FS watcher) detect external mutations.

---

## §3 — API contract

### Schema

```json
{
  "$defs": {
    "StoreAclMode": {
      "type": "string",
      "enum": ["read", "read-write", "deny"]
    },
    "StoreAclEntry": {
      "type": "object",
      "required": ["actor", "mode"],
      "properties": {
        "actor": {"type": "string", "minLength": 1},
        "mode":  {"$ref": "#/$defs/StoreAclMode"}
      }
    },
    "StoreAcl": {
      "type": "object",
      "required": ["store_id", "acl"],
      "properties": {
        "store_id":     {"type": "string", "pattern": "^[a-z][a-z0-9_-]{0,63}$"},
        "default_mode": {"$ref": "#/$defs/StoreAclMode", "default": "read-write"},
        "acl":          {"type": "array", "items": {"$ref": "#/$defs/StoreAclEntry"}}
      }
    }
  }
}
```

### Resolver

```python
# modules/memory/cyberos/core/store_acl.py
from __future__ import annotations
import fnmatch
from dataclasses import dataclass
from pathlib import Path
from typing import Literal, Optional
import yaml

Mode = Literal["read", "read-write", "deny"]


@dataclass(frozen=True)
class StoreAcl:
    store_id:     str
    default_mode: Mode
    acl:          tuple[tuple[str, Mode], ...]    # ordered, first-match-wins

    @classmethod
    def from_yaml(cls, path: Path) -> "StoreAcl":
        raw = yaml.safe_load(path.read_text())
        if not isinstance(raw, dict):
            raise ValueError(f"{path}: must be a YAML object")
        store_id     = raw["store_id"]
        default_mode = raw.get("default_mode", "read-write")
        entries      = raw.get("acl", [])
        return cls(
            store_id=store_id, default_mode=default_mode,
            acl=tuple((e["actor"], e["mode"]) for e in entries),
        )

    def resolve_mode(self, actor: str) -> Mode:
        for pattern, mode in self.acl:
            if pattern == "*" or pattern == actor or fnmatch.fnmatchcase(actor, pattern):
                return mode
        return self.default_mode


def find_governing_store_yaml(memory_root: Path, target: Path) -> Optional[Path]:
    """Walk UP from target until we find STORE.yaml or hit memory_root."""
    current = (memory_root / target).resolve().parent
    memory_root = memory_root.resolve()
    while current >= memory_root and current.is_relative_to(memory_root):
        candidate = current / "STORE.yaml"
        if candidate.exists():
            return candidate
        if current == memory_root:
            break
        current = current.parent
    return None


@dataclass(frozen=True)
class AclResult:
    allowed:        bool
    mode:           Mode
    store_id:       Optional[str]
    yaml_path:      Optional[str]
    matched_entry:  Optional[str]      # e.g. "actor=stephen@*"
    reason:         Optional[str]


def check_write(memory_root: Path, target: Path, actor: str, warn_only: bool) -> AclResult:
    yml = find_governing_store_yaml(memory_root, target)
    if yml is None:
        return AclResult(allowed=True, mode="read-write", store_id=None,
                         yaml_path=None, matched_entry=None,
                         reason="no STORE.yaml — permissive default")
    acl = StoreAcl.from_yaml(yml)
    mode = acl.resolve_mode(actor)
    if mode == "read-write":
        return AclResult(allowed=True, mode=mode, store_id=acl.store_id,
                         yaml_path=str(yml.relative_to(memory_root)),
                         matched_entry=f"actor matched against acl",
                         reason=None)
    return AclResult(
        allowed=warn_only,           # WARN-ONLY → "allowed=True with reason"
        mode=mode,
        store_id=acl.store_id,
        yaml_path=str(yml.relative_to(memory_root)),
        matched_entry=f"actor={actor!r}",
        reason=f"acl_denied:mode={mode}" if not warn_only else f"warn_only:mode={mode}",
    )
```

### AGENTS.md §14.4 amendment text (proposed)

```text
## §14.4  Store-level ACL (added by P20 — requires APPROVE chat-turn per §0.2)

§14.4.1  Each subtree of `<memory-root>/` MAY declare a `STORE.yaml` file
at its root. The file's shape is normative in `memory.schema.json#/$defs/StoreAcl`.
Subtrees without a `STORE.yaml` inherit the default permissive policy
(`{actor: "*", mode: "read-write"}`).

§14.4.2  The canonical writer (`cyberos.core.writer.Writer`) MUST enforce
ACLs on every `put` / `move` / `delete` operation. ACL is resolved by
walking up from the target path to the nearest `STORE.yaml`. First-match-wins
on the `acl` list. Explicit `deny` always blocks regardless of subsequent
allow patterns.

§14.4.3  Reads are NOT subject to ACL enforcement at the protocol level.
Read isolation is the operator's responsibility via OS filesystem permissions.

§14.4.4  Rejected writes MUST emit a `memory.acl_denied` aux audit row with
payload `{actor, target_path, store_id, yaml_path, mode, matched_entry,
attempt_kind: "put"|"move"|"delete"}`. The audit row is emitted even in
WARN-ONLY mode (where rejection is logged but write proceeds).

§14.4.5  `move(src, dst)` MUST check ACL on BOTH `src` and `dst`. Either
side failing blocks the operation.

§14.4.6  The `INTEROP.md` consumer subset MUST honour `STORE.yaml` `acl`
for writes; reads MAY ignore.
```

---

## §4 — Acceptance criteria

1. **Permissive default when no STORE.yaml** — write to `memories/x.md` with no STORE.yaml anywhere → succeeds; AclResult `allowed: true`. *(traces_to: §1 #3, DEC-230)*
2. **Wildcard actor `*` matches** — STORE.yaml `[{actor: "*", mode: "read-write"}]`; any actor succeeds. *(traces_to: §1 #5, §1 #6)*
3. **Read-only mode rejects write** — STORE.yaml `[{actor: "*", mode: "read"}]`; write attempt → AclResult `allowed: false`, mode `read`; `memory.acl_denied` aux row emitted. *(traces_to: §1 #7)*
4. **Explicit deny overrides allow** — `[{actor: "scheduled-importer", mode: "deny"}, {actor: "*", mode: "read-write"}]`; actor=`scheduled-importer` rejected. *(traces_to: §1 #5)*
5. **Glob actor matching** — `[{actor: "stephen@*", mode: "read-write"}, ...]`; actor=`stephen@example.com` matches; actor=`alice@example.com` doesn't. *(traces_to: §1 #5, §1 #6)*
6. **Closest STORE.yaml wins** — `memories/STORE.yaml` denies; `memories/episodes/STORE.yaml` allows; write to `memories/episodes/x.md` → allowed (closest wins). *(traces_to: §1 #3)*
7. **First-match-wins on acl list** — `[{actor: "*", mode: "read"}, {actor: "stephen", mode: "read-write"}]`; actor=stephen rejected (first wildcard matches first). *(traces_to: §1 #5)*
8. **Default_mode applied when no entry matches** — `default_mode: deny`; acl `[{actor: "stephen", mode: "read-write"}]`; actor=alice → deny. *(traces_to: §1 #4)*
9. **Reject emits aux row** — every denied write produces exactly one `memory.acl_denied` row with the documented payload shape. *(traces_to: §1 #7)*
10. **Move respects both paths** — src in read-only store, dst in read-write store → reject (src side fails); src in r-w, dst in deny → reject (dst side fails). *(traces_to: §1 #8)*
11. **STORE.yaml mtime cache invalidation** — write succeeds; STORE.yaml edited to deny; next write fails (cache invalidated on mtime change). *(traces_to: §1 #9)*
12. **Migration script idempotent** — existing STORE.yaml not overwritten; missing ones populated with permissive default. Re-run is no-op. *(traces_to: §1 #10)*
13. **Migration --dry-run** — `migrate-store-acl.py --dry-run` prints planned writes but creates no files. *(traces_to: §1 #10)*
14. **STORE.yaml shape invalid → walker error** — handcraft a malformed yaml → `cyberos doctor` fails with `store-yaml-acl-valid`. *(traces_to: §1 #11)*
15. **`cyberos acl show`** — lists store_id + acl for each STORE.yaml in the memory. *(traces_to: §1 #12)*
16. **`cyberos acl explain <path>`** — for a given path + active actor, outputs the resolved mode + matched entry. *(traces_to: §1 #12)*
17. **WARN-ONLY without §14.4 amendment** — AGENTS.md lacks §14.4; write that would be denied → writes proceed (allowed=True due to warn_only); `memory.acl_denied` aux row still emitted with `reason: "warn_only:..."`. *(traces_to: §1 #13, §1 #14)*
18. **Enforcement when §14.4 present** — AGENTS.md has §14.4 anchor; same denied write → writes refused. *(traces_to: §1 #13)*
19. **Built-in actor literals** — `dream-runner`, `dream-applier`, `scheduled-importer`, `claude-code-hook` all match by literal name without glob. *(traces_to: §1 #6)*
20. **Symlinked memory_root** — STORE.yaml lookup follows symlinks to find the real path. *(traces_to: §1 #3)*

---

## §5 — Verification

```python
# modules/memory/tests/core/test_store_acl.py
import pytest, yaml
from pathlib import Path
from cyberos.core.store_acl import StoreAcl, check_write, find_governing_store_yaml


def write_store_yaml(p: Path, acl, default="read-write", store_id="test"):
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(yaml.safe_dump({"store_id": store_id, "default_mode": default, "acl": acl}))


def test_permissive_default(tmp_memory):
    """AC #1"""
    res = check_write(tmp_memory, Path("memories/x.md"), actor="stephen", warn_only=False)
    assert res.allowed is True
    assert res.yaml_path is None


def test_wildcard_actor(tmp_memory):
    """AC #2"""
    write_store_yaml(tmp_memory / "memories/STORE.yaml",
                     [{"actor": "*", "mode": "read-write"}])
    res = check_write(tmp_memory, Path("memories/x.md"), actor="stephen", warn_only=False)
    assert res.allowed is True


def test_read_only_blocks_write(tmp_memory):
    """AC #3"""
    write_store_yaml(tmp_memory / "memories/STORE.yaml",
                     [{"actor": "*", "mode": "read"}])
    res = check_write(tmp_memory, Path("memories/x.md"), actor="stephen", warn_only=False)
    assert res.allowed is False
    assert res.mode == "read"


def test_explicit_deny_overrides_allow(tmp_memory):
    """AC #4"""
    write_store_yaml(tmp_memory / "memories/STORE.yaml", [
        {"actor": "scheduled-importer", "mode": "deny"},
        {"actor": "*",                  "mode": "read-write"},
    ])
    res = check_write(tmp_memory, Path("memories/x.md"),
                      actor="scheduled-importer", warn_only=False)
    assert res.allowed is False
    assert res.mode == "deny"


def test_glob_actor_matching(tmp_memory):
    """AC #5"""
    write_store_yaml(tmp_memory / "memories/STORE.yaml", [
        {"actor": "stephen@*", "mode": "read-write"},
        {"actor": "*",         "mode": "read"},
    ])
    assert check_write(tmp_memory, Path("memories/x.md"),
                       actor="stephen@example.com", warn_only=False).allowed
    assert not check_write(tmp_memory, Path("memories/x.md"),
                            actor="alice@example.com", warn_only=False).allowed


def test_closest_store_yaml_wins(tmp_memory):
    """AC #6"""
    write_store_yaml(tmp_memory / "memories/STORE.yaml",
                     [{"actor": "*", "mode": "read"}])           # outer = read
    write_store_yaml(tmp_memory / "memories/episodes/STORE.yaml",
                     [{"actor": "*", "mode": "read-write"}])     # inner = r-w
    res = check_write(tmp_memory, Path("memories/episodes/x.md"),
                      actor="stephen", warn_only=False)
    assert res.allowed is True
    assert "episodes" in res.yaml_path


def test_first_match_wins(tmp_memory):
    """AC #7"""
    write_store_yaml(tmp_memory / "memories/STORE.yaml", [
        {"actor": "*",       "mode": "read"},
        {"actor": "stephen", "mode": "read-write"},
    ])
    res = check_write(tmp_memory, Path("memories/x.md"), actor="stephen", warn_only=False)
    # First entry (wildcard read) wins, not the more-specific stephen entry
    assert res.allowed is False


def test_default_mode_applied(tmp_memory):
    """AC #8"""
    write_store_yaml(tmp_memory / "memories/STORE.yaml",
                     [{"actor": "stephen", "mode": "read-write"}],
                     default="deny")
    res = check_write(tmp_memory, Path("memories/x.md"), actor="alice", warn_only=False)
    assert res.allowed is False
    assert res.mode == "deny"


def test_warn_only_mode_allows_but_emits_aux(tmp_memory):
    """AC #17"""
    write_store_yaml(tmp_memory / "memories/STORE.yaml",
                     [{"actor": "*", "mode": "read"}])
    res = check_write(tmp_memory, Path("memories/x.md"), actor="stephen", warn_only=True)
    # In WARN-ONLY mode, allowed=True even when mode is read; reason names warn_only
    assert res.allowed is True
    assert res.reason.startswith("warn_only:")


def test_built_in_actors_match_literally(tmp_memory):
    """AC #19"""
    write_store_yaml(tmp_memory / "memories/STORE.yaml", [
        {"actor": "dream-runner", "mode": "read-write"},
        {"actor": "*", "mode": "read"},
    ])
    res = check_write(tmp_memory, Path("memories/x.md"),
                      actor="dream-runner", warn_only=False)
    assert res.allowed is True
```

```python
# modules/memory/tests/core/test_store_acl.py
import pytest, yaml, subprocess
from pathlib import Path


def test_migration_creates_permissive_default(tmp_memory):
    """AC #12"""
    (tmp_memory / "memories").mkdir(exist_ok=True)
    (tmp_memory / "meta").mkdir(exist_ok=True)
    subprocess.run(["python", "scripts/migrate-store-acl.py",
                    "--store", str(tmp_memory)], check=True)
    for sub in ("memories", "meta"):
        yml = tmp_memory / sub / "STORE.yaml"
        assert yml.exists()
        body = yaml.safe_load(yml.read_text())
        assert body["acl"][0]["mode"] == "read-write"


def test_migration_idempotent(tmp_memory):
    """AC #12"""
    (tmp_memory / "memories").mkdir(exist_ok=True)
    (tmp_memory / "memories/STORE.yaml").write_text(yaml.safe_dump({
        "store_id": "custom",
        "default_mode": "deny",
        "acl": [{"actor": "stephen", "mode": "read-write"}],
    }))
    subprocess.run(["python", "scripts/migrate-store-acl.py",
                    "--store", str(tmp_memory)], check=True)
    body = yaml.safe_load((tmp_memory / "memories/STORE.yaml").read_text())
    # Existing config NOT overwritten
    assert body["store_id"] == "custom"
    assert body["default_mode"] == "deny"


def test_migration_dry_run(tmp_memory):
    """AC #13"""
    (tmp_memory / "memories").mkdir(exist_ok=True)
    result = subprocess.run(["python", "scripts/migrate-store-acl.py",
                             "--store", str(tmp_memory), "--dry-run"],
                            capture_output=True, text=True, check=True)
    assert not (tmp_memory / "memories/STORE.yaml").exists()
    assert "would create" in result.stdout


def test_invalid_store_yaml_walker_error(tmp_memory):
    """AC #14"""
    (tmp_memory / "memories").mkdir(exist_ok=True)
    (tmp_memory / "memories/STORE.yaml").write_text(":::not valid yaml:::")
    result = subprocess.run(["python", "-m", "cyberos", "--store", str(tmp_memory),
                             "doctor"], capture_output=True, text=True)
    assert result.returncode != 0
    assert "store-yaml-acl-valid" in (result.stderr + result.stdout)
```

---

## §6 — Implementation skeleton

API contracts above are the skeleton. Order:

1. AGENTS.md §14.4 amendment text (DO NOT commit until APPROVE chat-turn).
2. Schema (`memory.schema.json`).
3. Walker invariant.
4. `cyberos/core/store_acl.py` resolver.
5. `cyberos/core/writer.py` integration with WARN-ONLY mode toggle on §14.4 anchor check.
6. Migration script.
7. `cyberos acl` CLI subcommands.
8. Tests.
9. INTEROP.md one-liner.
10. CHANGELOG.

---

## §7 — Dependencies

- **FR-MEMORY-115 (related)** — dream rows respect store ACL; dream-runner / dream-applier are built-in actor literals.
- **FR-MEMORY-118 (this FR blocks)** — `put_if` precondition-hash operates within ACL constraints (rejected writes don't proceed regardless of precondition match).
- **FR-MEMORY-103 (transitively related)** — multi-device sync respects ACL on import; foreign-chain rows arriving via sync get the `imported` actor literal.
- **FR-MEMORY-106 (related)** — sync_class is orthogonal to ACL; both can be active simultaneously.

---

## §8 — Example payloads

### `STORE.yaml` for SRE-style demo

```yaml
# memories/org-wide-knowledge/STORE.yaml
store_id: org-wide-knowledge
default_mode: read
acl:
  - {actor: "stephen@cyberskill.world", mode: "read-write"}
  - {actor: "dream-applier",            mode: "read-write"}
  - {actor: "*",                        mode: "read"}
```

```yaml
# memories/sre/STORE.yaml
store_id: sre-working
default_mode: read-write
acl:
  - {actor: "scheduled-importer", mode: "deny"}
  - {actor: "*",                  mode: "read-write"}
```

### `memory.acl_denied` aux row

```json
{
  "kind": "memory.acl_denied",
  "payload": {
    "actor":         "scheduled-importer",
    "target_path":   "memories/sre/dispatch-1.md",
    "store_id":      "sre-working",
    "yaml_path":     "memories/sre/STORE.yaml",
    "mode":          "deny",
    "matched_entry": "actor='scheduled-importer'",
    "attempt_kind":  "put",
    "warn_only":     false
  }
}
```

### `cyberos acl explain` output

```text
$ cyberos acl explain memories/sre/dispatch-1.md --actor scheduled-importer
Path:             memories/sre/dispatch-1.md
Governing yaml:   memories/sre/STORE.yaml (store_id: sre-working)
Effective mode:   deny
Matched entry:    {actor: "scheduled-importer", mode: "deny"} [first match]
Result:           writes WILL be rejected
```

---

## §9 — Open questions

All resolved. Deferred:
- `default_mode: deny` general support — §1 #15; slice 4 (requires more design around the "secrets/" subtree pattern).
- `cyberos acl audit --since 24h` — §1 #16; slice 4 polish.
- Per-store rate limits — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| §14.4 missing | Writer construction check | WARN-ONLY mode active | Operator runs APPROVE chat-turn |
| STORE.yaml malformed | walker invariant | `cyberos doctor` non-zero | Operator fixes file |
| Glob pattern catastrophic | fnmatch is bounded | normal speed | None — by design |
| Actor not matching any entry, no default_mode | schema default = "read-write" | permissive default | None |
| Migration script crash mid-pass | per-file write is atomic | partial migration; re-run picks up remaining | None — idempotent |
| Symlinked memory_root | `resolve()` follows symlinks | works | None |
| STORE.yaml present but no `acl` array | jsonschema rejects | walker invariant fails | Operator adds at least one entry |
| Concurrent edit to STORE.yaml during write | mtime cache invalidates on next access | new policy applied to next write | None |
| ACL entry with mode outside enum | jsonschema rejects | walker fails | Operator fixes |
| Empty `acl: []` with default_mode | resolver returns default_mode | works | None |
| Recursive symlink in `<memory-root>/` | `resolve()` raises | walker fails | Operator fixes filesystem |
| Operator forgets actor flag → defaults to "agent" | normal resolution | matches against "agent" pattern in ACL | None |

---

## §11 — Implementation notes

- **fnmatch with `case`-sensitive matching** — actor strings are case-sensitive by spec. `fnmatchcase` (not `fnmatch`) enforces.
- **Cache invalidation via mtime** — simple; the FS watcher in FR-MEMORY-107 doesn't need to know about ACL caching.
- **Migration script lives at `scripts/` not `cyberos/`** — it's an operator tool, not part of the runtime.
- **WARN-ONLY mode is the bridge** — operators can install this FR before the APPROVE chat-turn and see what would happen; once they're satisfied, they APPROVE and enforcement engages.
- **Reads NOT enforced — by design.** This is the most surprising design choice. Operator wanting read isolation uses OS file permissions. The protocol's contract: "writes are protocol-controlled; reads are filesystem-controlled."
- **No new audit kind for ACL grants/revokes** — STORE.yaml edits are just file edits; FR-MEMORY-107's FS watcher already captures them as `put` rows on the path. The walker invariant validates the resulting shape.
- **`Writer.__init__` reads AGENTS.md once at construction** to set `warn_only`. Re-reading on every write would be wasteful; restart-after-APPROVE is the contract.
- **The migration script writes one STORE.yaml per top-level directory existing at migration time, NOT per any-depth directory.** This matches DEC-230's intent — operators want a small finite set of stores, not "every leaf has its own."

---

*End of FR-MEMORY-117.*

## As built (2026-07-02)

ACL logic spans services/memory + modules/memory as consolidated.
