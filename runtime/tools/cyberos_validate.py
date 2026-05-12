#!/usr/bin/env python3
"""
cyberos-validate — independent validator for `.cyberos-memory/` stores.

Walks any `.cyberos-memory/` directory and reports findings against
CyberOS-AGENTS.md sections §4 (operations), §5 (memory format), §7 (audit ledger),
§9 (denylist + supersedes graph), §13 (state classifier).

This is the v0 reference implementation. Use against any existing store.

Exit codes
----------
0 = no findings worse than INFO
1 = at least one WARN finding (cap headroom, stale stats, dangling refs)
2 = at least one CRITICAL finding (chain break, schema invariant, supersedes cycle)
3 = invocation error (path missing, manifest unparseable, etc.)

Usage
-----
    cyberos-validate <path>                  # text output (default)
    cyberos-validate --format json <path>    # machine-parseable
    cyberos-validate --format sarif <path>   # IDE integration
    cyberos-validate --self-test             # run against tests/vectors/
    cyberos-validate --quiet <path>          # only critical findings to stdout

Dependencies (optional)
-----------------------
    pyyaml         — required if any memories present (frontmatter parse)
    rfc8785        — preferred for §7.2 JCS canonical JSON
                     (falls back to approx hand-rolled if missing)

Author: CyberOS local-optimization Stage 2
License: Same as CyberOS-AGENTS.md
"""

from __future__ import annotations

import argparse
import dataclasses
import datetime as dt
import hashlib
import json
import os
import re
import sys
import unicodedata
from pathlib import Path
from typing import Any, Iterable

# ---------------------------------------------------------------------------
# Optional dependencies
# ---------------------------------------------------------------------------

try:
    import yaml  # type: ignore
except ImportError:  # pragma: no cover
    yaml = None  # type: ignore

try:
    import rfc8785  # type: ignore
    _HAS_JCS = True
except ImportError:  # pragma: no cover
    _HAS_JCS = False


# ---------------------------------------------------------------------------
# Severity model (§8.7)
# ---------------------------------------------------------------------------

CRITICAL = "CRITICAL"
WARN = "WARN"
INFO = "INFO"

_SEVERITY_ORDER = {INFO: 0, WARN: 1, CRITICAL: 2}


@dataclasses.dataclass
class Finding:
    severity: str
    section: str  # AGENTS.md section reference, e.g. "§7.2"
    code: str  # short stable identifier, e.g. "chain-link-mismatch"
    path: str  # relative path inside `.cyberos-memory/`, or "manifest.json", etc.
    message: str  # human-readable explanation
    details: dict[str, Any] = dataclasses.field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        return {
            "severity": self.severity,
            "section": self.section,
            "code": self.code,
            "path": self.path,
            "message": self.message,
            "details": self.details,
        }


# ---------------------------------------------------------------------------
# Memory-id + audit-id regex (§5.2)
# ---------------------------------------------------------------------------

_UUID_V7_RE = re.compile(
    r"^(mem|evt)_[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$"
)
_ULID_RE = re.compile(r"^(mem|evt)_[0-9A-HJKMNP-TV-Z]{26}$")
_ISO8601_RE = re.compile(
    r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?([+-]\d{2}:\d{2}|Z)$"
)


def is_valid_memory_id(value: str) -> bool:
    """Per §5.2: UUIDv7 or ULID, prefix `mem_` or `evt_`."""
    if not isinstance(value, str):
        return False
    return bool(_UUID_V7_RE.match(value)) or bool(_ULID_RE.match(value))


def is_valid_iso8601(value: Any) -> bool:
    """Per §5.2 + DEC-088: accept ISO-8601 string OR tz-aware datetime."""
    if isinstance(value, dt.datetime):
        return value.tzinfo is not None
    if isinstance(value, str):
        return bool(_ISO8601_RE.match(value))
    return False


# ---------------------------------------------------------------------------
# Resource caps (§5.5)
# ---------------------------------------------------------------------------

CAP_FILE_BODY_HARD = 30 * 1024  # 30 KB
CAP_FILE_BODY_IDEAL = 10 * 1024  # 10 KB
CAP_FRONTMATTER_HARD = 4 * 1024  # 4 KB
CAP_STORE_HARD = 10 * 1024 * 1024  # 10 MB
CAP_STORE_SOFT = 1 * 1024 * 1024  # 1 MB
CAP_STORE_FILES = 10_000
CAP_DIR_DEPTH = 12
CAP_AUDIT_ROW = 64 * 1024


# ---------------------------------------------------------------------------
# RFC 8785 JCS canonical JSON (§7.2)
# ---------------------------------------------------------------------------

def canonicalize_json(value: Any) -> bytes:
    """Return RFC 8785 JCS canonical bytes.

    Uses `rfc8785` package if available (bit-identical). Falls back to a
    `json.dumps(sort_keys=True, separators=(",", ":"), ensure_ascii=False)`
    approximation, which differs on number serialisation (Python emits 1.0,
    JCS emits 1) and on UTF-16 vs byte-level key ordering. The fallback is
    flagged as INFO per §7.2 cross-writer-version compatibility rule.
    """
    if _HAS_JCS:
        return rfc8785.dumps(value)  # type: ignore[no-any-return]
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
    ).encode("utf-8")


# ---------------------------------------------------------------------------
# Frontmatter parsing
# ---------------------------------------------------------------------------

_FRONTMATTER_OPEN = "---\n"


def split_frontmatter(text: str) -> tuple[str | None, str]:
    """Return (frontmatter_yaml, body) or (None, body) if no frontmatter.

    Per §4.3: must open with `---\\n`, close with exactly one `\\n---\\n`
    (or `\\n---` at EOF). Code-fenced examples are exempt from the
    secondary-block check (DEC-087); we do NOT enforce that here — this
    parser is tolerant for read-side validation.
    """
    if not text.startswith(_FRONTMATTER_OPEN):
        return None, text

    rest = text[len(_FRONTMATTER_OPEN):]
    # Find closing ---\n or ---<EOF>
    close_match = re.search(r"\n---(\n|$)", rest)
    if not close_match:
        return None, text  # malformed, treat as no frontmatter

    fm_yaml = rest[: close_match.start()]
    body = rest[close_match.end():]
    return fm_yaml, body


