---
id: TASK-PLUGIN-001
title: "Plugin manifest schema v1.0.0 — canonical plugin.json validated against manifest.schema.json with cyberos-plugin pack reference packer"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-19T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: PLUGIN
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-19
shipped: null
memory_chain_hash: null
related_tasks: [TASK-PLUGIN-002, TASK-PLUGIN-003, TASK-PLUGIN-004, TASK-PLUGIN-005, TASK-PLUGIN-006, TASK-PLUGIN-007, TASK-PLUGIN-008, TASK-MCP-003, TASK-SKILL-111]
depends_on: []
blocks: [TASK-PLUGIN-002, TASK-PLUGIN-003, TASK-PLUGIN-004, TASK-PLUGIN-005, TASK-PLUGIN-006, TASK-PLUGIN-007, TASK-PLUGIN-008]

source_pages:
  - modules/plugin/README.md
  - modules/plugin/manifest.schema.json
  - modules/plugin/INTEROP.md

source_decisions:
  - DEC-2400 2026-05-19 — Manifest schema v1.0.0 is the SINGLE source of truth for plugin bundle shape; every adapter consumes it; bumps require this task amendment
  - DEC-2401 2026-05-19 — schema_version is a const ("1.0.0") not a range; future versions get task-PLUGIN-001a/b/c successor tasks to avoid drift
  - DEC-2402 2026-05-19 — id pattern is kebab-case, 3-64 chars, lowercase only; matches OCI image-name conventions for marketplace symmetry
  - DEC-2403 2026-05-19 — version is SemVer 2.0 strict; pre-release identifiers allowed (1.0.0-beta.1) but build metadata MUST NOT affect compatibility
  - DEC-2404 2026-05-19 — capabilities is a closed enum of 7 keys (read_memory / write_memory / execute_workflow / list_skills / invoke_skill / publish_skill / route_natural_language); additions require schema bump
  - DEC-2405 2026-05-19 — tool names MUST match `^cyberos\\.[a-z][a-z0-9]*\\.[a-z][a-z0-9_]*$` per TASK-MCP-003 SEP-986
  - DEC-2406 2026-05-19 — auth.method is a const ("oauth-pkce") in v1; long-lived secrets (api-key, basic-auth) are explicitly rejected
  - DEC-2407 2026-05-19 — signature.rekor_uuid is REQUIRED at the schema level (not optional) — unsigned bundles cannot validate
  - DEC-2408 2026-05-19 — Reference packer is Python (`modules/plugin/cyberos_plugin/`), Rust binary deferred to TASK-PLUGIN-007; Python first matches CUO/memory reference-impl convention
  - DEC-2409 2026-05-19 — Packer MUST be reproducible — same input → same SHA-256; timestamps and machine names MUST NOT leak into the bundle

