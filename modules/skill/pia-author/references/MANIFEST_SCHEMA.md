# Manifest schema — `manifest@1`

Version: 1.0.0  Status: Normative for every author skill that emits multiple artefacts in a batch.

This file describes the on-disk shape of `manifest.json`, the state machine's persistent store. The manifest is the re-entrancy anchor; the skill's phase (PLAN / WORKER / RESUME) is computed from its contents.

---

## §1  File location

`<output_dir>/manifest.json` by default. Overridable via input envelope `manifest_path`.

## §2  Atomic write rules

Per AGENTS.md §4.1 (memory module): two-phase write.

1. Write to `<manifest_path>.tmp.<nonce>`.
2. fsync (use `fcntl(F_BARRIERFSYNC)` on macOS).
3. Rename to `<manifest_path>`.
4. fsync parent directory.

The manifest is rewritten after every state transition (not batched). Crash recovery relies on this.

## §3  Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "cyberos.skill.manifest/v1",
  "type": "object",
  "required": [
    "manifest_version", "skill_id", "skill_version", "trace_id",
    "source_hash", "source_files", "plan", "artefacts", "hitl_pending", "last_audit_at"
  ],
  "properties": {
    "manifest_version": { "type": "string", "const": "manifest@1" },
    "skill_id": { "type": "string" },
    "skill_version": { "type": "string", "pattern": "^\\d+\\.\\d+\\.\\d+(-[A-Za-z0-9.-]+)?$" },
    "trace_id": { "type": "string", "format": "uuid" },
    "source_hash": {
      "type": "string",
      "pattern": "^[0-9a-f]{64}$",
      "description": "SHA-256 over UTF-8 NFC-normalised concat of source_files, in declared order. See §3.1."
    },
    "source_files": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["path", "media_type", "hash", "size_bytes", "read_at"],
        "properties": {
          "path": { "type": "string" },
          "media_type": { "type": "string" },
          "hash": { "type": "string", "pattern": "^[0-9a-f]{64}$" },
          "size_bytes": { "type": "integer", "minimum": 0 },
          "read_at": { "type": "string", "format": "date-time" }
        }
      }
    },
    "plan": {
      "type": "object",
      "required": ["status", "approval_hash", "created_at"],
      "properties": {
        "status": {
          "type": "string",
          "enum": ["DRAFT", "AWAITING_APPROVAL", "APPROVED", "AMENDED_AWAITING_APPROVAL", "INVALIDATED"]
        },
        "approval_hash": { "type": "string", "pattern": "^[0-9a-f]{64}$" },
        "created_at": { "type": "string", "format": "date-time" },
        "approved_at": { "type": "string", "format": "date-time" },
        "approved_by": { "type": "string" }
      }
    },
    "artefacts": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["id", "slug", "file_path", "status", "iterations"],
        "properties": {
          "id": { "type": "string" },
          "slug": { "type": "string", "pattern": "^[a-z0-9-]+$" },
          "file_path": { "type": "string" },
          "artefact_hash": { "type": "string", "pattern": "^[0-9a-f]{64}$" },
          "audit_hash": { "type": "string", "pattern": "^[0-9a-f]{64}$" },
          "source_refs": { "type": "array", "items": { "type": "string" } },
          "depends_on": { "type": "array", "items": { "type": "string" } },
          "status": {
            "type": "string",
            "enum": ["PENDING", "DRAFTING", "PASS", "HITL_PAUSE", "EXHAUSTED", "FAIL", "STALE", "WONTFIX"]
          },
          "iterations": { "type": "integer", "minimum": 0 },
          "blocking_issues": {
            "type": "array",
            "items": {
              "type": "object",
              "required": ["rule_id", "question", "category"],
              "properties": {
                "rule_id": { "type": "string" },
                "question": { "type": "string" },
                "category": { "type": "string" },
                "resolution": { "type": ["string", "null"], "default": null },
                "resolved_at": { "type": "string", "format": "date-time" }
              }
            }
          }
        }
      }
    },
    "hitl_pending": {
      "type": "object",
      "required": ["any_blocking"],
      "properties": {
        "any_blocking": { "type": "boolean" },
        "issue_count": { "type": "integer", "minimum": 0 }
      }
    },
    "amendments": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["id", "risk_class", "status", "change_description"],
        "properties": {
          "id": { "type": "string", "pattern": "^AMD-\\d{3,}$" },
          "risk_class": { "type": "string", "enum": ["low", "medium", "high"] },
          "status": { "type": "string", "enum": ["proposed", "approved", "applied", "rejected"] },
          "change_description": { "type": "string" }
        }
      }
    },
    "last_audit_at": { "type": "string", "format": "date-time" }
  }
}
```

### §3.1  source_hash computation

`source_hash = sha256(NFC-normalise(b'\n'.join(sorted(file.path) for file in source_files) + b'\x00\x00' + b'\n'.join(file.content_bytes for file in source_files in declared order)))`

Sort + dual-separator are deterministic. Recompute on every invocation; if the new value differs from the manifest's stored `source_hash`, the manifest's affected artefacts transition to STALE and the skill emits `INPUTS_CHANGED` per `references/FAILURE_MODES.md`.

### §3.2  STALE handling

When `source_hash` drifts:

1. Mark every `PASS` and `HITL_PAUSE` artefact with `status: STALE`.
2. Emit `INPUTS_CHANGED` block to the operator with a per-artefact diff.
3. Operator chooses per artefact: `REVERT_TO_MANIFEST` (re-author from pre-drift sources), `OVERWRITE_WITH_NEW` (re-author from current sources), `WONTFIX` (leave as-is with a `wontfix` note).
4. Skill applies the choices and re-enters PLAN phase if any artefact is `OVERWRITE_WITH_NEW`.

### §3.3  Per-artefact fields the schema generator must populate

When PLAN phase enumerates the backlog, each artefact entry MUST be populated with:

- `id`: deterministic from PLAN-time index (e.g. `PIA-001` for the first item).
- `slug`: kebab-cased short description, ≤32 chars.
- `file_path`: `<output_dir>/<id>-<slug>.md`.
- `source_refs`: list of file paths + line ranges (`./EXAMPLE.md:42-58`) that justify this artefact's existence.
- `depends_on`: list of other artefact IDs in this batch that must complete first.
- `status`: starts at `PENDING`.
- `iterations`: starts at 0.
- `blocking_issues`: starts empty `[]`.

### §3.4  Write discipline

| Trigger | Write |
|---|---|
| PLAN phase begins | `plan.status = DRAFT` + empty artefacts. |
| PLAN approval requested | `plan.status = AWAITING_APPROVAL`. |
| PLAN approved | `plan.status = APPROVED`, `plan.approved_at`, `plan.approved_by`. |
| WORKER claims an artefact | `artefacts[X].status = DRAFTING`. |
| WORKER writes an artefact | `artefacts[X].artefact_hash`, increment `iterations`. |
| Audit returns verdict | `artefacts[X].status = PASS / HITL_PAUSE / EXHAUSTED`, `audit_hash`. |
| Amendment proposed | `amendments[].status = proposed`. |
| Amendment applied | `amendments[].status = applied`, plan re-rendered. |
| HITL resolution applied | `artefacts[X].blocking_issues[N].resolution`, `resolved_at`. |
| Any of the above | `last_audit_at = now()`. |

The manifest is the source of truth. The skill MUST NOT cache state across calls; it re-reads the manifest on every invocation.