# ---------------------------------------------------------------------------
# Validator
# ---------------------------------------------------------------------------

class Validator:
    """Walks a `.cyberos-memory/` and accumulates findings."""

    def __init__(self, root: Path) -> None:
        self.root = root
        self.findings: list[Finding] = []
        self.manifest: dict[str, Any] | None = None
        self.audit_rows: list[dict[str, Any]] = []
        self.memory_files: dict[str, dict[str, Any]] = {}  # rel_path -> frontmatter
        self.memory_ids: set[str] = set()  # all memory_ids encountered

    # -- top-level entry -----------------------------------------------------

    def run(self) -> int:
        """Run all checks. Return worst severity as exit code (0/1/2)."""
        if not self.root.exists():
            self._add(CRITICAL, "§13.0", "store-missing", str(self.root),
                      f"Path does not exist: {self.root}")
            return 3
        if not (self.root / "manifest.json").exists():
            self._add(CRITICAL, "§13.0", "manifest-missing", "manifest.json",
                      "manifest.json not found — store appears uninitialised")
            return 3

        # Batch 11 / Aspect 5.7 — acquire shared lock around the validate pass.
        # If brain_writer holds the exclusive lock, wait up to 5s; otherwise
        # degrade silently (best-effort, never blocks the validate).
        self._maybe_acquire_shared_lock()

        self._check_manifest()
        self._walk_audit_ledger()
        self._walk_memory_files()
        self._check_supersedes_graph()
        self._check_orphan_files()
        self._check_resource_caps()
        self._check_tombstone_consistency()
        self._check_reconciliation_checkpoint()  # Stage 1 §8.7 phase 4 amendment
        self._check_shamir_consistency()  # Stage 5 §5.6.3
        self._check_merkle_checkpoints()  # Stage 6 §7.6 + §8.7 phase 4 ext
        self._check_compacted_ledgers()  # Stage 6 §7.7
        self._check_frontmatter_strictness()  # Batch 9: fix mutation-test gaps
        self._check_content_gate_body()  # Batch 9: fix mutation-test gap (§4.2)
        self._run_pluggable_validators()  # Aspect 12.1
        return self._exit_code()

    # -- Batch 9: tighten frontmatter checks (mutation-test surfaced gaps) ---

    def _check_frontmatter_strictness(self) -> None:
        """Tighten validation for fields that mutation testing exposed as not enforced.

        Bugs caught by `cyberos mutation-test` in Batch 8:
          - negative-version          → reject version < 1
          - remove-provenance         → require provenance: block
          - invalid-sync-class        → enforce sync_class enum
        """
        valid_sync = {"local-only", "publishable", "shared", "client-visible"}
        for rel, fm in self.memory_files.items():
            if not isinstance(fm, dict):
                continue
            # 1. Version must be a positive integer
            ver = fm.get("version")
            if ver is not None:
                try:
                    n = int(ver)
                    if n < 1:
                        self._add(CRITICAL, "§5.1",
                                  "invalid-version",
                                  rel,
                                  f"version must be ≥ 1; got {ver}")
                except (TypeError, ValueError):
                    self._add(CRITICAL, "§5.1",
                              "invalid-version-type",
                              rel,
                              f"version must be integer; got {ver!r}")

            # 2. Provenance is required per §5.1
            if "provenance" not in fm:
                self._add(WARN, "§5.1",
                          "provenance-missing",
                          rel,
                          "frontmatter has no `provenance:` block")
            elif not isinstance(fm.get("provenance"), dict):
                self._add(WARN, "§5.1",
                          "provenance-malformed",
                          rel,
                          f"provenance: must be a dict; got {type(fm.get('provenance')).__name__}")

            # 3. sync_class must be one of the 4 valid enums
            sync = fm.get("sync_class")
            if sync is not None and sync not in valid_sync:
                self._add(WARN, "§17",
                          "invalid-sync-class",
                          rel,
                          f"sync_class must be one of {sorted(valid_sync)}; got {sync!r}")

    def _check_content_gate_body(self) -> None:
        """§4.2 content-gate scan of memory bodies (Batch 9 fix).

        Mutation test inject-marker SURVIVED because the old validator
        only scanned frontmatter. Now scan the body for prompt-injection
        markers and surface as WARN (CRITICAL would block valid memories
        that document the markers — e.g. this very function references
        them).
        """
        markers = [
            "[INST]", "<system>", "<<SYS>>",
            "<|im_start|>", "<|system|>", "<|assistant|>",
            "###Instruction", "###System:",
            "ignore previous instructions", "ignore the above",
        ]
        # Exclude paths that legitimately mention markers as documentation
        WHITELIST_SUBSTR = (
            "tests/fuzz/", "tests/mutation/",
            "memories/refinements/REF-",  # protocol amendments reference markers
            "meta/validators/",
            "/conflicts/",
            "/postmortems/",
        )
        for rel, _ in self.memory_files.items():
            if any(w in rel for w in WHITELIST_SUBSTR):
                continue
            path = self.root / rel
            try:
                text = path.read_text(encoding="utf-8")
            except Exception:
                continue
            # Strip frontmatter — we only scan the body
            if text.startswith("---\n"):
                end = text.find("\n---\n", 4)
                if end >= 0:
                    body = text[end + 5:]
                else:
                    body = text
            else:
                body = text
            lower = body.lower()
            for m in markers:
                if m.lower() in lower:
                    self._add(WARN, "§4.2",
                              "content-gate-injection-marker",
                              rel,
                              f"body contains potential injection marker: {m!r}")
                    break  # one finding per file is enough

    # -- Aspect 12.1: pluggable validators -----------------------------------

    def _run_pluggable_validators(self) -> None:
        """Discover and run user-defined validator checks in meta/validators/.

        Each *.py file there exports `check(memory: dict, manifest: dict) -> list[dict]`.
        Each returned dict must have keys: severity, code, message. Optional: path.
        Exceptions are caught and surfaced as WARN findings so a buggy plugin
        cannot block the rest of validation.
        """
        plugin_dir = self.root / "meta" / "validators"
        if not plugin_dir.is_dir():
            return
        plugins = sorted(p for p in plugin_dir.glob("check-*.py") if p.is_file())
        if not plugins:
            return
        # Load manifest dict once
        try:
            manifest = json.loads((self.root / "manifest.json").read_text())
        except Exception:
            manifest = {}
        import importlib.util
        for plugin in plugins:
            try:
                spec = importlib.util.spec_from_file_location(f"cyberos_check_{plugin.stem}", plugin)
                module = importlib.util.module_from_spec(spec)
                spec.loader.exec_module(module)
                if not hasattr(module, "check"):
                    self._add(WARN, "§12.1", "validator-plugin-no-check",
                              str(plugin.relative_to(self.root)),
                              f"plugin missing `check(memory, manifest)` function")
                    continue
            except Exception as e:
                self._add(WARN, "§12.1", "validator-plugin-load-error",
                          str(plugin.relative_to(self.root)),
                          f"plugin failed to import: {e}")
                continue

            for rel, fm in self.memory_files.items():
                try:
                    results = module.check(dict(fm), manifest) or []
                except Exception as e:
                    self._add(WARN, "§12.1", "validator-plugin-error",
                              rel,
                              f"plugin {plugin.name} raised on {rel}: {e}")
                    continue
                if not isinstance(results, list):
                    continue
                for r in results:
                    if not isinstance(r, dict) or "severity" not in r:
                        continue
                    sev = r.get("severity", WARN)
                    if sev not in (CRITICAL, WARN, INFO):
                        sev = WARN
                    self._add(sev, "§12.1",
                              r.get("code", "plugin-finding"),
                              r.get("path", rel),
                              r.get("message", "(no message)"),
                              details={"plugin": plugin.name})

    # -- manifest ------------------------------------------------------------

    def _check_manifest(self) -> None:
        path = self.root / "manifest.json"
        try:
            with path.open("r", encoding="utf-8") as f:
                self.manifest = json.load(f)
        except json.JSONDecodeError as e:
            self._add(CRITICAL, "§6", "manifest-unparseable", "manifest.json",
                      f"manifest.json is not valid JSON: {e}")
            return

        m = self.manifest
        # Required top-level fields per §6
        for field in ("memory_layer", "tenant", "owner", "project",
                      "scope_root", "audit_chain_head"):
            if field not in m:
                self._add(CRITICAL, "§6", "manifest-missing-field",
                          "manifest.json",
                          f"required field absent: {field}")

        # project.root_path must match real local path (§0.1)
        proj = m.get("project", {})
        if "root_path" not in proj:
            self._add(CRITICAL, "§0.1", "manifest-missing-root-path",
                      "manifest.json",
                      "project.root_path absent — required by §0.1")

        # protocol pin (§0.5)
        if "protocol" not in m:
            self._add(WARN, "§0.5", "manifest-missing-protocol-pin",
                      "manifest.json",
                      "manifest.protocol absent — §0.5 SHA pin not enforced. "
                      "Old store predating §0.5; consider running §0.5 baseline.")

    # -- audit ledger --------------------------------------------------------

    def _walk_audit_ledger(self) -> None:
        audit_dir = self.root / "audit"
        if not audit_dir.exists():
            self._add(CRITICAL, "§7", "audit-dir-missing", "audit/",
                      "audit/ directory absent")
            return

        ledger_files = sorted(audit_dir.glob("*.jsonl"))
        if not ledger_files:
            self._add(CRITICAL, "§7", "audit-ledger-empty", "audit/",
                      "no .jsonl ledger files in audit/")
            return

        prev_chain: str | None = None
        for ledger in ledger_files:
            try:
                with ledger.open("r", encoding="utf-8") as f:
                    for lineno, line in enumerate(f, start=1):
                        line = line.rstrip("\n")
                        if not line:
                            continue
                        try:
                            row = json.loads(line)
                        except json.JSONDecodeError as e:
                            self._add(CRITICAL, "§7.3", "audit-row-unparseable",
                                      f"audit/{ledger.name}",
                                      f"line {lineno}: not valid JSON: {e}",
                                      details={"line": lineno})
                            return  # freeze further audit checks
                        self.audit_rows.append(row)

                        # Chain LINK invariant (§7.2)
                        actual_prev = row.get("prev_chain")
                        if prev_chain is not None and actual_prev != prev_chain:
                            self._add(CRITICAL, "§7.2", "chain-link-mismatch",
                                      f"audit/{ledger.name}",
                                      f"line {lineno}: row.prev_chain ({actual_prev}) "
                                      f"does not equal previous row's chain "
                                      f"({prev_chain})",
                                      details={
                                          "line": lineno,
                                          "expected_prev_chain": prev_chain,
                                          "actual_prev_chain": actual_prev,
                                          "audit_id": row.get("audit_id"),
                                      })
                        prev_chain = row.get("chain")

                        # Row size cap (§5.5)
                        if len(line.encode("utf-8")) > CAP_AUDIT_ROW:
                            self._add(WARN, "§5.5", "audit-row-oversized",
                                      f"audit/{ledger.name}",
                                      f"line {lineno}: row exceeds {CAP_AUDIT_ROW} bytes",
                                      details={"line": lineno})

                        # audit_id format
                        aid = row.get("audit_id")
                        if aid and not is_valid_memory_id(aid):
                            self._add(WARN, "§5.2", "audit-id-malformed",
                                      f"audit/{ledger.name}",
                                      f"line {lineno}: audit_id `{aid}` is not "
                                      f"UUIDv7 or ULID per §5.2")
            except OSError as e:
                self._add(CRITICAL, "§7", "audit-read-error",
                          f"audit/{ledger.name}",
                          f"cannot read ledger: {e}")
                return

        # audit_chain_head reachability
        if self.manifest:
            head = self.manifest.get("audit_chain_head")
            if head:
                chains = {row.get("chain") for row in self.audit_rows}
                # genesis case
                if head == "sha256:" + "0" * 64 and not self.audit_rows:
                    pass  # ok — pristine store
                elif head not in chains:
                    self._add(CRITICAL, "§13.0", "audit-chain-head-unreachable",
                              "manifest.json",
                              f"manifest.audit_chain_head ({head}) does not "
                              f"appear as any row's chain in the ledger",
                              details={"head": head})

    # -- memory files --------------------------------------------------------

    # Exemption paths (per AGENTS.md §0.5, §4.2, §8.7).
    # These files are NOT memories — they are protocol-doc archives, deterministic
    # self-audit reports, registries, or rule-definition files. Skip §5.1 schema
    # checks and §5.5 body-size caps.
    _MEMORY_EXEMPT_PREFIXES = (
        "meta/protocol-history/",  # §0.5: verbatim AGENTS.md archives
        "meta/health/",  # §8.7: deterministic self-audit reports
    )
    _MEMORY_EXEMPT_FILES = {
        # §5.2: legacy-id registry, frontmatter-exempt
        "meta/legacy-ids.md",
        # §4.2 rule-definition exemption set
        "meta/tombstones.md",
        "meta/classification-rules.md",
        "meta/retention-rules.md",
        "meta/conflict-resolutions.md",
    }

    def _is_memory_exempt(self, rel: str) -> bool:
        if rel in self._MEMORY_EXEMPT_FILES:
            return True
        for prefix in self._MEMORY_EXEMPT_PREFIXES:
            if rel.startswith(prefix):
                return True
        return False

    def _walk_memory_files(self) -> None:
        if yaml is None:
            self._add(WARN, "§5.1", "yaml-missing",
                      str(self.root),
                      "pyyaml not installed; skipping frontmatter validation. "
                      "Install via `pip install pyyaml` for full coverage.")
            return

        scope_dirs = ["company", "module", "member", "client", "project",
                      "persona", "memories", "meta"]
        for scope in scope_dirs:
            scope_path = self.root / scope
            if not scope_path.exists():
                continue
            for md in scope_path.rglob("*.md"):
                rel = md.relative_to(self.root).as_posix()
                if md.name == "README.md":
                    continue  # README is denylist-exempt per §4.2; skip schema check
                if self._is_memory_exempt(rel):
                    continue  # §0.5/§4.2/§8.7 exempt — not a memory
                self._check_memory_file(md, rel)

    def _check_memory_file(self, path: Path, rel: str) -> None:
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError) as e:
            self._add(CRITICAL, "§4.3", "memory-read-error", rel,
                      f"cannot decode as UTF-8: {e}")
            return

        # Body-size cap (§5.5)
        body_bytes = len(text.encode("utf-8"))
        if body_bytes > CAP_FILE_BODY_HARD:
            self._add(CRITICAL, "§5.5", "body-cap-exceeded", rel,
                      f"file body {body_bytes} bytes exceeds hard cap "
                      f"{CAP_FILE_BODY_HARD}")
        elif body_bytes > CAP_FILE_BODY_IDEAL:
            self._add(INFO, "§5.5", "body-cap-soft", rel,
                      f"file body {body_bytes} bytes exceeds ideal cap "
                      f"{CAP_FILE_BODY_IDEAL}; consider §8.4 split")

        # File hygiene (§4.3)
        if "﻿" in text:
            self._add(CRITICAL, "§4.3", "bom-present", rel,
                      "UTF-8 BOM present (forbidden by §4.3)")
        if "\r" in text and "\r\n" not in text.replace("\r\n", ""):
            self._add(CRITICAL, "§4.3", "bare-cr", rel,
                      "bare \\r not part of \\r\\n (forbidden by §4.3)")

        fm_yaml, _body = split_frontmatter(text)
        if fm_yaml is None:
            self._add(WARN, "§5.1", "no-frontmatter", rel,
                      "no parseable frontmatter; cannot validate §5.1 schema")
            return

        fm_bytes = len(fm_yaml.encode("utf-8"))
        if fm_bytes > CAP_FRONTMATTER_HARD:
            self._add(CRITICAL, "§5.5", "frontmatter-cap-exceeded", rel,
                      f"frontmatter {fm_bytes} bytes exceeds hard cap "
                      f"{CAP_FRONTMATTER_HARD}")

        try:
            fm = yaml.safe_load(fm_yaml)
        except yaml.YAMLError as e:
            self._add(CRITICAL, "§5.1", "frontmatter-yaml-error", rel,
                      f"frontmatter not valid YAML: {e}")
            return

        if not isinstance(fm, dict):
            self._add(CRITICAL, "§5.1", "frontmatter-not-mapping", rel,
                      f"frontmatter is not a YAML mapping (got {type(fm).__name__})")
            return

        self.memory_files[rel] = fm

        # memory_id (§5.1, §5.2)
        mid = fm.get("memory_id")
        if not mid:
            self._add(CRITICAL, "§5.1", "memory-id-missing", rel,
                      "frontmatter missing memory_id")
        elif not is_valid_memory_id(mid):
            # Check legacy registry
            legacy_path = self.root / "meta" / "legacy-ids.md"
            if legacy_path.exists() and mid in legacy_path.read_text(
                    encoding="utf-8"):
                self._add(INFO, "§5.2", "memory-id-legacy", rel,
                          f"memory_id `{mid}` is non-conforming but registered "
                          f"in meta/legacy-ids.md (allowed per §5.2)")
            else:
                self._add(CRITICAL, "§5.2", "memory-id-malformed", rel,
                          f"memory_id `{mid}` is not UUIDv7/ULID and not in "
                          f"meta/legacy-ids.md")
        else:
            if mid in self.memory_ids:
                self._add(CRITICAL, "§5.1", "memory-id-duplicate", rel,
                          f"memory_id `{mid}` already used by another file")
            self.memory_ids.add(mid)

        # Required fields per §5.1
        for field in ("scope", "classification", "authority", "version",
                      "created_at", "last_updated_at"):
            if field not in fm:
                self._add(CRITICAL, "§5.1", "frontmatter-missing-field", rel,
                          f"required field absent: {field}")

        # Authority hierarchy (§5.3)
        auth = fm.get("authority")
        if auth and auth not in {
            "human-edited", "human-confirmed",
            "llm-explicit", "llm-implicit",
        }:
            self._add(CRITICAL, "§5.3", "authority-invalid", rel,
                      f"authority `{auth}` not in §5.3 hierarchy")

        # Classification (§5.4)
        cls = fm.get("classification")
        if cls and cls not in {"personnel", "client", "operational", "public"}:
            self._add(CRITICAL, "§5.4", "classification-invalid", rel,
                      f"classification `{cls}` not in §5.4 set")

        # Confidence cap (§5.2)
        prov = fm.get("provenance") or {}
        conf = prov.get("confidence") if isinstance(prov, dict) else None
        if isinstance(conf, bool):
            self._add(CRITICAL, "§5.2", "confidence-bool", rel,
                      "provenance.confidence is a boolean (rejected per §5.2)")
        elif conf is not None and not isinstance(conf, (int, float)):
            self._add(CRITICAL, "§5.2", "confidence-not-number", rel,
                      f"provenance.confidence type {type(conf).__name__} "
                      f"(must be number per §5.2)")
        elif isinstance(conf, (int, float)) and not (0.0 <= conf <= 1.0):
            self._add(CRITICAL, "§5.2", "confidence-out-of-range", rel,
                      f"provenance.confidence {conf} not in [0.0, 1.0]")

        # Timestamps (§5.2)
        for ts_field in ("created_at", "last_updated_at", "expires_at",
                         "deleted_at"):
            v = fm.get(ts_field)
            if v is not None and not is_valid_iso8601(v):
                self._add(CRITICAL, "§5.2", "timestamp-invalid", rel,
                          f"{ts_field}: not ISO-8601 string nor tz-aware datetime")

        # Stage 5 encryption envelope (§5.6.1)
        self._check_encryption_envelope(fm, rel)

        # Temporal monotonicity (§5.2)
        ca = fm.get("created_at")
        lua = fm.get("last_updated_at")
        if isinstance(ca, dt.datetime) and isinstance(lua, dt.datetime):
            if ca > lua:
                self._add(CRITICAL, "§5.2", "temporal-monotonicity",
                          rel, f"created_at > last_updated_at")
        elif isinstance(ca, str) and isinstance(lua, str):
            if ca > lua:  # ISO-8601 string compare = chronological
                self._add(CRITICAL, "§5.2", "temporal-monotonicity",
                          rel, f"created_at > last_updated_at")

    # -- supersedes graph (§9.5) --------------------------------------------

    def _check_supersedes_graph(self) -> None:
        # Build adjacency map: memory_id -> supersedes targets
        supersedes_map: dict[str, list[str]] = {}
        superseded_by_map: dict[str, str] = {}
        rel_for_id: dict[str, str] = {}

        for rel, fm in self.memory_files.items():
            mid = fm.get("memory_id")
            if not mid:
                continue
            rel_for_id[mid] = rel
            sup = fm.get("supersedes")
            if isinstance(sup, list):
                supersedes_map[mid] = [s for s in sup if isinstance(s, str)]
            elif isinstance(sup, str):
                supersedes_map[mid] = [sup]
            else:
                supersedes_map[mid] = []
            sb = fm.get("superseded_by")
            if isinstance(sb, str):
                superseded_by_map[mid] = sb

        # Dangling supersedes
        for mid, targets in supersedes_map.items():
            for t in targets:
                if t not in self.memory_ids and t not in rel_for_id:
                    self._add(CRITICAL, "§9.5", "supersedes-dangling",
                              rel_for_id.get(mid, mid),
                              f"`{mid}` supersedes `{t}` but `{t}` not found")

        # Dangling superseded_by
        for mid, sb in superseded_by_map.items():
            if sb not in self.memory_ids and sb not in rel_for_id:
                self._add(CRITICAL, "§9.5", "superseded-by-dangling",
                          rel_for_id.get(mid, mid),
                          f"`{mid}` superseded_by `{sb}` but `{sb}` not found")

        # Cycle detection (DFS)
        WHITE, GRAY, BLACK = 0, 1, 2
        color: dict[str, int] = {mid: WHITE for mid in supersedes_map}

        def dfs(node: str, path: list[str]) -> None:
            if color.get(node, WHITE) == GRAY:
                cycle = path[path.index(node):] + [node]
                self._add(CRITICAL, "§9.5", "supersedes-cycle",
                          rel_for_id.get(node, node),
                          f"supersedes cycle: {' → '.join(cycle)}")
                return
            if color.get(node, WHITE) == BLACK:
                return
            color[node] = GRAY
            for next_node in supersedes_map.get(node, []):
                dfs(next_node, path + [node])
            color[node] = BLACK

        for mid in list(supersedes_map.keys()):
            if color.get(mid, WHITE) == WHITE:
                dfs(mid, [])

        # superseded_by != null ⇒ tombstoned: true (§9.5)
        for mid, sb in superseded_by_map.items():
            rel = rel_for_id.get(mid)
            if rel and not self.memory_files[rel].get("tombstoned"):
                self._add(WARN, "§9.5", "superseded-not-tombstoned", rel,
                          f"superseded_by set but tombstoned not true")

    # -- orphan files --------------------------------------------------------

    def _check_orphan_files(self) -> None:
        # An orphan = memory file whose path has no `create | str_replace |
        # insert | rename` audit row that's not later reverted.
        # v0 simplification: we only check that every memory file has SOME
        # audit row referencing it.
        paths_in_audit: set[str] = set()
        for row in self.audit_rows:
            p = row.get("path")
            if p:
                # Strip leading `.cyberos-memory/` prefix
                p = p.removeprefix(".cyberos-memory/")
                paths_in_audit.add(p)
        for rel in self.memory_files:
            if rel not in paths_in_audit:
                self._add(WARN, "§8.7", "orphan-file", rel,
                          "memory file has no audit row referencing its path")

    # -- resource caps -------------------------------------------------------

    def _check_resource_caps(self) -> None:
        total_size = 0
        file_count = 0
        for p in self.root.rglob("*"):
            if not p.is_file():
                continue
            # Exclude index/, exports/, .lock, .tmp.* per §11.1
            rel = p.relative_to(self.root).as_posix()
            if rel.startswith(("index/", "exports/")):
                continue
            if p.name == ".lock" or p.name.startswith(".tmp."):
                continue
            total_size += p.stat().st_size
            file_count += 1

        if total_size > CAP_STORE_HARD:
            self._add(CRITICAL, "§5.5", "store-size-exceeds-hard",
                      str(self.root),
                      f"total size {total_size} bytes exceeds hard cap "
                      f"{CAP_STORE_HARD}")
        elif total_size > 0.8 * CAP_STORE_HARD:
            self._add(WARN, "§5.5", "store-size-80pct",
                      str(self.root),
                      f"total size {total_size} bytes is "
                      f"{total_size / CAP_STORE_HARD:.0%} of hard cap")
        if file_count > CAP_STORE_FILES:
            self._add(CRITICAL, "§5.5", "file-count-exceeds-hard",
                      str(self.root),
                      f"file count {file_count} exceeds hard cap "
                      f"{CAP_STORE_FILES}")
        elif file_count > 0.8 * CAP_STORE_FILES:
            self._add(WARN, "§5.5", "file-count-80pct",
                      str(self.root),
                      f"file count {file_count} is "
                      f"{file_count / CAP_STORE_FILES:.0%} of hard cap")

    # -- Stage 6 Merkle checkpoint + compacted-ledger checks ----------------

    def _build_merkle_root(self, chains: list[str]) -> str:
        """Per AGENTS.md §7.6: leaves are raw chain bytes; pad odd levels by
        duplicating last; internal = sha256(left || right); root = sha256:hex."""
        if not chains:
            return "sha256:" + "0" * 64
        level = [bytes.fromhex(c.replace("sha256:", "")) for c in chains]
        while len(level) > 1:
            if len(level) % 2:
                level.append(level[-1])
            level = [hashlib.sha256(level[i] + level[i + 1]).digest()
                     for i in range(0, len(level), 2)]
        return "sha256:" + level[0].hex()

    def _check_merkle_checkpoints(self) -> None:
        """For every op:'consolidation_run' row carrying merkle_root, recompute
        the root over rows since previous checkpoint and verify equality."""
        consolidation_rows: list[tuple[int, dict]] = [
            (i, r) for i, r in enumerate(self.audit_rows)
            if r.get("op") == "consolidation_run" and "merkle_root" in r
        ]
        if not consolidation_rows:
            return  # no checkpoints yet — nothing to verify
        prev_idx = -1
        for idx, row in consolidation_rows:
            # Rows since previous checkpoint (or genesis): rows from
            # prev_idx+1 through idx-1 inclusive (the consolidation row itself
            # is the checkpoint, not a leaf)
            leaves = [r.get("chain") for r in self.audit_rows[prev_idx + 1:idx]
                      if r.get("chain")]
            recomputed = self._build_merkle_root(leaves)
            stored = row.get("merkle_root")
            if stored != recomputed:
                self._add(CRITICAL, "§7.6", "merkle-checkpoint-divergence",
                          f"audit/{row.get('audit_id', '?')}",
                          f"merkle_root mismatch: stored {stored[:24]}... "
                          f"recomputed {recomputed[:24]}... "
                          f"over {len(leaves)} rows since previous checkpoint",
                          details={"audit_id": row.get("audit_id"),
                                   "leaves_count": len(leaves)})
            prev_idx = idx

    def _check_compacted_ledgers(self) -> None:
        """For every audit/<YYYY-MM>.compacted.jsonl, verify each row's
        merkle_proof against the period's checkpoint root."""
        audit_dir = self.root / "audit"
        if not audit_dir.exists():
            return
        compacted = sorted(audit_dir.glob("*.compacted.jsonl"))
        if not compacted:
            return
        # Build a checkpoint-root lookup: we need to find the merkle_root that
        # corresponds to each compacted period.
        roots_by_period = {}
        for r in self.audit_rows:
            if r.get("op") == "consolidation_run" and "merkle_root" in r:
                ts = r.get("ts", "")
                period = ts[:7]  # YYYY-MM
                # Last checkpoint of each period wins (representative root)
                roots_by_period[period] = r.get("merkle_root")

        for ledger in compacted:
            period = ledger.stem.replace(".compacted", "")  # 2026-05.compacted -> 2026-05
            expected_root = roots_by_period.get(period)
            if not expected_root:
                self._add(WARN, "§7.7", "compacted-ledger-no-root",
                          f"audit/{ledger.name}",
                          f"compacted ledger has no corresponding consolidation_run "
                          f"merkle_root for period {period}; cannot verify proofs")
                continue
            try:
                with ledger.open("r", encoding="utf-8") as f:
                    for lineno, line in enumerate(f, 1):
                        if not line.strip():
                            continue
                        try:
                            entry = json.loads(line)
                        except json.JSONDecodeError:
                            self._add(CRITICAL, "§7.7",
                                      "compacted-row-unparseable",
                                      f"audit/{ledger.name}",
                                      f"line {lineno}: not valid JSON")
                            continue
                        proof = entry.get("merkle_proof")
                        chain = entry.get("final_chain")
                        if not proof or not chain:
                            self._add(CRITICAL, "§7.7",
                                      "compacted-row-missing-proof",
                                      f"audit/{ledger.name}",
                                      f"line {lineno}: missing merkle_proof or final_chain")
                            continue
                        # Verify proof: walk the proof path applying SHA-256
                        leaf = bytes.fromhex(chain.replace("sha256:", ""))
                        current = leaf
                        for sibling_entry in proof:
                            sibling = bytes.fromhex(
                                sibling_entry["hash"].replace("sha256:", ""))
                            if sibling_entry.get("position") == "left":
                                current = hashlib.sha256(sibling + current).digest()
                            else:
                                current = hashlib.sha256(current + sibling).digest()
                        derived = "sha256:" + current.hex()
                        if derived != expected_root:
                            self._add(CRITICAL, "§7.7",
                                      "merkle-proof-divergence",
                                      f"audit/{ledger.name}",
                                      f"line {lineno}: merkle_proof for "
                                      f"{entry.get('memory_id', '?')} fails "
                                      f"to derive period root")
            except OSError as e:
                self._add(CRITICAL, "§7.7", "compacted-ledger-read-error",
                          f"audit/{ledger.name}", str(e))

    # -- Stage 5 encryption envelope checks ---------------------------------

    def _check_encryption_envelope(self, fm: dict, rel: str) -> None:
        """If memory has `encrypted: true`, verify the §5.6.1 envelope shape."""
        if not fm.get("encrypted"):
            return
        block = fm.get("encryption")
        if not isinstance(block, dict):
            self._add(CRITICAL, "§5.6", "encryption-block-missing", rel,
                      "encrypted: true but `encryption:` block absent")
            return
        algo = block.get("algorithm")
        if algo not in ("xchacha20poly1305-ietf", "xchacha20poly1305-ietf-v0"):
            self._add(CRITICAL, "§5.6", "encryption-algo-unrecognised", rel,
                      f"unknown algorithm `{algo}`")
        nonce = block.get("nonce")
        if not isinstance(nonce, str):
            self._add(CRITICAL, "§5.6", "encryption-nonce-missing", rel,
                      "`encryption.nonce` not a string")
            return
        try:
            import base64
            decoded = base64.b64decode(nonce)
            if len(decoded) != 24:
                self._add(CRITICAL, "§5.6", "encryption-nonce-length", rel,
                          f"nonce must be 24 bytes (got {len(decoded)})")
        except Exception:  # noqa: BLE001
            self._add(CRITICAL, "§5.6", "encryption-nonce-malformed", rel,
                      "nonce is not valid base64")

        # AAD must be the canonical formula (we don't recompute the actual
        # SHA here; we just check the AAD field documents the formula)
        aad = block.get("aad")
        if aad != "sha256(memory_id||last_updated_at)":
            self._add(WARN, "§5.6", "encryption-aad-formula-drift", rel,
                      f"AAD formula `{aad}` differs from canonical "
                      f"`sha256(memory_id||last_updated_at)`")

    def _check_shamir_consistency(self) -> None:
        """If encryption_policy.enabled, shamir_fragments must be populated."""
        if not self.manifest:
            return
        pol = self.manifest.get("encryption_policy") or {}
        if not pol.get("enabled"):
            return
        sh = self.manifest.get("shamir_fragments") or {}
        fp = sh.get("master_key_fingerprint")
        frags = sh.get("fragments", [])
        if not fp:
            self._add(CRITICAL, "§5.6.3", "shamir-fingerprint-missing",
                      "manifest.json",
                      "encryption enabled but no master_key_fingerprint pinned")
        threshold = sh.get("threshold", 3)
        total = sh.get("total", 5)
        distributed = [f for f in frags if f.get("distributed_at")]
        if len(distributed) < total:
            self._add(WARN, "§5.6.3", "shamir-incomplete-distribution",
                      "manifest.json",
                      f"only {len(distributed)}/{total} fragments confirmed "
                      f"distributed; threshold for recovery is {threshold}")

    # -- reconciliation checkpoint (Stage 1 §8.7 phase 4 amendment) ---------

    def _check_reconciliation_checkpoint(self) -> None:
        """If `manifest.reconciliation_checkpoint` is set, confirm
        `audit_id` resolves to a row in the ledger AND `chain` matches that
        row's `chain`. Mismatch → CRITICAL stale-checkpoint per §8.7 phase 4.
        """
        if not self.manifest:
            return
        cp = self.manifest.get("reconciliation_checkpoint")
        if not cp or not isinstance(cp, dict):
            return  # not set or null — Stage 1 default
        cp_id = cp.get("audit_id")
        cp_chain = cp.get("chain")
        if not cp_id or not cp_chain:
            return  # null/empty fields are allowed (means "not yet set")

        # Walk audit rows we already collected to find matching audit_id
        for row in self.audit_rows:
            if row.get("audit_id") == cp_id:
                if row.get("chain") != cp_chain:
                    self._add(CRITICAL, "§8.7", "stale-checkpoint",
                              "manifest.json",
                              f"reconciliation_checkpoint.audit_id "
                              f"{cp_id[:24]}... resolves but chain mismatches "
                              f"(checkpoint={cp_chain[:23]}... "
                              f"vs ledger={row.get('chain', '')[:23]}...)",
                              details={
                                  "checkpoint_audit_id": cp_id,
                                  "checkpoint_chain": cp_chain,
                                  "ledger_chain": row.get("chain"),
                              })
                return  # found, no further check
        # Not found
        self._add(CRITICAL, "§8.7", "stale-checkpoint",
                  "manifest.json",
                  f"reconciliation_checkpoint.audit_id {cp_id[:24]}... "
                  f"does not resolve in any ledger row",
                  details={"checkpoint_audit_id": cp_id})

    # -- tombstone consistency -----------------------------------------------

    def _check_tombstone_consistency(self) -> None:
        for rel, fm in self.memory_files.items():
            if fm.get("tombstoned"):
                for f in ("deleted_at", "deleted_by", "tombstone_reason"):
                    if f not in fm:
                        self._add(WARN, "§4.6",
                                  "tombstone-missing-metadata", rel,
                                  f"tombstoned: true but {f} absent")

    def _maybe_acquire_shared_lock(self):
        """Batch 11 / Aspect 5.7 — best-effort .lock.shared during validation."""
        if os.environ.get("CYBEROS_NO_LOCK"):
            return
        try:
            import sys as _sys
            _sys.path.insert(0, str(Path(__file__).parent))
            from cyberos_lock import shared_lock
            self._lock_cm = shared_lock(self.root.parent, timeout=5.0)
            self._lock_held = self._lock_cm.__enter__()
        except Exception:
            self._lock_held = False
            self._lock_cm = None

    # -- helpers -------------------------------------------------------------

    def _add(self, severity: str, section: str, code: str, path: str,
             message: str, *, details: dict[str, Any] | None = None) -> None:
        self.findings.append(Finding(severity, section, code, path, message,
                                     details or {}))

    def _exit_code(self) -> int:
        worst = INFO
        for f in self.findings:
            if _SEVERITY_ORDER[f.severity] > _SEVERITY_ORDER[worst]:
                worst = f.severity
        return {INFO: 0, WARN: 1, CRITICAL: 2}[worst]