build_envelope:
  language: python 3.10
  service: modules/plugin/cyberos_plugin/
  new_files:
    - modules/plugin/cyberos_plugin/__init__.py
    - modules/plugin/cyberos_plugin/packer.py
    - modules/plugin/cyberos_plugin/validator.py
    - modules/plugin/cyberos_plugin/cli.py
    - modules/plugin/cyberos_plugin/reproducible.py
    - modules/plugin/pyproject.toml
    - modules/plugin/tests/test_schema_required_fields.py
    - modules/plugin/tests/test_schema_tool_name_pattern.py
    - modules/plugin/tests/test_schema_semver_pattern.py
    - modules/plugin/tests/test_packer_reproducible.py
    - modules/plugin/tests/test_packer_signature_required.py
    - modules/plugin/tests/test_cli_pack_smoke.py
    - modules/plugin/tests/fixtures/valid_minimal_plugin.json
    - modules/plugin/tests/fixtures/valid_complete_plugin.json
    - modules/plugin/tests/fixtures/invalid_missing_signature.json
    - modules/plugin/tests/fixtures/invalid_tool_name.json
    - modules/plugin/manifests/cyberos@1.0.0.plugin.json

  modified_files:
    - modules/plugin/manifest.schema.json
    - website docs (Plugin page)

  allowed_tools:
    - file_read: modules/plugin/**
    - file_write: modules/plugin/{cyberos_plugin,tests,manifests}/**
    - bash: cd modules/plugin && python -m pytest tests/

  disallowed_tools:
    - skip signature field (per DEC-2407)
    - inject timestamps into bundle (per DEC-2409)
    - hard-code rust language (per DEC-2408 — Python first)

effort_hours: 8
subtasks:
  - "1.0h: refine manifest.schema.json — validate 8 fixture cases pass/fail correctly"
  - "0.5h: cyberos_plugin/__init__.py + pyproject.toml"
  - "1.5h: packer.py — read manifest, validate, build reproducible zip"
  - "0.8h: validator.py — JSONSchema validation + custom checks (tool name SEP-986, capability/scope coherence)"
  - "0.6h: reproducible.py — strip mtimes, sort entries, fixed permissions"
  - "0.5h: cli.py — argparse for `pack`, `validate`, `doctor`"
  - "2.0h: tests — 6 test files exercising schema + packer + CLI"
  - "1.1h: cyberos@1.0.0.plugin.json fixture + README update"

risk_if_skipped: "Without manifest schema as the single source of truth, every adapter (Claude Code, Cursor, Cowork, Codex CLI) drifts into its own bundle format. Plugin bundles produced by different toolchains lose cross-runtime portability. The marketplace cannot validate uploads. The audit chain cannot rely on stable plugin_id/version pairs. Without DEC-2407 mandatory signature, unsigned bundles can be published unchecked. Without DEC-2409 reproducibility, supply-chain attestation becomes impossible — two builds from the same source produce different hashes, invalidating Sigstore Rekor anchoring."
---

## §1 — Description (BCP-14 normative)

The PLUGIN module **MUST** ship the manifest schema and reference packer at `modules/plugin/cyberos_plugin/`. The schema is JSONSchema 2020-12 at `modules/plugin/manifest.schema.json`; the packer is Python at `modules/plugin/cyberos_plugin/packer.py` with CLI entrypoint `cyberos-plugin pack`. Together they define how a CyberOS plugin bundle is described, validated, and produced.

1. **MUST** validate canonical manifests against `manifest.schema.json` per DEC-2400. Validation runs at three sites: (a) author time via `cyberos-plugin validate manifests/<id>@<version>.plugin.json`; (b) pack time via `cyberos-plugin pack` (refuses to emit a bundle if the manifest doesn't validate); (c) install time via the host runtime (each adapter inherits the schema). Validation errors MUST be human-readable with the failing JSON-pointer and the constraint that failed.

2. **MUST** declare `schema_version` as the const `"1.0.0"` per DEC-2401. Schema evolutions are handled by writing task-PLUGIN-001a (then b, then c) — never by widening the v1 schema in place. This keeps every shipped plugin pinned to a known schema generation.

3. **MUST** enforce the `id` pattern `^[a-z0-9][a-z0-9-]{1,62}[a-z0-9]$` per DEC-2402 — lowercase kebab-case, 3-64 characters, no leading/trailing hyphen. This pattern is OCI-image-name compatible so marketplace artefacts can be addressed as `oci://plugins.cyberskill.world/<id>:<version>` without transformation.

4. **MUST** enforce SemVer 2.0 on the `version` field per DEC-2403. The validator accepts `MAJOR.MINOR.PATCH`, `MAJOR.MINOR.PATCH-PRERELEASE` (e.g. `1.0.0-beta.1`), and `MAJOR.MINOR.PATCH+BUILD` (e.g. `1.0.0+sha.abc123`). Build metadata MUST NOT affect compatibility — two manifests with `version: "1.0.0+a"` and `version: "1.0.0+b"` are semantically equal.

5. **MUST** restrict `capabilities` to the closed enum of 7 keys per DEC-2404: `read_memory`, `write_memory`, `execute_workflow`, `list_skills`, `invoke_skill`, `publish_skill`, `route_natural_language`. `additionalProperties` is false. Adding a capability requires bumping the schema version per DEC-2401.

6. **MUST** enforce the SEP-986 tool name pattern `^cyberos\.[a-z][a-z0-9]*\.[a-z][a-z0-9_]*$` per DEC-2405. Each tool's `name` MUST start with `cyberos.`, then a module slug, then a verb-underscore-noun pair (e.g. `cyberos.cuo.execute_workflow`, `cyberos.memory.read_audit`, `cyberos.skill.list_catalog`).

7. **MUST** enforce `auth.method == "oauth-pkce"` as a JSONSchema const per DEC-2406. Other auth methods (api-key, basic-auth, bearer-static) MUST be rejected at validation. Future versions MAY add methods via task-PLUGIN-001a but v1 is locked.

8. **MUST** require `signature.rekor_uuid` and `signature.sigstore_bundle` per DEC-2407 — these fields are at the top of `manifest.schema.json#/required`. A manifest cannot validate without a Sigstore Rekor transparency-log entry referenced. The packer (clause 11) refuses to emit unsigned bundles.

9. **MUST** validate capability/scope coherence: a manifest declaring `capabilities.write_memory: true` MUST also list at least one tool whose `scopes` array contains `cyberos:memory:write`. This is a JSONSchema-extension check in `validator.py`, not pure schema. Reason: hosts surface declared capabilities at install time; if the declaration doesn't match the actual scopes requested, users grant under false pretences.

10. **MUST** produce reproducible bundles per DEC-2409. `packer.py` MUST: (a) set every zip entry's `mtime` to a fixed epoch (1980-01-01 00:00:00 UTC — the zip-format minimum); (b) sort entries by path; (c) use fixed permissions (0o644 for files, 0o755 for directories); (d) include no machine names, user IDs, or local paths in any file. Two builds of the same manifest from the same source MUST produce identical SHA-256.

11. **MUST** expose CLI `cyberos-plugin pack <manifest> [--out <path>]` and `cyberos-plugin validate <manifest>` and `cyberos-plugin doctor <bundle>`. `pack` produces a zip containing `plugin.json` + `commands/` + `skills/` + adapter assets. `validate` is dry-run schema check. `doctor` opens an existing bundle and checks the 8 INTEROP invariants.

12. **MUST NOT** allow `additionalProperties` at the top level of the manifest per DEC-2400. Unknown fields are rejected to keep the contract closed. Adapter-specific fields go under `targets[].extensions{}` (introduced in TASK-PLUGIN-007).

13. **MUST NOT** silently coerce types (e.g. `version: 1.0` → `"1.0.0"`). Type mismatches fail validation with a clear error.

14. **MUST NOT** accept manifests where any tool's `annotations.destructive == true` but `scopes` does not include a corresponding write scope. Per TASK-MCP-006 tool gating, destructive tools require explicit scope coverage.

---

## §2 — Why this design

**Why JSONSchema 2020-12 (DEC-2400)?** Latest stable JSONSchema draft; broad tooling support (Python `jsonschema` 4.x, Rust `jsonschema`, JS `ajv` 8.x); supports `const`, `additionalProperties: false`, pattern, format, enum — every constraint we need. Older drafts (draft-07) lack some constraints we use; draft-2019-09 is superseded.

**Why const schema_version (DEC-2401)?** Allowing a range (e.g. `^1\\..*$`) invites silent compatibility drift — fields appear in v1.3 manifests that v1.0 validators don't recognise. Constant versions force every new field to ship under a new task with explicit ecosystem migration guidance. This is the Lockfile pattern, applied to plugin schemas.

**Why OCI-compatible id pattern (DEC-2402)?** Strategy §4 Level 3 envisions a marketplace addressable as `oci://plugins.cyberskill.world/<id>:<version>`. If `id` accepts characters OCI rejects (uppercase, underscores, dots), the marketplace adapter has to transform before push, breaking round-trip identity. Keeping the patterns aligned avoids the transformation entirely.

**Why SemVer 2.0 (DEC-2403)?** Plugin consumers (hosts) need a well-known compatibility model. SemVer is the industry default. Pre-release identifiers (`1.0.0-beta.1`) are essential for staged rollouts. Build metadata is semantically irrelevant per SemVer spec — two builds of the same source can carry different `+sha.x` and still be the same plugin.

**Why closed capabilities enum (DEC-2404)?** Hosts render the capability list at install time so the user knows what they're granting. If capabilities are an open string set, malicious plugins could declare misleading capability names (`harmless_memory_lookup` instead of `read_memory`) that users skim past. A closed enum forces every capability to be reviewed and named in this task.

**Why SEP-986 enforced at manifest level (DEC-2405, clause 6)?** TASK-MCP-003 already enforces this at the gateway. Enforcing it at the manifest schema level catches violations earlier (author time, not runtime) and surfaces them in the same validator output as other schema errors. Defense-in-depth.

**Why OAuth-PKCE only in v1 (DEC-2406, clause 7)?** Long-lived secrets in bundles are a supply-chain liability — anyone who exfiltrates a bundle gets the key. OAuth-PKCE forces a per-install handshake against `auth.cyberskill.world`, with token rotation. The threat model is "stolen bundle MUST NOT yield credentials." API keys violate that; OAuth-PKCE preserves it.

**Why required Rekor UUID (DEC-2407, clause 8)?** Strategy §2 lists "open audit chain" as one of CyberOS's four defensible positions. A plugin without a Rekor anchor has no audit chain — anyone could swap the bytes after publication. Making the field required at the schema level means the absence is caught immediately, not at publish time.

**Why Python reference packer (DEC-2408, clause 11)?** Matches the CUO + memory convention (Python reference impl + Rust production binary). Python is faster to iterate, easier to fuzz, and `jsonschema` is mature. The Rust production binary (`services/plugin-host/`) lands in TASK-PLUGIN-007 alongside the multi-runtime adapters.

**Why reproducibility (DEC-2409, clause 10)?** Sigstore Rekor proves "this artefact was signed by X at time Y." That proof is only useful if a verifier can rebuild the bundle locally and confirm the hash matches. Non-reproducible bundles break that round-trip — you can verify the signature, but you can't verify the bytes came from the source. Reproducibility makes Sigstore actually work.

**Why JSON-pointer error messages (clause 1)?** Plugin authors will hit validation errors. `path /tools/3/name: pattern '^cyberos\\..*$' violated` is actionable. `validation failed` is not.

**Why capability/scope coherence check (clause 9)?** A common bug pattern: developer adds a capability declaration but forgets to add the corresponding scope in `tools[*].scopes`. Host surfaces the wrong consent UI. The cross-field check catches this at validate time.

---

## §3 — API contract

### `manifest.schema.json` (already at `modules/plugin/manifest.schema.json`)

The full schema is in the file. Required top-level keys: `schema_version`, `id`, `version`, `name`, `description`, `authors`, `license`, `capabilities`, `tools`, `auth`, `audit`, `targets`, `signature`. See the file for the full property definitions.

### Python packer surface (`cyberos_plugin/packer.py`)

```python
from dataclasses import dataclass
from pathlib import Path
from typing import Sequence

@dataclass(frozen=True)
class PackResult:
    bundle_path: Path
    sha256: str            # reproducible hash
    size_bytes: int
    tools_count: int
    commands_count: int
    skills_count: int

def pack(
    manifest_path: Path,
    target: str = "claude-code",      # one of {claude-code,cursor,cowork,codex-cli}
    out_dir: Path = Path("dist"),
    *,
    fail_on_unsigned: bool = True,    # DEC-2407
    reproducible: bool = True,        # DEC-2409
) -> PackResult: ...

def validate(manifest_path: Path) -> Sequence[ValidationError]: ...

def doctor(bundle_path: Path) -> Sequence[InvariantViolation]: ...
```

### CLI surface (`cyberos_plugin/cli.py`)

```text
$ cyberos-plugin pack <manifest> [--target {claude-code,cursor,cowork,codex-cli}] [--out <dir>]
$ cyberos-plugin validate <manifest>
$ cyberos-plugin doctor <bundle.plugin>
$ cyberos-plugin --version
```

### Minimal valid manifest

```json
{
  "schema_version": "1.0.0",
  "id": "cyberos",
  "version": "1.0.0",
  "name": "CyberOS",
  "description": "Persona-aware orchestration + memory + skills for any agentic IDE.",
  "authors": [{"name": "CyberSkill Software", "url": "https://cyberskill.world"}],
  "license": "Apache-2.0",
  "capabilities": {"read_memory": true, "execute_workflow": true, "list_skills": true},
  "tools": [
    {
      "name": "cyberos.cuo.list_personas",
      "description": "List the 47 active CyberOS personas available for orchestration.",
      "input_schema": {"type": "object", "properties": {}},
      "scopes": ["cyberos:cuo:list"]
    }
  ],
  "auth": {
    "method": "oauth-pkce",
    "authorize_url": "https://auth.cyberskill.world/v1/oauth/authorize",
    "token_url": "https://auth.cyberskill.world/v1/oauth/token",
    "scopes": ["cyberos:cuo:list"]
  },
  "audit": {
    "endpoint": "https://memory.cyberskill.world/v1/audit",
    "kinds": ["plugin.installed", "plugin.invoked", "plugin.uninstalled", "plugin.updated"]
  },
  "targets": ["claude-code"],
  "signature": {
    "sigstore_bundle": "<base64-encoded-sigstore-bundle>",
    "rekor_uuid": "24296fb24b8ad77a..."
  }
}
```

---

## §4 — Acceptance criteria

1. **Schema rejects missing schema_version** — fixture `invalid_missing_schema_version.json` fails validation with `path /: required property 'schema_version' missing`.
2. **Schema rejects schema_version != "1.0.0"** — fixture with `schema_version: "1.1.0"` fails with const violation.
3. **Schema rejects uppercase id** — fixture with `id: "CyberOS"` fails the pattern.
4. **Schema rejects id < 3 chars** — fixture with `id: "ab"` fails.
5. **Schema rejects malformed version** — fixture with `version: "1.0"` fails the SemVer pattern.
6. **Schema accepts SemVer pre-release** — `version: "1.0.0-beta.1"` passes.
7. **Schema accepts SemVer build metadata** — `version: "1.0.0+sha.abc123"` passes.
8. **Schema rejects unknown capability** — fixture with `capabilities.hack_the_planet: true` fails additionalProperties.
9. **Schema rejects non-SEP-986 tool name** — `tools[0].name: "foo.bar"` fails pattern.
10. **Schema rejects auth.method != "oauth-pkce"** — `auth.method: "api-key"` fails const.
11. **Schema rejects missing signature.rekor_uuid** — fixture `invalid_missing_signature.json` fails.
12. **Validator catches capability/scope mismatch** — manifest declaring `write_memory: true` but no tool with `cyberos:memory:write` scope fails with a custom error message.
13. **Validator catches destructive-without-write-scope** — tool with `annotations.destructive: true` and no write scope fails.
14. **Packer refuses unsigned bundle when fail_on_unsigned=True** — call packer with manifest missing signature → raises `UnsignedManifestError`.
15. **Packer produces reproducible SHA-256** — two `pack()` calls on the same manifest from the same source produce identical SHA-256.
16. **Packer entries have epoch mtime** — `unzip -l bundle.plugin` shows all entries timestamped 1980-01-01.
17. **Packer entries are sorted by path** — `unzip -l` shows ascending path order.
18. **Packer file permissions are 0o644** — `unzip -l` shows `-rw-r--r--` on every file.
19. **Packer rejects unknown target** — `pack(..., target="bogus")` raises `UnknownTargetError`.
20. **CLI pack returns exit 0 on success** — `cyberos-plugin pack manifests/cyberos@1.0.0.plugin.json` writes a file and exits 0.
21. **CLI validate returns exit 1 on failure** — `cyberos-plugin validate invalid_missing_signature.json` prints error to stderr, exits 1.
22. **CLI doctor catches missing Rekor on existing bundle** — running doctor on a bundle whose plugin.json has signature stripped fails.
23. **Validation errors include JSON-pointer paths** — error messages contain `/tools/0/name` style references, not "line 47."

---

## §5 — Verification

```python
# tests/test_schema_required_fields.py
import json, jsonschema, pathlib

SCHEMA = json.loads(pathlib.Path("modules/plugin/manifest.schema.json").read_text())

def test_minimal_valid_passes():
    m = json.loads(pathlib.Path("tests/fixtures/valid_minimal_plugin.json").read_text())
    jsonschema.validate(m, SCHEMA)  # no exception

def test_missing_schema_version_fails():
    m = json.loads(pathlib.Path("tests/fixtures/valid_minimal_plugin.json").read_text())
    del m["schema_version"]
    with pytest.raises(jsonschema.ValidationError) as e:
        jsonschema.validate(m, SCHEMA)
    assert "schema_version" in str(e.value)

def test_schema_version_const():
    m = _load_minimal()
    m["schema_version"] = "1.1.0"
    with pytest.raises(jsonschema.ValidationError):
        jsonschema.validate(m, SCHEMA)
```

```python
# tests/test_schema_tool_name_pattern.py
@pytest.mark.parametrize("name,valid", [
    ("cyberos.cuo.execute_workflow", True),
    ("cyberos.memory.read_audit", True),
    ("cyberos.skill.list_catalog", True),
    ("CyberOS.cuo.execute", False),         # uppercase
    ("cyberos.execute", False),             # missing module segment
    ("foo.bar.baz", False),                 # wrong prefix
    ("cyberos..execute_workflow", False),   # empty module
    ("cyberos.cuo.123_execute", False),     # verb starts with digit
])
def test_tool_name_pattern(name, valid):
    m = _load_minimal()
    m["tools"][0]["name"] = name
    if valid:
        jsonschema.validate(m, SCHEMA)
    else:
        with pytest.raises(jsonschema.ValidationError):
            jsonschema.validate(m, SCHEMA)
```

```python
# tests/test_packer_reproducible.py
def test_pack_is_reproducible(tmp_path):
    m = tmp_path / "manifest.json"; m.write_text(MINIMAL_MANIFEST)
    r1 = pack(m, out_dir=tmp_path / "a")
    r2 = pack(m, out_dir=tmp_path / "b")
    assert r1.sha256 == r2.sha256

def test_pack_epoch_mtime(tmp_path):
    r = pack(_minimal(tmp_path), out_dir=tmp_path / "out")
    import zipfile
    with zipfile.ZipFile(r.bundle_path) as zf:
        for info in zf.infolist():
            assert info.date_time == (1980, 1, 1, 0, 0, 0)
```

```python
# tests/test_packer_signature_required.py
def test_pack_refuses_unsigned(tmp_path):
    m = tmp_path / "m.json"
    raw = json.loads(MINIMAL_MANIFEST); del raw["signature"]; m.write_text(json.dumps(raw))
    with pytest.raises(UnsignedManifestError):
        pack(m, fail_on_unsigned=True)
```

```bash
# tests/test_cli_pack_smoke.sh
cyberos-plugin pack modules/plugin/manifests/cyberos@1.0.0.plugin.json \
    --target claude-code --out /tmp/pack-test
test -f /tmp/pack-test/cyberos-1.0.0.plugin
cyberos-plugin doctor /tmp/pack-test/cyberos-1.0.0.plugin
echo $?  # → 0
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton — `packer.py`, `validator.py`, `reproducible.py`, `cli.py` are thin orchestrators around `jsonschema.validate` + `zipfile.ZipFile`. The custom validator additions live in `validator.py::check_capability_scope_coherence`.)

---

## §7 — Dependencies

- **Upstream:** none — this is the keystone task with no task-level dependencies.
- **Downstream:** every other task in this module reads `manifest.schema.json` for its contract. TASK-PLUGIN-002 reads `tools[*]` for what MCP tools to register. TASK-PLUGIN-003 reads `commands[*]`. TASK-PLUGIN-004 reads `skills[*]`. TASK-PLUGIN-005 reads `auth.*`. TASK-PLUGIN-006 reads `audit.*`. TASK-PLUGIN-007 reads `targets[*]`. TASK-PLUGIN-008 reads `marketplace.*`.
- **Cross-module:** TASK-MCP-003 (SEP-986 tool naming — clause 6 enforces it), TASK-MCP-006 (tool annotation gating — clause 14 enforces it), TASK-SKILL-111 (description enrichment — manifest `description` 60-480 char range matches the trigger-discovery discipline).

---

## §8 — Example payloads

### Valid minimal manifest (see §3 for content)

### Validation error example

```json
{
  "errors": [
    {
      "json_pointer": "/tools/0/name",
      "constraint": "pattern",
      "expected": "^cyberos\\.[a-z][a-z0-9]*\\.[a-z][a-z0-9_]*$",
      "actual": "execute_workflow",
      "message": "Tool name 'execute_workflow' must follow SEP-986: 'cyberos.{module}.{verb}_{noun}'. Per TASK-MCP-003."
    },
    {
      "json_pointer": "/signature/rekor_uuid",
      "constraint": "required",
      "message": "Missing Rekor transparency-log UUID. Bundles must be Sigstore-signed per TASK-PLUGIN-001 §1 clause 8."
    }
  ]
}
```

### Pack result

```json
{
  "bundle_path": "dist/cyberos-1.0.0.plugin",
  "sha256": "a1b2c3d4e5f6...",
  "size_bytes": 184320,
  "tools_count": 8,
  "commands_count": 4,
  "skills_count": 12
}
```

---

## §9 — Open questions

All resolved.

- ~~Should we support api-key auth in v1?~~ → No, oauth-pkce only per DEC-2406. Re-evaluated for task-PLUGIN-001a (post-v1 schema bump).
- ~~Should reproducibility be opt-in?~~ → No, default-on per DEC-2409. The opt-out `reproducible=False` exists for debugging only.
- ~~Should the packer ship as Rust to match production?~~ → No, Python reference first per DEC-2408. Rust binary lands in TASK-PLUGIN-007 alongside multi-runtime adapter work.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Manifest missing required field | JSONSchema validation | `cyberos-plugin pack` exits 1 with `/<field>: required` | Author adds field |
| Tool name violates SEP-986 | JSONSchema pattern | exit 1 with `/tools/N/name: pattern` | Author renames per cyberos.{module}.{verb}_{noun} |
| schema_version mismatch | JSONSchema const | exit 1 with `/schema_version: const` | Author downgrades to "1.0.0" or waits for task-PLUGIN-001a |
| Capability declared without matching scope | validator.py custom check | exit 1 with `capability '<x>' declared but no tool has scope '<y>'` | Author adds the scope to one of the tools |
| Destructive tool without write scope | validator.py custom check | exit 1 with `tool '<n>' marked destructive but lacks write scope` | Author adds the write scope OR un-marks the annotation |
| Unsigned manifest at pack time | packer.py check | raises `UnsignedManifestError` | Author runs Sigstore sign first, then re-packs |
| Reproducibility broken by clock-injection bug | CI: two consecutive `pack()` calls compare SHA-256 | CI fails | Investigate which file source injects mtime; remove |
| Adapter for `--target` not yet implemented | `pack(target="goose")` in P1 | raises `UnknownTargetError` | User picks a P1 target or waits for P2 |
| Bundle SHA-256 mismatch on verifier rebuild | Sigstore verifier rebuilds bundle, compares hash | verifier rejects | Investigate non-determinism in packer; report bug |
| Mtimes injected by network filesystem | `pack()` on NFS sees host clocks | reproducible check fails | Use local fs for packing; CI runs on local tmpfs |
| Marketplace upload of non-validated bundle | `cyberos-plugin publish` re-runs validate before upload | exit 1 | Author re-runs pack with valid manifest |
| Future schema field appears in v1 manifest | additionalProperties: false | exit 1 with `unknown property` | Field MUST land via task-PLUGIN-001a, not in v1 |
| Empty tools array | `tools.minItems: 1` | exit 1 | A plugin with zero tools is meaningless; add at least one |
| Description shorter than 60 chars | `description.minLength: 60` | exit 1 | Author writes meaningful description |
| Tool input_schema not a JSONSchema object | JSONSchema type | exit 1 | Author fixes input_schema |
| Cross-platform path separator in commands/file | validator: enforces forward slashes | exit 1 | Author uses POSIX paths in manifest |

---

## §11 — Implementation notes

- §11.1 **JSONSchema library choice.** Python `jsonschema` 4.21+ for validation. Custom checks (capability/scope coherence, destructive/write-scope coherence) live in `validator.py` as separate functions called after `jsonschema.validate`. Reason: JSONSchema can't express cross-field constraints cleanly; mixing schema + custom is the standard pattern.

- §11.2 **Reproducible zip implementation.** Use `zipfile.ZipFile` with `ZipInfo` objects constructed manually. Set `date_time = (1980, 1, 1, 0, 0, 0)` (zip-format epoch), `external_attr` to `(0o644 << 16)` for files and `(0o755 << 16) | 0x10` for dirs. Iterate entries in sorted order. Force `compress_type=ZIP_DEFLATED, compresslevel=6` to avoid level-default churn across zlib versions.

- §11.3 **CLI ergonomics.** Use `argparse` (stdlib) not `click` to avoid the dependency. Subcommands: `pack`, `validate`, `doctor`. Each returns exit 0 on success, exit 1 on validation failure, exit 2 on usage error.

- §11.4 **Error message format.** Every validation error MUST include the JSON pointer to the failing field. Errors include a hint pointing back to this task (e.g. "per TASK-PLUGIN-001 clause 6") so the author can find the rationale.

- §11.5 **Performance.** Validation runs in <100ms for manifests with up to 200 tools (typical). The reproducibility-stripping pass is O(n_files) — for a 12-skill plugin with ~50 files, packing completes in <200ms.

- §11.6 **Schema versioning escape valve.** When v2 lands (task-PLUGIN-001a), the validator will accept both `schema_version: "1.0.0"` (forwarded to v1 validation rules) and `schema_version: "2.0.0"` (v2 rules). The dual-validator lives in `validator.py::validate_dispatch`. v1 plugins remain installable indefinitely.

- §11.7 **Marketplace pre-check.** When TASK-PLUGIN-008 lands, `cyberos-plugin publish` runs `validate` before upload. This is duplicative with packer-time validation but is mandatory because plugins may be uploaded by tools other than the official packer.

- §11.8 **Why not use Pydantic.** Pydantic is great for runtime models but JSONSchema is the wire format. Maintaining both Pydantic models and JSONSchema would require synchronisation. The cost of the synchronisation is higher than the ergonomic benefit Pydantic provides for a once-per-pack validation.

- §11.9 **Sigstore verification at install time.** Hosts that implement signature verification (Claude Code planned, Cursor TBD) re-anchor the Rekor entry independently. The host MUST NOT trust the `sigstore_bundle` field blindly — it MUST fetch the entry from Rekor by UUID and verify. The bundle field is included for offline-first reasons.

- §11.10 **Manifest as code.** The shipped manifests at `modules/plugin/manifests/*.plugin.json` are checked into git. They are the canonical source of truth for what the CyberOS plugin contains. The packer reads from there; it does not synthesise manifests from runtime introspection. Reason: deterministic builds + reviewable in diff.

---

*End of TASK-PLUGIN-001 spec.*
