---
id: TASK-MEMORY-119
title: "memory session transcript ledger — opt-in `cyberos session {start,append,end}` writes turn-level transcript rows under sessions/<date>/<id>.binlog.zst; default classification=confidential; configurable retention; feeds TASK-MEMORY-115 dream"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-19T00:00:00+07:00
department: engineering
author: "@stephencheng"
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
related_tasks: [TASK-MEMORY-109, TASK-MEMORY-111, TASK-MEMORY-115, TASK-MEMORY-117]
depends_on: []
blocks: [TASK-MEMORY-115]
protocol_amendment_required: "AGENTS.md §18 (new) — session-transcript ledger; approval phrase: APPROVE protocol change P22 §18"

source_pages:
  # see "input sessions" segment that dream consumes
  - playground/extracts/memory-and-dreaming.transcript.txt
source_decisions:
  - DEC-250 (Session transcript is OPT-IN — not every cyberos invocation creates a session; opt in via `cyberos session start --id <slug>`)
  - DEC-251 (Default `classification: confidential` per Stephen's 2026-05-19 decision — encryption recommended but not required; operators can pin to `restricted` to force encryption envelope)
  - DEC-252 (Retention default = 30 days; configurable via `manifest.json:sessions.retention_days`; after expiry, body is purged with the same provenance pattern as `delete(purge)`)
  - DEC-253 (Sessions are append-only conceptually; turns flow `session.start → session.turn × N → session.end`; out-of-order or missing-end transitions are detectable by walker invariant)

language: python 3.10
service: modules/memory/cyberos/
new_files:
  - modules/memory/cyberos/core/session.py
  - modules/memory/cyberos/cli/session.py
  - modules/memory/tests/core/test_transcript.py
  - modules/memory/tests/core/test_session.py
modified_files:
  # wire `cyberos session start|append|end|read|list|purge-expired` subcommands
  - modules/memory/cyberos/__main__.py
  # add session-aware emit path; sessions/<date>/<id>.binlog.zst storage
  - modules/memory/cyberos/core/writer.py
  # `SessionFrontmatter` + `SessionTurnPayload` + `SessionAuditKind` definitions
  - modules/memory/memory.schema.json
  # `session-lifecycle-well-formed` + `session-classification-valid` rules
  - modules/memory/memory.invariants.yaml
  # add §18 — sessions (REQUIRES amendment via APPROVE chat-turn P22 §18)
  - AGENTS.md
allowed_tools:
  - file_read: modules/memory/**
  - file_write: modules/memory/cyberos/**, modules/memory/tests/**, modules/memory/memory.schema.json, modules/memory/memory.invariants.yaml, AGENTS.md
  - bash: cd modules/memory && python -m pytest tests/test_session_*.py -v
  - bash: cd modules/memory && python -m cyberos session start --id smoke-test && python -m cyberos session append --id smoke-test --role user --content "hello" && python -m cyberos session end --id smoke-test
disallowed_tools:
  #6 — walker rejects)
  - emit session.turn rows without a preceding session.start for that id (per §1
  #4)
  - write session bodies in plaintext when classification: restricted is configured (per task §5.4 + §1
  - mutate AGENTS.md §18 without APPROVE protocol change P22 §18 chat-turn

effort_hours: 24
subtasks:
  - "1.0h: AGENTS.md §18 amendment text drafted (requires Stephen APPROVE chat-turn)"
  - "1.5h: memory.schema.json — `SessionFrontmatter` (id, started_at, ended_at?, classification, retention_days), `SessionTurnPayload` (role enum, content, turn_seq, ts, redactions_applied?), `SessionAuditKind` enum (session.start, session.turn, session.end, session.purged)"
  - "1.0h: memory.invariants.yaml — `session-lifecycle-well-formed` (error: every session.start matched by zero-or-one session.end; turn_seq monotone increasing per session) + `session-classification-valid` (error: enum)"
  - "3.0h: cyberos/core/session.py — `Session` dataclass + `start(writer, id, classification, retention_days)`, `append(writer, id, role, content)`, `end(writer, id, reason?)`; storage at `<root>/sessions/<YYYY-MM-DD>/<id>.binlog.zst`; turn rows compressed with zstd"
  - "2.0h: cyberos/cli/session.py — full subcommand surface (start, append, end, read, list, purge-expired)"
  - "1.5h: writer.py — session-aware emit path; reuse task §3 atomic-write + §4 lock; sessions chain INDEPENDENTLY from the main audit chain (own binlog segment) but emit summary rows (session.start, session.end) onto the main chain for TASK-MEMORY-115 dream consumption"
  - "1.5h: encryption integration — when classification: restricted, body of each turn is encrypted per §5.4 envelope before storage; meta sidecar plaintext per §5.4"
  - "2.0h: purge job — `cyberos session purge-expired` scans manifest retention_days and purges expired sessions with `delete(purge)` semantics on the chain"
  - "3.0h: modules/memory/tests/core/test_transcript.py — 18 cases (full lifecycle, append-without-start rejected, out-of-order rejected, dual-end rejected, encryption when restricted, read round-trip, list-by-date, classification override per-call)"
  - "1.5h: modules/memory/tests/core/test_session.py — 8 cases (purge-expired removes session body, audit row of purge emitted, retention overridden by --retain flag, expired but unpurged read fails)"
  - "1.0h: CLI integration test against the seeded memory"
  - "0.5h: CHANGELOG entry + AGENTS.md cross-ref to TASK-MEMORY-115 #1 #3 input-sessions"
risk_if_skipped: "Without the session transcript ledger, TASK-MEMORY-115 dream's `patterns` detector operates only on `episode.logged` aux rows and memory bodies — not on the conversational context that produced them. The talk's headline use case (`5 agents all hit the same 60-second retry pattern across sessions`) is detectable from episodes alone, but the richer 'why' (specific phrasing of error messages, sequence of tool calls, context the operator gave) lives in transcripts. The talk's product positions transcripts as the primary input to dreaming. Without TASK-MEMORY-119, TASK-MEMORY-115 ships at slice-3 with degraded pattern-detection quality; the gap closes when transcripts feed in. Additionally: without transcripts the memory has no audit trail for 'what did the agent actually say to the user that led to this memory write?' — critical for TASK-MEMORY-115 stale-detection (the most powerful signal for marking memory stale is 'the agent said this fact, the user corrected it, the next write reflected the correction')."
---

## §1 — Description (BCP-14 normative)

The session-transcript ledger is an **opt-in turn-level audit trail** for agent-user conversations. Sessions are stored independently from the main audit chain (separate `sessions/` directory) but emit summary rows on the main chain so downstream consumers (TASK-MEMORY-115 dream) can discover them. The contract:

1. **MUST** support three CLI lifecycle commands:
- `cyberos session start --id <slug> [--classification confidential|restricted] [--retention-days <N>]` — emits `session.start` row on the main chain + creates `sessions/<YYYY-MM-DD>/<id>.binlog.zst`
- `cyberos session append --id <slug> --role {user|assistant|system|tool} --content <text>` — emits `session.turn` row into the session's own binlog
- `cyberos session end --id <slug> [--reason <text>]` — emits `session.end` row on the main chain + seals the session's binlog (compresses, computes final chain hash, marks immutable)
2. **MUST** store session bodies at `<memory-root>/sessions/<YYYY-MM-DD>/<id>.binlog.zst` — date-partitioned for easy retention purge. The binlog format mirrors §6.2 (length-prefixed framed records) so the existing reader code paths can parse it.
3. **MUST** default `classification: confidential` per DEC-251. Operators MAY override per-session with `--classification restricted` to force encryption envelope per §5.4. The `public` and `internal` classifications are NOT permitted on sessions (they'd undermine the whole purpose of the transcript being sensitive).
4. **MUST** when `classification: restricted`, encrypt every `session.turn` payload's `content` field via the §5.4 envelope. The meta-frame (role, ts, turn_seq) stays plaintext for fast filtering without decryption.
5. **MUST** validate session lifecycle invariants via walker rule `session-lifecycle-well-formed`:
- Every `session.start` matched by 0 or 1 `session.end` (in-flight sessions are valid; double-end is not)
- Within one session's binlog: `turn_seq` strictly monotonically increasing from 0 to N
- No `session.turn` row precedes its `session.start` or follows its `session.end`
6. **MUST** reject `append` for a session id that has no preceding `start` OR has a preceding `end`. Error message names the violated state.
7. **MUST** support `cyberos session read --id <slug> [--decrypt]` to render a session's transcript as a sequence of turn entries. `--decrypt` is required for `restricted` sessions; without the flag, restricted sessions display turn metadata only ("[encrypted content; --decrypt to read]").
8. **MUST** support `cyberos session list [--since 24h] [--classification ...] [--ended {true|false|all}]` to enumerate sessions. Default returns last 24h of all sessions.
9. **MUST** support retention via `manifest.json:sessions.retention_days` (default = 30 per DEC-252). After expiry, `cyberos session purge-expired` (run on cron) drops the body via `delete(purge)` semantics — the session.start / session.end summary rows on the main chain remain (with the body's hash as proof-of-content-having-existed); the session's `.binlog.zst` file is overwritten with a tombstone manifest.
10. **MUST** emit `session.purged` audit row on the main chain when a session's body is dropped. Payload: `{session_id, original_started_at, original_ended_at, turns_count, purged_at, reason: "retention_expired"|"manual"}`.
11. **MUST** honour TASK-MEMORY-117 store-ACL for the `sessions/` subtree — operators can set a `sessions/STORE.yaml` to restrict which actors can `start`/`append`/`end` sessions. Reads remain unrestricted at the protocol level (same DEC-232).
12. **MUST** require the AGENTS.md §18 amendment APPROVED via `APPROVE protocol change P22 §18` chat-turn before sessions can be started. Writer checks for the §18 anchor at construction; absent → `cyberos session start` raises with the structured error.
13. **MUST** include a `session_id` field on every memory write that originated during an active session (i.e. the agent was in a session AND made memory writes during it). The writer reads the current active session id from `<memory-root>/sessions/.active` (a lock-file-style pointer) and attaches it as `extra.session_id` on the put/move/delete row. This binds memory writes to the transcript that produced them.
14. **MUST** allow `cyberos session append` to flag specific turn content as needing PII redaction by setting `--redactions-applied true`; the TASK-MEMORY-111 PII detector is invoked on the content before storage. The redaction outcome is recorded in the turn payload's `redactions_applied` field.
15. **SHOULD** support `cyberos session export --id <slug> --format jsonl` for cross-system handoff. Slice-4 stretch.
16. **SHOULD** support multi-session correlation via `--correlates-with <other-id>` to link related sessions (e.g. a multi-step task spans two sessions). Slice-4+ stretch.

---

## §2 — Why this design (rationale for humans)

**Why opt-in, not always-on (§1 #1, DEC-250).** Sessions add storage cost and operator complexity. Most cyberos invocations (single `put`, doctor checks, exports) are NOT conversations — there's nothing to transcribe. Forcing session lifecycle on every invocation is overhead the operator didn't ask for. Opt-in keeps the simple case simple; conversation-aware agents that NEED transcripts call `session start` explicitly.

**Why default classification: confidential (§1 #3, DEC-251).** Stephen explicitly chose this on 2026-05-19. Sessions contain raw user-agent dialogue; even when the topic is mundane, the dialogue itself is private context (who asked, when, what they wanted). `confidential` strikes the right balance: encryption envelope RECOMMENDED (§5.4) but not REQUIRED — operators can encrypt for high-stakes deployments, leave plaintext for casual local use. `restricted` (force-encrypt) is one flag away for sensitive deployments.

**Why date-partitioned sessions directory (§1 #2).** Three reasons. (a) Retention purge is `rm -rf sessions/2026-04-19/` once a date's retention expires — atomic and fast. (b) Disk usage by date is the natural operator question. (c) The session id is unique within a date; multiple operators on the same machine can't collide unless they pick the same id on the same day.

**Why separate binlog instead of inlining into main audit (§1 #2, §1 #5).** Two reasons. (a) Cardinality — a 1-hour conversation can produce 100+ turn rows; inlining would balloon the main chain and slow walker. (b) Privacy — sessions are confidential by default; the main chain might be shared (via `cyberos export`) while sessions are kept local. Separation enables differential sharing.

**Why summary rows on main chain (§1 #1, §1 #10).** TASK-MEMORY-115 dream needs to *discover* sessions — it can't scan `sessions/` directly because it operates on chain rows. The summary rows (`session.start`, `session.end`, `session.purged`) live on the main chain and carry enough metadata (id, classification, turn count) for dream to decide whether to dive into a session's binlog for context.

**Why lifecycle invariants enforced by walker (§1 #5, §1 #6, DEC-253).** Append-without-start would create orphaned transcript fragments — no way to know whose conversation they came from. Double-end would be ambiguous about which `end` row "won." Walker enforcement catches operator typos at `cyberos doctor` time rather than at dream-runner time (when the failure mode would be obscure).

**Why `extra.session_id` on memory writes during active session (§1 #13).** Closes the loop between transcript and memory. When the agent makes a `put` mid-conversation, the resulting audit row carries the session id; TASK-MEMORY-120's `cyberos history <path>` can then say "this memory was written during session X turn 17" — directly answering "what was the agent thinking when it wrote this?" The pointer file `sessions/.active` is the simplest implementation; the alternative (passing session_id through every CLI call) is fragile.

**Why retention default 30 days (DEC-252).** Long enough for TASK-MEMORY-115 dream to consume sessions on a daily cadence + retrospective debugging up to a month back. Short enough to limit privacy exposure. Operators can override per-store via manifest; high-compliance deployments set 7 days, research deployments set 365.

**Why `session.purged` row on main chain (§1 #10).** Audit-trail completeness. Operators reviewing "did the agent have access to private user info on date X?" need to answer "is the transcript still readable?" Even after purge, the *fact* of purge is on the chain — same pattern as `delete(purge)` from §3.6.

---

## §3 — API contract

### Schema fragment

```json
{
  "$defs": {
    "SessionClassification": {
      "type": "string",
      "enum": ["confidential", "restricted"]
    },
    "SessionFrontmatter": {
      "type": "object",
      "required": ["id", "started_at", "classification"],
      "properties": {
        "id":              {"type": "string", "pattern": "^[a-z][a-z0-9_-]{0,63}$"},
        "started_at":      {"type": "string", "format": "date-time"},
        "ended_at":        {"type": "string", "format": "date-time"},
        "ended_reason":    {"type": "string"},
        "classification":  {"$ref": "#/$defs/SessionClassification"},
        "retention_days":  {"type": "integer", "minimum": 1, "default": 30},
        "actor":           {"type": "string"},
        "correlates_with": {"type": "string"}
      }
    },
    "SessionTurnPayload": {
      "type": "object",
      "required": ["session_id", "role", "turn_seq", "ts"],
      "properties": {
        "session_id":         {"type": "string"},
        "role":               {"type": "string", "enum": ["user", "assistant", "system", "tool"]},
        "content":            {"type": "string"},
        "content_cipher":     {"$ref": "#/$defs/Envelope"},
        "turn_seq":           {"type": "integer", "minimum": 0},
        "ts":                 {"type": "string", "format": "date-time"},
        "redactions_applied": {"type": "boolean"}
      },
      "oneOf": [
        {"required": ["content"]},
        {"required": ["content_cipher"]}
      ]
    },
    "SessionAuditKind": {
      "type": "string",
      "enum": ["session.start", "session.end", "session.purged"]
    }
  }
}
```

### Session module

```python
# modules/memory/cyberos/core/session.py
from __future__ import annotations
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Literal, Optional
import zstandard

from cyberos.core.writer import Writer

Classification = Literal["confidential", "restricted"]
Role = Literal["user", "assistant", "system", "tool"]


@dataclass
class Session:
    id:             str
    started_at:     datetime
    classification: Classification
    retention_days: int          = 30
    actor:          str          = "agent"
    ended_at:       Optional[datetime] = None
    correlates_with: Optional[str] = None


def start(writer: Writer, id: str, classification: Classification = "confidential",
          retention_days: int = 30, actor: str = "agent") -> Session:
    writer._require_protocol_amendment_p22()          # §1 #12
    if (writer.store_path / "sessions" / ".active").exists():
        raise RuntimeError("a session is already active; end it before starting another")

    s = Session(id=id, started_at=datetime.now(timezone.utc),
                classification=classification, retention_days=retention_days,
                actor=actor)

    # date-partitioned storage
    date_dir = writer.store_path / "sessions" / s.started_at.strftime("%Y-%m-%d")
    date_dir.mkdir(parents=True, exist_ok=True)
    binlog_path = date_dir / f"{id}.binlog.zst"
    if binlog_path.exists():
        raise RuntimeError(f"session id {id!r} already exists on {s.started_at.date()}")

    # Empty binlog (will be populated by append)
    binlog_path.write_bytes(b"")

    # Pointer to active session
    (writer.store_path / "sessions" / ".active").write_text(id)

    # Summary row on main chain
    writer.emit_aux(
        kind="session.start",
        payload={
            "session_id":     id,
            "started_at":     s.started_at.isoformat(),
            "classification": classification,
            "retention_days": retention_days,
            "actor":          actor,
        },
        actor=actor,
    )
    return s


def append(writer: Writer, id: str, role: Role, content: str,
           redactions_applied: Optional[bool] = None) -> int:
    active = (writer.store_path / "sessions" / ".active")
    if not active.exists():
        raise RuntimeError("no active session; start one first")
    if active.read_text() != id:
        raise RuntimeError(f"active session is {active.read_text()!r}, not {id!r}")

    s = _load_session_state(writer, id)
    if s.ended_at is not None:
        raise RuntimeError(f"session {id!r} has ended")

    turn_seq = _next_turn_seq(writer, id)
    ts = datetime.now(timezone.utc).isoformat()

    payload = {
        "session_id": id, "role": role, "turn_seq": turn_seq, "ts": ts,
    }
    if s.classification == "restricted":
        payload["content_cipher"] = _encrypt(content, writer)
    else:
        payload["content"] = content
    if redactions_applied is not None:
        payload["redactions_applied"] = redactions_applied

    binlog = _binlog_path(writer, id)
    _append_framed(binlog, _canonical_json(payload).encode("utf-8"))
    return turn_seq


def end(writer: Writer, id: str, reason: Optional[str] = None) -> Session:
    active = writer.store_path / "sessions" / ".active"
    if not active.exists() or active.read_text() != id:
        raise RuntimeError(f"session {id!r} is not active")

    s = _load_session_state(writer, id)
    s.ended_at = datetime.now(timezone.utc)
    active.unlink()

    # Seal the binlog (compute final chain tip; mark immutable per FS perms if desired)
    binlog = _binlog_path(writer, id)
    final_hash = _sha256_file(binlog)

    writer.emit_aux(
        kind="session.end",
        payload={
            "session_id":   id,
            "ended_at":     s.ended_at.isoformat(),
            "ended_reason": reason or "explicit",
            "turns_count":  _turns_count(writer, id),
            "binlog_hash":  final_hash,
        },
        actor=s.actor,
    )
    return s


def purge_expired(writer: Writer, dry_run: bool = False) -> dict:
    retention = writer.config().sessions.retention_days
    now = datetime.now(timezone.utc)
    purged = []
    for binlog in (writer.store_path / "sessions").rglob("*.binlog.zst"):
        date_str = binlog.parent.name
        try:
            date = datetime.strptime(date_str, "%Y-%m-%d").replace(tzinfo=timezone.utc)
        except ValueError:
            continue
        age_days = (now - date).days
        if age_days <= retention:
            continue
        session_id = binlog.stem.replace(".binlog", "")
        purged.append({"session_id": session_id, "date": date_str, "age_days": age_days})
        if not dry_run:
            # tombstone marker replaces the body
            binlog.write_text(_tombstone_manifest(session_id, date))
            writer.emit_aux(
                kind="session.purged",
                payload={
                    "session_id":  session_id,
                    "original_started_at": date.isoformat(),
                    "purged_at":   now.isoformat(),
                    "reason":      "retention_expired",
                },
                actor="retention-purger",
            )
    return {"purged_count": len(purged), "dry_run": dry_run, "purged": purged}


def _binlog_path(writer: Writer, session_id: str) -> Path:
    # find the binlog by scanning today's and yesterday's dirs (sessions can span midnight)
    for date_dir in sorted((writer.store_path / "sessions").iterdir(), reverse=True):
        if date_dir.is_dir() and date_dir.name not in (".active",):
            candidate = date_dir / f"{session_id}.binlog.zst"
            if candidate.exists():
                return candidate
    raise FileNotFoundError(f"session binlog for {session_id!r} not found")


def _next_turn_seq(writer: Writer, session_id: str) -> int:
    binlog = _binlog_path(writer, session_id)
    # Count existing frames; turn_seq starts at 0
    return _count_frames(binlog)


# Helpers (zstd framing, encryption, hashing) — implemented mirroring §6.2 patterns
```

### AGENTS.md §18 amendment text (proposed)

```text
## §18  Session transcript ledger (added by P22 — requires APPROVE chat-turn per §0.2)

§18.1  Sessions are an OPTIONAL turn-level audit trail for agent-user
conversations. Operators OPT IN per conversation via `cyberos session start`.

§18.2  Session bodies live at
`<memory-root>/sessions/<YYYY-MM-DD>/<id>.binlog.zst` and are independent
from the main audit chain. The framed binlog format mirrors §6.2.

§18.3  Sessions MUST carry a `classification` of either `confidential`
(default) or `restricted`. The classifications `public` and `internal`
are NOT permitted on sessions.

§18.4  When `classification: restricted`, every session.turn payload's
`content` field MUST be encrypted via the §5.4 envelope. The meta-frame
(role, ts, turn_seq) remains plaintext.

§18.5  Sessions emit summary rows on the main audit chain:
- `session.start` at lifecycle start
- `session.end`   at lifecycle end (or, on retention purge, NEVER — see §18.6)
- `session.purged` when a session's body is dropped per retention

§18.6  Retention is configured via `manifest.json:sessions.retention_days`
(default 30 days). Purge replaces the session body with a tombstone
manifest; the summary rows on the main chain remain. The fact of purge is
itself a chain leaf and is not erasable.

§18.7  Memory writes that occur during an active session MUST carry
`extra.session_id` on the put/move/delete row. The active session is
indicated by `<memory-root>/sessions/.active`; only one session may be
active at a time per memory.

§18.8  Lifecycle invariants enforced by the walker:
- Every `session.start` has 0 or 1 `session.end` for the same session_id
- Within a session's binlog, turn_seq is strictly monotonically increasing
- No `session.turn` precedes session.start or follows session.end
```

---

## §4 — Acceptance criteria

1. **Full lifecycle round-trip** — start → append × 3 → end → read returns the 3 turns in order. *(traces_to: §1 #1, §1 #7)*
2. **Default classification is confidential** — `cyberos session start --id x` → frontmatter `classification: confidential`. *(traces_to: §1 #3, DEC-251)*
3. **Restricted forces encryption** — `--classification restricted` → every turn's `content` is in `content_cipher`, not `content`. *(traces_to: §1 #4)*
4. **`public` and `internal` classifications rejected** — `--classification public` → CLI parse error or schema rejection. *(traces_to: §1 #3)*
5. **Append without start rejected** — `cyberos session append --id never-started ...` → error naming the violated state. *(traces_to: §1 #6)*
6. **Append after end rejected** — start → end → append → error. *(traces_to: §1 #6)*
7. **Double end rejected** — start → end → end → second end raises. *(traces_to: §1 #5)*
8. **Two active sessions rejected** — start one, attempt to start another without ending → raises. *(traces_to: §1 #1)*
9. **`session.start` row on main chain** — after `cyberos session start --id x`, HEAD has a new aux row of kind `session.start` with payload matching the schema. *(traces_to: §1 #1, §1 #5)*
10. **`session.end` row on main chain** — same for end. *(traces_to: §1 #1)*
11. **`session_id` attached to memory writes during active session** — start → put memories/x.md → end → the put row's `extra.session_id` equals the session id. *(traces_to: §1 #13)*
12. **`session_id` absent when no active session** — put outside session → no `session_id` in extra. *(traces_to: §1 #13)*
13. **Walker catches lifecycle violation** — handcraft an end without start in the binlog → `cyberos doctor` fails with `session-lifecycle-well-formed`. *(traces_to: §1 #5)*
14. **Walker catches turn_seq non-monotone** — handcraft turn_seq 0, 1, 1 in binlog → walker fails. *(traces_to: §1 #5)*
15. **Read without decrypt for restricted shows placeholder** — `cyberos session read --id x` (no --decrypt) on restricted session → turn metadata visible, content shown as `[encrypted content; --decrypt to read]`. *(traces_to: §1 #7)*
16. **Read with decrypt for restricted shows plaintext** — `--decrypt` flag on restricted session → content visible. *(traces_to: §1 #7)*
17. **`session list --since 24h`** — lists sessions started in last 24h with their classification + state (active|ended|purged). *(traces_to: §1 #8)*
18. **Retention purge expired** — fixture with sessions dated 35 days ago + retention=30 → `cyberos session purge-expired` produces `session.purged` rows; binlog bodies replaced by tombstone manifests. *(traces_to: §1 #9, §1 #10)*
19. **Retention purge `--dry-run`** — dry-run prints what would be purged; no binlog modified. *(traces_to: §1 #9)*
20. **`session.purged` payload shape** — payload matches §1 #10 schema (session_id, original_started_at, purged_at, reason). *(traces_to: §1 #10)*
21. **ACL on sessions/ subtree** — `sessions/STORE.yaml` denies `scheduled-importer` → `cyberos session start --actor scheduled-importer` rejected. *(traces_to: §1 #11)*
22. **§18 anchor required** — AGENTS.md lacks §18 → `cyberos session start` raises with structured message naming APPROVE phrase. *(traces_to: §1 #12)*
23. **PII redaction applied on opt-in** — `cyberos session append --redactions-applied true` → content runs through TASK-MEMORY-111 PII detector; `redactions_applied: true` set in payload. *(traces_to: §1 #14)*
24. **Storage path date-partitioned** — session started 2026-05-19 → binlog at `sessions/2026-05-19/<id>.binlog.zst`. *(traces_to: §1 #2)*
25. **Spanning midnight** — session started 23:55 UTC continues to receive turns at 00:05 UTC the next day; binlog remains in the start-date directory. *(traces_to: §1 #2)*

---

## §5 — Verification

```python
# modules/memory/tests/core/test_transcript.py
import pytest, json
from datetime import datetime, timedelta, timezone
from pathlib import Path

from cyberos.core.session import start, append, end, purge_expired


def test_full_lifecycle(empty_memory, ensure_section_18):
    """AC #1"""
    s = start(empty_memory, "sess-1")
    append(empty_memory, "sess-1", role="user",      content="hello")
    append(empty_memory, "sess-1", role="assistant", content="hi")
    append(empty_memory, "sess-1", role="user",      content="how are you")
    end(empty_memory, "sess-1")
    binlog = empty_memory.store_path / "sessions" / s.started_at.strftime("%Y-%m-%d") / "sess-1.binlog.zst"
    assert binlog.exists()
    # Read back via cyberos.core.session.read (not shown here for brevity)


def test_default_classification(empty_memory, ensure_section_18):
    """AC #2"""
    s = start(empty_memory, "sess-x")
    assert s.classification == "confidential"


def test_restricted_classification_encrypts(empty_memory, ensure_section_18, monkeypatch):
    """AC #3"""
    s = start(empty_memory, "sess-y", classification="restricted")
    append(empty_memory, "sess-y", role="user", content="secret hello")
    # Inspect raw binlog frame and confirm `content_cipher` present, `content` absent
    binlog = empty_memory.store_path / "sessions" / s.started_at.strftime("%Y-%m-%d") / "sess-y.binlog.zst"
    frame = _read_first_frame(binlog)
    payload = json.loads(frame)
    assert "content_cipher" in payload
    assert "content" not in payload


def test_invalid_classification_rejected(empty_memory, ensure_section_18):
    """AC #4"""
    with pytest.raises((ValueError, RuntimeError)):
        start(empty_memory, "sess-z", classification="public")


def test_append_without_start_rejected(empty_memory, ensure_section_18):
    """AC #5"""
    with pytest.raises(RuntimeError, match="no active session"):
        append(empty_memory, "ghost", role="user", content="hello")


def test_append_after_end_rejected(empty_memory, ensure_section_18):
    """AC #6"""
    start(empty_memory, "sess-a")
    end(empty_memory, "sess-a")
    with pytest.raises(RuntimeError):
        append(empty_memory, "sess-a", role="user", content="too late")


def test_double_end_rejected(empty_memory, ensure_section_18):
    """AC #7"""
    start(empty_memory, "sess-b")
    end(empty_memory, "sess-b")
    with pytest.raises(RuntimeError):
        end(empty_memory, "sess-b")


def test_two_active_sessions_rejected(empty_memory, ensure_section_18):
    """AC #8"""
    start(empty_memory, "sess-c")
    with pytest.raises(RuntimeError, match="already active"):
        start(empty_memory, "sess-d")


def test_session_id_attached_to_memory_writes(empty_memory, ensure_section_18):
    """AC #11"""
    s = start(empty_memory, "sess-e")
    head_before = empty_memory.head_seq()
    empty_memory.put("memories/facts/x.md", "body", meta={})
    row = empty_memory.read_audit_row(head_before + 1)
    assert row["extra"]["session_id"] == "sess-e"


def test_session_id_absent_outside_session(empty_memory, ensure_section_18):
    """AC #12"""
    head_before = empty_memory.head_seq()
    empty_memory.put("memories/facts/y.md", "body", meta={})
    row = empty_memory.read_audit_row(head_before + 1)
    assert "session_id" not in row.get("extra", {})


def test_protocol_amendment_required(memory_without_section_18):
    """AC #22"""
    with pytest.raises(Exception, match=r"APPROVE protocol change P22 §18"):
        start(memory_without_section_18, "sess-x")


def test_storage_date_partitioned(empty_memory, ensure_section_18):
    """AC #24"""
    s = start(empty_memory, "sess-dated")
    expected = empty_memory.store_path / "sessions" / s.started_at.strftime("%Y-%m-%d") / "sess-dated.binlog.zst"
    assert expected.exists()


def test_walker_catches_lifecycle_violation(memory_with_orphan_end, ensure_section_18):
    """AC #13"""
    import subprocess
    result = subprocess.run(["python", "-m", "cyberos", "--store",
                             str(memory_with_orphan_end.store_path), "doctor"],
                            capture_output=True, text=True)
    assert result.returncode != 0
    assert "session-lifecycle-well-formed" in result.stderr + result.stdout
```

```python
# modules/memory/tests/core/test_session.py
import pytest
from datetime import datetime, timedelta, timezone
from cyberos.core.session import purge_expired


def test_purge_expired_removes_old(memory_with_expired_sessions, ensure_section_18):
    """AC #18 + #20"""
    result = purge_expired(memory_with_expired_sessions)
    assert result["purged_count"] > 0
    # Inspect main chain for session.purged row
    rows = memory_with_expired_sessions.read_recent_audit_rows(20)
    assert any(r["kind"] == "session.purged" for r in rows)


def test_dry_run_does_not_purge(memory_with_expired_sessions, ensure_section_18):
    """AC #19"""
    result = purge_expired(memory_with_expired_sessions, dry_run=True)
    assert result["dry_run"] is True
    # No binlog modified
    for binlog in (memory_with_expired_sessions.store_path / "sessions").rglob("*.binlog.zst"):
        assert binlog.stat().st_size > 0  # original content preserved
```

---

## §6 — Implementation skeleton

API contracts above. Order:

1. AGENTS.md §18 amendment text (DO NOT commit until APPROVE chat-turn).
2. Schema (`memory.schema.json`).
3. Walker invariants.
4. `cyberos/core/session.py`.
5. `cyberos/cli/session.py`.
6. `cyberos/__main__.py` wiring.
7. Writer: `extra.session_id` injection + `_require_protocol_amendment_p22`.
8. Tests.
9. CHANGELOG.

---

## §7 — Dependencies

- **TASK-MEMORY-111 (related)** — PII detector invoked on `--redactions-applied true`.
- **TASK-MEMORY-115 (this task blocks)** — dream's patterns detector consumes `session.start` / `session.end` rows + (optionally) decrypts session bodies for finer-grained pattern detection.
- **TASK-MEMORY-117 (related)** — ACL on `sessions/` subtree controls who can start/append/end sessions.
- **TASK-MEMORY-109 (related)** — Claude Code hook can call `cyberos session append` to record real conversation turns; the hook can pre-redact via TASK-MEMORY-111 then set `redactions_applied: true`.

---

## §8 — Example payloads

### `session.start` aux row

```json
{
  "kind": "session.start",
  "payload": {
    "session_id":     "sre-investigation-2026-05-19",
    "started_at":     "2026-05-19T08:00:00Z",
    "classification": "confidential",
    "retention_days": 30,
    "actor":          "stephen"
  }
}
```

### `session.end` aux row

```json
{
  "kind": "session.end",
  "payload": {
    "session_id":   "sre-investigation-2026-05-19",
    "ended_at":     "2026-05-19T08:42:13Z",
    "ended_reason": "explicit",
    "turns_count":  47,
    "binlog_hash":  "f3a9b2c1d4e5f6789a0b1c2d3e4f56789a0b1c2d3e4f56789a0b1c2d3e4f5678"
  }
}
```

### Encrypted turn payload (restricted)

```json
{
  "session_id":     "private-1",
  "role":           "user",
  "turn_seq":       17,
  "ts":             "2026-05-19T08:23:11Z",
  "content_cipher": {
    "alg":   "aes-256-gcm",
    "key_ref": "default",
    "nonce": "abc123…",
    "ct":    "f3a9b2…"
  }
}
```

### `session.purged` aux row

```json
{
  "kind": "session.purged",
  "payload": {
    "session_id":          "old-conversation-2026-04-15",
    "original_started_at": "2026-04-15T00:00:00Z",
    "purged_at":           "2026-05-19T03:00:00Z",
    "reason":              "retention_expired"
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- `cyberos session export --format jsonl` — §1 #15; slice 4.
- `--correlates-with <other-id>` linkage — §1 #16; slice 4+.
- Real-time tail (`cyberos session tail --id x`) — slice 4+.
- Multi-tenant per-session retention overrides — slice 4+ once TASK-MEMORY-117 lands.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| §18 missing | constructor check | session.start raises | APPROVE chat-turn |
| Two `start` for same id | path-exists check | second raises | Operator picks unique id |
| Two active sessions | .active file check | second start raises | End current first |
| Append without start | .active file absent | raises | Start session first |
| Crash between start and end | .active file persists | next start fails | Operator deletes `.active` manually (audit row exists) |
| Disk full during append | write fails | binlog frame partial; CRC32C catches on next read | walker repairs by truncating to last intact frame |
| Encryption key missing | encryption call fails | start raises | Operator provisions key |
| Walker invariant violation | doctor | non-zero exit | Operator fixes binlog or rolls back |
| Restricted session read without --decrypt | placeholder shown | works | Pass --decrypt |
| Session retained past expiry by mistake | purge-expired catches on next cron | scheduled retroactive purge | None |
| Manual purge of active session | reject | raises | End session first |
| Cross-day session (midnight rollover) | binlog stays in original date dir | works | None |
| ACL denies session start | ACL check before path-exists | raises | Adjust ACL |
| Concurrent appends from two processes | only one .active session at a time | second process sees same active | Sequential by design |
| `.active` file corrupted (empty / nonexistent session_id) | read returns empty/missing | start raises ambiguous | Operator deletes .active manually |
| Old session's binlog file deleted manually | next purge can't find it | warned in stderr; chain-rows remain | Operator restores from backup if needed |
| Tombstone manifest hand-edited | walker validates schema | doctor fails | Restore tombstone |
| 1MB turn content | normal zstd compression | works | None |
| Restricted session purged → encrypted blob gone | already encrypted; purge removes ciphertext + IV | irretrievable per design | None |

---

## §11 — Implementation notes

- **`.active` pointer file** — single-process-active is enforced by file presence. Two-active is a deliberate choice; supporting concurrent sessions on one memory is slice-4+ and would require per-PID session pointers.
- **Turn sequencing within a session** — `_next_turn_seq()` counts existing frames. Atomic per `.lock` (same exclusive lock as memory writes); no race.
- **Date-partitioning at start-date** — sessions spanning midnight stay in their start date's directory. Retention purges by start date, which is the natural mental model.
- **Tombstone manifest** — replaces the binlog body with a JSON document declaring the session id, original metadata, purge timestamp. Walker can still find the session pointer.
- **Encryption envelope reuses §5.4** — same `Envelope` definition as memory files; key management is operator's responsibility (slice-4 KMS integration).
- **`session.purged` is the load-bearing chain leaf** — keeps the *fact* of purge auditable forever, even after the body is gone. Matches the §3.6 `delete(purge)` semantics for memory files.
- **`--actor` flag for sessions** — recorded in session frontmatter + every aux row; ACL check uses this actor against `sessions/STORE.yaml`.
- **Session list output** — JSON line per session for piping into tools. `--format pretty` for human-readable; default JSONL for scripting.
- **The `ensure_section_18` test fixture** — symlinks AGENTS.md to a fixture that includes the §18 amendment text. Lets tests run end-to-end as if the APPROVE chat-turn had already happened.

---

*End of TASK-MEMORY-119.*