# ---------------------------------------------------------------------------
# Reporters
# ---------------------------------------------------------------------------

def report_text(findings: list[Finding], root: Path, *, quiet: bool = False) -> None:
    counts = {CRITICAL: 0, WARN: 0, INFO: 0}
    for f in findings:
        counts[f.severity] += 1

    print(f"cyberos-validate {root}")
    print(f"  CRITICAL: {counts[CRITICAL]}")
    print(f"  WARN:     {counts[WARN]}")
    print(f"  INFO:     {counts[INFO]}")
    print()

    severities = [CRITICAL] if quiet else [CRITICAL, WARN, INFO]
    for sev in severities:
        rows = [f for f in findings if f.severity == sev]
        if not rows:
            continue
        print(f"=== {sev} ({len(rows)}) ===")
        for f in rows:
            print(f"  [{f.section}] {f.code}: {f.path}")
            print(f"    {f.message}")
        print()

    if counts[CRITICAL] == 0 and counts[WARN] == 0:
        print("✅ no findings; store appears healthy.")


def report_json(findings: list[Finding], root: Path) -> None:
    output = {
        "tool": "cyberos-validate",
        "version": "0.1.0",
        "root": str(root),
        "findings": [f.to_dict() for f in findings],
    }
    print(json.dumps(output, indent=2, sort_keys=True))


def report_sarif(findings: list[Finding], root: Path) -> None:
    """SARIF 2.1.0 (https://docs.oasis-open.org/sarif/sarif/v2.1.0/)."""
    sev_map = {CRITICAL: "error", WARN: "warning", INFO: "note"}
    rules = sorted({(f.code, f.section) for f in findings})
    sarif = {
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "cyberos-validate",
                    "version": "0.1.0",
                    "informationUri": "https://cyberskill.world",
                    "rules": [
                        {
                            "id": code,
                            "shortDescription": {"text": f"{section} {code}"},
                        } for code, section in rules
                    ],
                }
            },
            "results": [
                {
                    "ruleId": f.code,
                    "level": sev_map[f.severity],
                    "message": {"text": f.message},
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": f.path,
                                "uriBaseId": "STORE_ROOT",
                            }
                        }
                    }],
                } for f in findings
            ],
        }],
    }
    print(json.dumps(sarif, indent=2, sort_keys=True))


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="cyberos-validate",
        description="Independent validator for .cyberos-memory/ stores.",
    )
    parser.add_argument(
        "path", nargs="?",
        help="Path to a .cyberos-memory/ directory (or a project root).",
    )
    parser.add_argument(
        "--format", choices=["text", "json", "sarif"], default="text",
        help="Output format (default: text)",
    )
    parser.add_argument(
        "--quiet", action="store_true",
        help="Only print CRITICAL findings (text format only)",
    )
    parser.add_argument(
        "--pre-commit", action="store_true",
        help="Pre-commit-hook-friendly mode: silent on success, "
             "exit fast on any CRITICAL, exit 0 otherwise",
    )
    parser.add_argument(
        "--self-test", action="store_true",
        help="Run against tests/vectors/ and exit",
    )
    args = parser.parse_args(argv)

    if args.self_test:
        return run_self_test()

    if not args.path:
        parser.error("path is required (unless --self-test)")

    root = Path(args.path).resolve()
    # If user pointed at project root, descend to .cyberos-memory/
    if (root / ".cyberos-memory").is_dir():
        root = root / ".cyberos-memory"

    v = Validator(root)
    code = v.run()

    if args.pre_commit:
        # Silent on success; report only CRITICAL; exit 0 if no CRITICAL,
        # 2 if any CRITICAL. Suppresses INFO/WARN entirely.
        critical = [f for f in v.findings if f.severity == CRITICAL]
        if critical:
            print(f"✘ {len(critical)} CRITICAL findings — commit blocked:")
            for f in critical:
                print(f"  [{f.section}] {f.code}: {f.path}")
                print(f"    {f.message}")
            return 2
        return 0

    if args.format == "text":
        report_text(v.findings, root, quiet=args.quiet)
    elif args.format == "json":
        report_json(v.findings, root)
    elif args.format == "sarif":
        report_sarif(v.findings, root)

    return code


def run_self_test() -> int:
    """Run validator against tests/vectors/ fixtures."""
    here = Path(__file__).parent
    vectors = here / "tests" / "vectors"
    if not vectors.exists():
        print(f"no test vectors at {vectors}")
        return 3
    failures = 0
    for fixture in sorted(vectors.iterdir()):
        if not fixture.is_dir():
            continue
        store = fixture / ".cyberos-memory"
        if not store.exists():
            continue
        expected_path = fixture / "expected.json"
        if not expected_path.exists():
            continue
        v = Validator(store)
        v.run()
        expected = json.loads(expected_path.read_text(encoding="utf-8"))
        actual_codes = sorted(f.code for f in v.findings
                              if f.severity == CRITICAL)
        expected_codes = sorted(expected.get("expected_critical_codes", []))
        if actual_codes == expected_codes:
            print(f"✅ {fixture.name}")
        else:
            print(f"❌ {fixture.name}")
            print(f"   expected CRITICAL: {expected_codes}")
            print(f"   got CRITICAL:      {actual_codes}")
            failures += 1
    return 0 if failures == 0 else 2


if __name__ == "__main__":
    sys.exit(main())
