#!/usr/bin/env python3
"""code_audit_validator — deterministic conformance checker for AUDIT.md outputs.

(Repo-side CLI: `python3 core/evals/validate.py`, a thin shim over this module —
kept so the PyPI install claims `code_audit_validator`, not the generic
top-level name `validate`.)

Checks a run's docs/ directory (BACKLOG.md + HANDOFF.md) against the
machine-checkable subset of AUDIT.md's core rules:

  R1-NO-OUTPUT       measured baseline/metric without a verbatim fenced output block
  R1-UNLINKED-OUTPUT measured row whose verify command appears in no fenced block
  R1-NO-REASON       UNMEASURED / NOT-APPLICABLE without a (reason)
  R1-GUI-TOOL        GUI tool (profiler/inspector/devtools) used as a Verify command
  R2-UNCITED         target is neither cited-with-URL nor labeled INTERNAL / N-A
  R2-NONNUMERIC      banned non-numeric target word (Minimal / High / Strict / ...)
  R3-PROTECTED       DONE task references a path under protected_areas
  R5-BAD-STATUS      task status outside { OPEN, IN-PROGRESS, DONE, BLOCKED }
  R5-BAD-SEV         severity outside { Critical, High, Medium, Low }
  R5-BAD-ID          task ID not matching L<loop>-T<n>
  R6-NO-ROOTCAUSE    BLOCKED task without a "Root cause:" note
  R8-SECRET          unredacted credential pattern in any output file
  P5-NO-STOP-REASON  HANDOFF.md does not cite which stop condition fired
  TARGET-HEALTH-UNVERIFIED  completed HANDOFF (v1.5.0+) lacks a `Target health:
                     PASS|FAIL` line — the target's RUN_COMMANDS were not verified
                     to still pass after the audit's changes (Phase 5 gate)
  HANDOFF-BAD-MSTATUS  metrics Status outside the closed metric-status set
  GATED-UNAPPROVED-EXEC  executed task not on the loop's `Approved:` line (gated mode)
  TEMPLATE-NONCONFORMANT BACKLOG.md does not follow the Phase 2 template, so the
                     rule tripwires above cannot see it (BLINDSPOTS BS-12)
  CONFIG-PLACEHOLDER a CONFIG value in the target's AUDIT.md still contains
                     unedited <placeholder> text (Phase 0 preflight)
  CONFIG-BAD-ENUM    MODE / DEPTH / BENCHMARK_MODE / SEVERITY_FLOOR outside its
                     allowed set (Phase 0 preflight)
  MALFORMED-FILE     artifact is not valid UTF-8 text or exceeds the size
                     ceiling — a verdict, never a traceback
  WAIVER-EXPIRED     a docs/AUDIT-WAIVERS.yaml entry matched a violation but is
                     expired/undated; the original violation stays active

A loop with zero findings is VALID (R7): absence of tasks is never a violation.
Waivers (docs/AUDIT-WAIVERS.yaml in the target repo) suppress matched
violations with an audit trail: code + reason + approved_by + expires (ISO
date, mandatory). Expired waivers un-suppress and are themselves flagged.

When the run directory (or its parent, if you point --run at docs/ itself)
contains the target's AUDIT.md, the CONFIG block is preflighted and
PROTECTED_AREAS is loaded from it automatically — `--protected` then extends
rather than replaces it (closes the double-entry gap, review item G-F).

Usage:
  python3 core/evals/validate.py --run <dir-containing-docs>  [--protected p1,p2] [--fail-on High]
  python3 core/evals/validate.py --run <dir> --report json    # structured findings export
  python3 core/evals/validate.py --run <dir> --report sarif   # GitHub code-scanning format
  python3 core/evals/validate.py --run <dir> --emit-feedback  # feedback@1 record skeleton
  python3 core/evals/validate.py --batch targets.yaml         # fleet runner -> reports + portfolio.json
  python3 core/evals/validate.py --aggregate r1.json r2.json  # portfolio roll-up of report JSONs
  python3 core/evals/validate.py --compare prev.json curr.json  # run-over-run regressions
  python3 core/evals/validate.py --all          # run every fixture, compare to expectations
  python3 core/evals/validate.py --all --json   # machine-readable results

A target may ship `audit-profile.yaml` at its root to EXTEND the GUI-tool and
secret-pattern denylists per stack (gui_tools list; secret_patterns name+regex
entries). `--fail-on` applies a severity policy to the exit code only — every
violation is always reported.

Exit codes: 0 = all good; 1 = violations / fixture mismatch; 2 = usage error.
"""

import argparse
import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
FIXTURES = HERE / "fixtures"

TASK_STATUSES = {"OPEN", "IN-PROGRESS", "DONE", "BLOCKED"}
SEVERITIES = {"Critical", "High", "Medium", "Low"}
METRIC_STATUSES = {"MEASURED", "UNMEASURED", "NOT-APPLICABLE"}
ID_RE = re.compile(r"^L\d+-T\d+$")

GUI_TOOLS = [
    "react profiler", "react devtools", "chrome devtools", "devtools",
    "browser inspector", "web inspector", "xcode instruments", "instruments.app",
    "lighthouse panel", "performance tab", "network tab", "profiler tab",
]

SECRET_PATTERNS = [
    ("aws-key", re.compile(r"\bAKIA[0-9A-Z]{16}\b")),
    ("github-token", re.compile(r"\bghp_[A-Za-z0-9]{36}\b")),
    ("gitlab-token", re.compile(r"\bglpat-[A-Za-z0-9_-]{20}\b")),
    ("stripe-key", re.compile(r"\bsk_live_[A-Za-z0-9]{16,}\b")),
    ("anthropic-key", re.compile(r"\bsk-ant-[A-Za-z0-9_-]{16,}\b")),
    ("openai-key", re.compile(r"\bsk-proj-[A-Za-z0-9_-]{20,}\b")),
    ("google-api-key", re.compile(r"\bAIza[0-9A-Za-z_-]{35}\b")),
    ("npm-token", re.compile(r"\bnpm_[A-Za-z0-9]{36}\b")),
    ("slack-token", re.compile(r"\bxox[bpoas]-[0-9A-Za-z-]{10,}\b")),
    ("private-key", re.compile(r"-----BEGIN (?:RSA |EC |OPENSSH |PGP )?PRIVATE KEY-----")),
    ("jwt", re.compile(r"\beyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\b")),
]

BANNED_TARGET_WORDS = {
    "minimal", "high", "strict", "low", "medium", "fast", "slow", "good",
    "great", "optimal", "best-in-class", "world-class", "enterprise-grade",
}

UNMEASURED_RE = re.compile(r"\b(UNMEASURED|NOT-APPLICABLE)\b")
REASONED_RE = re.compile(r"\b(?:UNMEASURED|NOT-APPLICABLE)\s*\([^)]+\)")
URL_RE = re.compile(r"https?://\S+")
STOP_RE = re.compile(r"Stop condition:\s*\(?[abc]\)?", re.IGNORECASE)
# Phase 5 target-health gate (v1.5.0): a completed run's HANDOFF must record that
# the target still passes its own RUN_COMMANDS after the audit's changes.
HEALTH_RE = re.compile(r"(?mi)Target health:\s*(PASS|FAIL)\b")
AUDIT_TITLE_VER_RE = re.compile(r"v(\d+)\.(\d+)\.(\d+)")


CELL_SPLIT_RE = re.compile(r"(?<!\\)\|")  # an escaped \| is cell CONTENT, not a separator


def norm(cell: str) -> str:
    """Strip markdown wrapping without mangling shell syntax: backticks go
    everywhere, but * and _ are stripped only at the edges (emphasis), so
    commands like `wc -l src/*.py` survive intact. Escaped pipes (\\|) are
    unescaped back to literal | so commands like `grep -cE "TODO\\|FIXME"`
    round-trip to what the agent actually ran (fixture G05)."""
    s = cell.strip().replace("`", "").replace("\\|", "|")
    return re.sub(r"^[*_]+|[*_]+$", "", s).strip()


def split_cells(line: str):
    return [norm(c) for c in CELL_SPLIT_RE.split(line.strip().strip("|"))]


def parse_tables(text: str):
    """Yield (header_cells, rows, end_line_idx) for every markdown table.

    Fence-aware (architect review F-1): R1 *requires* pasting raw tool output
    into ``` fences, and that output may itself contain GFM-table-shaped lines
    (markdown-emitting coverage/lint tools, `gh` CLI). Quoted lines inside a
    fence are raw evidence, not run artifacts — they must neither trip
    task/benchmark checks nor count toward template conformance.
    `section_fences` keeps its own independent fence walk."""
    lines = text.splitlines()
    i, in_fence = 0, False
    while i < len(lines):
        line = lines[i].strip()
        if line.startswith("```"):
            in_fence = not in_fence
            i += 1
            continue
        if in_fence:
            i += 1
            continue
        if line.startswith("|") and i + 1 < len(lines) and re.match(r"^\|[\s:|-]+\|?$", lines[i + 1].strip()):
            header = split_cells(line)
            rows, j = [], i + 2
            while j < len(lines) and lines[j].strip().startswith("|"):
                cells = split_cells(lines[j])
                if any(cells):
                    rows.append(cells)
                j += 1
            yield header, rows, j
            i = j
        else:
            i += 1


def section_fences(text: str, after_line: int):
    """Contents of fenced code blocks between after_line and the next heading."""
    lines = text.splitlines()
    fences, current, inside = [], [], False
    for line in lines[after_line:]:
        if not inside and re.match(r"^#{1,3}\s", line):
            break
        if line.strip().startswith("```"):
            if inside:
                fences.append("\n".join(current))
                current = []
            inside = not inside
            continue
        if inside:
            current.append(line)
    return fences


def col(header, *names):
    """Index of first header cell containing any of names (case-insensitive)."""
    low = [h.lower() for h in header]
    for n in names:
        for i, h in enumerate(low):
            if n in h:
                return i
    return None


def load_profile(target_root: Path, violations):
    """M-1 — stack profile packs. An optional `audit-profile.yaml` at the
    target root EXTENDS (never replaces) the built-in denylists, so
    stack-specific gaps become data calibrated per engagement, not validator
    code edits:

        gui_tools:
          - android studio profiler
        secret_patterns:
          - name: acme-internal-token
            regex: "\\\\bacme_[A-Za-z0-9]{20}\\\\b"

    Runner mode (the profile is the target's COMPLETE audit identity): an
    optional `config:` section carries the same KEY: value pairs as AUDIT.md's
    CONFIG block, so the protocol itself never has to be copied into the
    target — it stays single-source in the framework repo. Precedence: a full
    AUDIT.md copy at the target root wins over the profile's config: (copy
    mode = self-contained/pinned; runner mode = always-current protocol).

    Returns (gui_tools, secret_patterns, config). A bad regex is an artifact
    problem and yields MALFORMED-FILE (verdict, not traceback) for that entry."""
    gui = list(GUI_TOOLS)
    secrets = list(SECRET_PATTERNS)
    cfg = {}
    f = target_root / "audit-profile.yaml"
    if not f.exists():
        return gui, secrets, cfg
    section, cur = None, None
    for raw in f.read_text(encoding="utf-8", errors="replace").splitlines():
        line = raw.rstrip()
        s = line.strip()
        if not s or s.startswith("#"):
            continue
        if s == "gui_tools:":
            section, cur = "gui", None
            continue
        if s == "secret_patterns:":
            section, cur = "secrets", None
            continue
        if s == "config:":
            section, cur = "config", None
            continue
        if section == "config":
            m = CONFIG_KEY_RE.match(s)
            if m:
                key, raw_val = m.groups()
                cfg[key] = re.split(r"\s{2,}#", raw_val, 1)[0].strip()
            continue
        if section == "gui" and s.startswith("- "):
            gui.append(s[2:].strip().strip("\"'").lower())
        elif section == "secrets":
            if s.startswith("- "):
                cur = {}
                s = s[2:].strip()
            if cur is None or ":" not in s:
                continue
            k, _, v = s.partition(":")
            cur[k.strip()] = v.strip().strip("\"'")
            if "name" in cur and "regex" in cur and "_done" not in cur:
                try:
                    secrets.append((f"profile:{cur['name']}", re.compile(cur["regex"])))
                except re.error as e:
                    violations.append(("MALFORMED-FILE", "audit-profile.yaml",
                                       f"secret pattern '{cur['name']}' has an invalid regex: {e}"))
                cur["_done"] = True
    return gui, secrets, cfg


def check_benchmark_like_table(header, rows, end_idx, text, violations, src, is_handoff, gui_tools=None):
    gui_tools = gui_tools if gui_tools is not None else GUI_TOOLS
    mi = col(header, "metric")
    bi = col(header, "baseline")
    ti = col(header, "target")
    vi = col(header, "verify")
    si = col(header, "status")
    if mi is None or bi is None:
        return
    has_measured_row = False
    measured_rows = []  # (metric, verify) pairs claiming a measurement
    for r in rows:
        def cell(ix):
            return r[ix] if ix is not None and ix < len(r) else ""
        baseline, target, verify = cell(bi), cell(ti), cell(vi)
        # R1 — escape hatch must carry a reason
        for v in (baseline, cell(si) if is_handoff else ""):
            if UNMEASURED_RE.search(v) and not REASONED_RE.search(v):
                # In HANDOFF the reason may live in the Baseline/Final cells; only
                # flag the cell that itself claims UNMEASURED without any reason nearby.
                if not REASONED_RE.search(" ".join(r)):
                    violations.append((f"R1-NO-REASON", src, f"'{v}' lacks (reason): row {r[:2]}"))
        # Which rows claim a measurement?
        if is_handoff:
            status = cell(si)
            if status and status not in METRIC_STATUSES:
                violations.append(("HANDOFF-BAD-MSTATUS", src, f"Status '{status}' not in {sorted(METRIC_STATUSES)}"))
            if status == "MEASURED":
                has_measured_row = True
                measured_rows.append((cell(mi), verify))
        else:
            if baseline and not UNMEASURED_RE.search(baseline):
                has_measured_row = True
                measured_rows.append((cell(mi), verify))
        # R2 — honest targets
        if ti is not None and target:
            t = target.strip()
            t_low = t.lower().rstrip(".")
            if t_low in BANNED_TARGET_WORDS:
                violations.append(("R2-NONNUMERIC", src, f"banned non-numeric target '{t}'"))
            elif not (
                URL_RE.search(t)
                or "internal target" in t_low
                or "no external benchmark applicable" in t_low
                or t_low in {"n-a", "n/a", "—", "-"}
            ):
                violations.append(("R2-UNCITED", src, f"target '{t}' has no URL and no INTERNAL label"))
        # R1 — GUI tools are not verification
        if verify:
            v_low = verify.lower()
            for tool in gui_tools:
                if tool in v_low:
                    violations.append(("R1-GUI-TOOL", src, f"'{verify}' is a GUI tool, not a shell command"))
                    break
    # R1 — measured rows need verbatim output nearby…
    fences = section_fences(text, end_idx)
    if has_measured_row and not fences:
        violations.append(("R1-NO-OUTPUT", src, "table has MEASURED/measured rows but no fenced raw-output block before next heading"))
    # …and each measured row must be traceable to ITS verify command.
    # Whitespace-normalized containment (architect review F-6): a long command
    # re-wrapped across lines inside the fence is still the same command.
    elif fences:
        joined_ws = " ".join("\n".join(fences).split())
        for metric, verify in measured_rows:
            if verify and verify not in {"—", "-", ""} and " ".join(verify.split()) not in joined_ws:
                violations.append(("R1-UNLINKED-OUTPUT", src, f"measured metric '{metric}': verify command '{verify}' appears in no fenced output block"))


def check_task_table(header, rows, violations, src, protected):
    ii = col(header, "id")
    sev_i = col(header, "sev")
    st_i = col(header, "status")
    d_i = col(header, "description")
    if ii is None or st_i is None:
        return
    for r in rows:
        def cell(ix):
            return r[ix] if ix is not None and ix < len(r) else ""
        tid, sev, status, desc = cell(ii), cell(sev_i), cell(st_i), cell(d_i)
        if tid and not ID_RE.match(tid):
            violations.append(("R5-BAD-ID", src, f"task id '{tid}' != L<loop>-T<n>"))
        if status and status not in TASK_STATUSES:
            violations.append(("R5-BAD-STATUS", src, f"status '{status}' not in {sorted(TASK_STATUSES)}"))
        if sev and sev not in SEVERITIES:
            violations.append(("R5-BAD-SEV", src, f"severity '{sev}' not in {sorted(SEVERITIES)}"))
        if status == "BLOCKED" and "root cause" not in " ".join(r).lower():
            violations.append(("R6-NO-ROOTCAUSE", src, f"BLOCKED task '{tid}' has no 'Root cause:' note"))
        if status == "DONE" and protected:
            joined = " ".join(r).casefold()  # case-insensitive: src/Billing == src/billing (F-6)
            for p in protected:
                if p and p.casefold() in joined:
                    violations.append(("R3-PROTECTED", src, f"DONE task '{tid}' touches protected path '{p}'"))


APPROVED_RE = re.compile(r"^Approved:\s*(.+)$", re.MULTILINE)
# `Mode:` may open the Scope line or follow another field (`Protocol: … | Mode: …`
# since v1.3.0) — match at line start or after a `|` separator, never mid-prose.
MODE_GATED_RE = re.compile(r"(?mi)(?:^|\|)\s*-?\s*Mode:\s*gated\b")
EXECUTED_STATUSES = {"DONE", "IN-PROGRESS", "BLOCKED"}


def check_approvals(text, violations, src):
    """Gated-mode invariant: every executed task (DONE / IN-PROGRESS /
    BLOCKED) in a loop section must be listed on that section's `Approved:`
    line. The check fires when the line exists OR the section echoes
    `Mode: gated` (v1.1.0) — a gated loop with executed tasks and no
    `Approved:` line is a violation, not an exemption. Sections declaring
    neither are treated as autonomous and not checked."""
    sections = re.split(r"(?m)^## (?=Loop\b)", text)
    for sec in sections:
        m = APPROVED_RE.search(sec)
        gated = MODE_GATED_RE.search(sec) is not None
        if not m and not gated:
            continue
        if m:
            raw = m.group(1).strip()
            approved = set() if raw.lower() == "none" else {norm(x) for x in raw.split(",") if x.strip()}
        else:
            approved = set()
        for header, rows, _ in parse_tables(sec):
            ii, st_i = col(header, "id"), col(header, "status")
            if ii is None or st_i is None or col(header, "metric") is not None:
                continue
            for r in rows:
                tid = r[ii] if ii < len(r) else ""
                status = r[st_i] if st_i < len(r) else ""
                if status in EXECUTED_STATUSES and tid and tid not in approved:
                    why = ("not on this loop's Approved: line" if m
                           else "this gated loop has no Approved: line at all")
                    violations.append(("GATED-UNAPPROVED-EXEC", src,
                                       f"task '{tid}' is {status} but {why}"))


def check_secrets(text, violations, src, secret_patterns=None):
    for kind, pat in (secret_patterns if secret_patterns is not None else SECRET_PATTERNS):
        for m in pat.finditer(text):
            ctx = text[max(0, m.start() - 40):m.start()]
            if "[REDACTED:" in ctx + m.group(0):
                continue
            violations.append(("R8-SECRET", src, f"unredacted {kind} matching '{m.group(0)[:12]}…'"))


MODE_LINE_RE = re.compile(r"(?mi)(?:^|\|)\s*-?\s*Mode:\s*\S+")
PROTO_LINE_RE = re.compile(r"(?mi)(?:^|\|)\s*-?\s*Protocol:\s*v(\d+)\.(\d+)\.(\d+)\b")
NO_FINDINGS_RE = re.compile(r"No significant findings", re.IGNORECASE)

# The template requirements this validator enforces, keyed to the protocol
# release it ships with (kept in lockstep by check-docs-sync.py). Artifacts
# that echo an older `Protocol:` are judged by THAT version's template —
# version-aware validation, architect review F-5. Artifacts without the echo
# are assumed current (and, from v1.3.0 on, flagged for omitting it).
CURRENT_PROTOCOL = (1, 5, 0)
MODE_ECHO_SINCE = (1, 1, 0)
PROTO_ECHO_SINCE = (1, 3, 0)
TARGET_HEALTH_SINCE = (1, 5, 0)  # Phase 5 HANDOFF must record a Target health: PASS/FAIL line


def check_template_conformance(text, violations, src):
    """BLINDSPOTS BS-12 — the meta-tripwire. Every other check activates only
    when output LOOKS like the Phase 2 template (pipe tables, headings, Mode
    echo); a run that emits prose instead silently escapes all of them. This
    converts that silent escape into a violation, making the rest of the rule
    set load-bearing. Per loop section the template requires (since v1.3.0) a
    `Protocol:` echo, (since v1.1.0) a `Mode:` line, and — all versions —
    EITHER (benchmark table AND task table) OR the R7 "No significant
    findings" line. Requirements are gated on the section's stated protocol
    version, so older artifacts are judged by their own template (F-5)."""
    sections = re.split(r"(?m)^##\s+(?=Loop\b)", text)[1:]
    if not sections:
        violations.append(("TEMPLATE-NONCONFORMANT", src,
                           "no '## Loop <N>' section found — output does not follow the Phase 2 template"))
        return
    for sec in sections:
        loop_id = (sec.splitlines() or ["?"])[0].strip()
        pm = PROTO_LINE_RE.search(sec)
        proto = tuple(int(g) for g in pm.groups()) if pm else CURRENT_PROTOCOL
        if pm is None and CURRENT_PROTOCOL >= PROTO_ECHO_SINCE:
            violations.append(("TEMPLATE-NONCONFORMANT", src,
                               f"'{loop_id}': Scope & method has no 'Protocol:' echo (required since v1.3.0; "
                               f"artifacts from older protocol versions validate with the matching release tag)"))
        if proto >= MODE_ECHO_SINCE and not MODE_LINE_RE.search(sec):
            violations.append(("TEMPLATE-NONCONFORMANT", src,
                               f"'{loop_id}': Scope & method has no 'Mode:' line (required since v1.1.0)"))
        tables = list(parse_tables(sec))
        has_bench = any(col(h, "metric") is not None and col(h, "baseline") is not None for h, _, _ in tables)
        has_task = any(col(h, "id") is not None and col(h, "status") is not None
                       and col(h, "metric") is None for h, _, _ in tables)
        if not (has_bench and has_task) and not NO_FINDINGS_RE.search(sec):
            violations.append(("TEMPLATE-NONCONFORMANT", src,
                               f"'{loop_id}': missing benchmark and/or task table and no 'No significant findings' line"))


CONFIG_ENUMS = {
    "MODE": {"gated", "autonomous"},
    "DEPTH": {"quick", "standard", "deep"},
    "BENCHMARK_MODE": {"auto", "provided", "none"},
    "SEVERITY_FLOOR": {"Critical", "High", "Medium", "Low"},
}
# Architect review F-4: `<...>` alone misreads Java/TS generics (List<OrderDTO>)
# and shell redirection (< seed.txt > out.log) as placeholders. A placeholder is
# either the WHOLE value wrapped in <...>, or text carrying one of the canonical
# template stems below (the literal phrasings shipped in AUDIT.md's CONFIG).
TEMPLATE_STEMS = ("<e.g.", "<one line", "<paths/", "<how to", "<constraints", "<optional:")
CONFIG_KEY_RE = re.compile(r"^([A-Z][A-Z_]+):\s*(.*)$")


def is_placeholder(value: str) -> bool:
    v = value.strip()
    if len(v) > 2 and v.startswith("<") and v.endswith(">"):
        return True
    low = v.lower()
    return any(stem in low for stem in TEMPLATE_STEMS)


def parse_audit_config(audit_md: Path):
    """Flat KEY: value parse of the CONFIG block in a target repo's AUDIT.md.
    Comments are stripped only at >=2 spaces before '#' (the template's own
    column style) so values like 'ticket #4211' survive intact (F-4)."""
    cfg, in_config = {}, False
    for line in audit_md.read_text(encoding="utf-8", errors="replace").splitlines():
        if re.match(r"^##\s*CONFIG\b", line):
            in_config = True
            continue
        if in_config and re.match(r"^##\s", line):
            break
        if not in_config:
            continue
        m = CONFIG_KEY_RE.match(line.strip())
        if not m:
            continue
        key, raw = m.groups()
        cfg[key] = re.split(r"\s{2,}#", raw, 1)[0].strip()
    return cfg


def check_config_preflight(target_root: Path, violations, protected, profile_cfg=None):
    """Phase 0 CONFIG preflight (review gap G-D) + PROTECTED_AREAS auto-load
    (gap G-F). Config source precedence: the target's AUDIT.md copy if present
    (copy mode), else the audit-profile.yaml `config:` section (runner mode —
    the protocol stays in the framework repo). Placeholder values never
    silently configure anything. Enum values are compared on the first
    whitespace token, so an inline trailing comment can't fail the enum."""
    audit = target_root / "AUDIT.md"
    if audit.exists():
        cfg = parse_audit_config(audit)
        src = "AUDIT.md"
    elif profile_cfg:
        cfg = profile_cfg
        src = "audit-profile.yaml"
    else:
        return
    for key, val in cfg.items():
        if is_placeholder(val):
            violations.append(("CONFIG-PLACEHOLDER", src,
                               f"{key} still contains unedited template text: '{val[:60]}'"))
        elif key in CONFIG_ENUMS and val:
            token = val.split()[0]
            if token not in CONFIG_ENUMS[key]:
                violations.append(("CONFIG-BAD-ENUM", src,
                                   f"{key} '{token}' not in {sorted(CONFIG_ENUMS[key])}"))
    areas = cfg.get("PROTECTED_AREAS", "")
    if areas and not is_placeholder(areas):
        for p in areas.split(","):
            p = p.strip()
            if p and p not in protected:
                protected.append(p)


MAX_ARTIFACT_BYTES = 10 * 1024 * 1024  # 10 MB ceiling — a "report" beyond this is not a report


def read_artifact(path: Path, violations, src):
    """Guarded reader (architect review F-3): artifact problems must become
    VERDICTS, never tracebacks — a gate that crashes is neither pass nor fail
    and invites `|| true` workarounds. Returns text, or None after recording
    a MALFORMED-FILE violation."""
    try:
        if path.stat().st_size > MAX_ARTIFACT_BYTES:
            violations.append(("MALFORMED-FILE", src,
                               f"file is {path.stat().st_size} bytes — exceeds the {MAX_ARTIFACT_BYTES // (1024*1024)} MB artifact ceiling"))
            return None
        return path.read_text(encoding="utf-8")
    except UnicodeDecodeError as e:
        violations.append(("MALFORMED-FILE", src,
                           f"not valid UTF-8 (decode error at byte {e.start}) — artifacts must be UTF-8 text"))
        return None
    except OSError as e:
        violations.append(("MALFORMED-FILE", src, f"unreadable: {e.__class__.__name__}"))
        return None


def load_waivers(docs: Path):
    """docs/AUDIT-WAIVERS.yaml in the TARGET repo — audit-trailed, expiring
    suppressions (architect review §3.1). Deliberately different from eval
    fixtures (which may never be weakened): waivers live in the audited repo,
    name an approver, and MUST expire. Minimal YAML subset, stdlib-only:

        - code: R2-UNCITED            # required: violation code to waive
          file: BACKLOG.md            # optional: artifact filename
          match: "Palantir"           # optional: substring of the detail
          reason: "approved comparison for marketing deck"
          approved_by: "name@company"
          expires: 2026-09-01         # required: ISO date
    """
    f = docs / "AUDIT-WAIVERS.yaml"
    entries, cur = [], None
    if not f.exists():
        return entries
    for raw in f.read_text(encoding="utf-8", errors="replace").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("- "):
            cur = {}
            entries.append(cur)
            line = line[2:].strip()
        if cur is None or ":" not in line:
            continue
        k, _, v = line.partition(":")
        cur[k.strip()] = v.split(" #")[0].strip().strip("\"'")
    return entries


def apply_waivers(docs: Path, violations, waived_out=None):
    """Partition violations into active vs waived. An expired (or undated)
    waiver does NOT suppress — the original violation stays active and the
    waiver itself becomes a WAIVER-EXPIRED violation. WAIVER-EXPIRED is not
    itself waivable."""
    import datetime
    waivers = load_waivers(docs)
    if not waivers:
        return violations
    today = datetime.date.today()
    active, flagged = [], set()
    for code, src, detail in violations:
        match = None
        for i, w in enumerate(waivers):
            if w.get("code") != code:
                continue
            if w.get("file") and w["file"] != src:
                continue
            if w.get("match") and w["match"] not in detail:
                continue
            match = (i, w)
            break
        if match is None:
            active.append((code, src, detail))
            continue
        i, w = match
        try:
            valid = datetime.date.fromisoformat(w.get("expires", "")) >= today
        except ValueError:
            valid = False
        if valid:
            if waived_out is not None:
                waived_out.append({"code": code, "file": src, "detail": detail,
                                   "reason": w.get("reason", ""), "approved_by": w.get("approved_by", ""),
                                   "expires": w.get("expires", "")})
        else:
            active.append((code, src, detail))
            if i not in flagged:
                flagged.add(i)
                active.append(("WAIVER-EXPIRED", "AUDIT-WAIVERS.yaml",
                               f"waiver for {code} ('{w.get('reason', 'no reason')}') expired or has no valid 'expires:' date — renew it or fix the violation"))
    return active


def validate_run(run_dir: Path, protected=None, waived_out=None):
    """Validate one run directory (containing docs/BACKLOG.md, docs/HANDOFF.md)."""
    protected = list(protected or [])
    violations = []
    docs = run_dir / "docs" if (run_dir / "docs").is_dir() else run_dir
    # The target repo's root is docs/'s parent — whether --run was pointed at
    # the repo root or at docs/ itself. CONFIG preflight may extend `protected`;
    # an optional audit-profile.yaml extends the denylists (M-1).
    gui_tools, secret_patterns, profile_cfg = load_profile(docs.parent, violations)
    check_config_preflight(docs.parent, violations, protected, profile_cfg)
    backlog = docs / "BACKLOG.md"
    handoff = docs / "HANDOFF.md"
    # Run's protocol version (for version-gated checks): a target AUDIT.md copy
    # wins, else the BACKLOG's first `Protocol:` echo, else assume current.
    run_proto = CURRENT_PROTOCOL
    _audit_copy = docs.parent / "AUDIT.md"
    if _audit_copy.exists():
        _m = AUDIT_TITLE_VER_RE.search(_audit_copy.read_text(encoding="utf-8", errors="replace").splitlines()[0])
        if _m:
            run_proto = tuple(int(g) for g in _m.groups())
    if not backlog.exists():
        violations.append(("MISSING-FILE", "docs/BACKLOG.md", "file not found"))
    if backlog.exists():
        text = read_artifact(backlog, violations, "BACKLOG.md")
        if text is not None:
            if not _audit_copy.exists():
                _pm = PROTO_LINE_RE.search(text)
                if _pm:
                    run_proto = tuple(int(g) for g in _pm.groups())
            check_template_conformance(text, violations, "BACKLOG.md")
            check_secrets(text, violations, "BACKLOG.md", secret_patterns)
            check_approvals(text, violations, "BACKLOG.md")
            for header, rows, end in parse_tables(text):
                # Architect review F-2: a metric table in the BACKLOG is ALWAYS
                # checked — column shape selects semantics, it never disables
                # the check (the `Final`-column escape hatch is closed).
                if col(header, "metric") is not None:
                    handoff_shaped = col(header, "final") is not None and col(header, "status") is not None
                    check_benchmark_like_table(header, rows, end, text, violations, "BACKLOG.md", is_handoff=handoff_shaped, gui_tools=gui_tools)
                elif col(header, "status") is not None and col(header, "id") is not None:
                    check_task_table(header, rows, violations, "BACKLOG.md", protected)
    if handoff.exists():
        text = read_artifact(handoff, violations, "HANDOFF.md")
        if text is not None:
            check_secrets(text, violations, "HANDOFF.md", secret_patterns)
            if not STOP_RE.search(text):
                violations.append(("P5-NO-STOP-REASON", "HANDOFF.md", "no 'Stop condition: (a|b|c)' line"))
            # Target-health gate (v1.5.0): a HANDOFF that declares the run complete
            # (cites a stop condition) must prove the target still passes its own
            # RUN_COMMANDS — a `Target health: PASS|FAIL` line. Version-gated so
            # pre-v1.5.0 artifacts are judged by their own template.
            elif run_proto >= TARGET_HEALTH_SINCE and not HEALTH_RE.search(text):
                violations.append(("TARGET-HEALTH-UNVERIFIED", "HANDOFF.md",
                                   "completed run (stop condition cited) has no 'Target health: PASS|FAIL' line — "
                                   "RUN_COMMANDS were not verified after the audit's changes (Phase 5; run core/evals/verify-target.sh)"))
            for header, rows, end in parse_tables(text):
                if col(header, "metric") is not None:
                    check_benchmark_like_table(header, rows, end, text, violations, "HANDOFF.md", is_handoff=(col(header, "final") is not None), gui_tools=gui_tools)
                elif col(header, "status") is not None and col(header, "id") is not None:
                    check_task_table(header, rows, violations, "HANDOFF.md", protected)
    return apply_waivers(docs, violations, waived_out)


LOOP_HEAD_RE = re.compile(r"^Loop\s+(\d+)\s*(?:—|-)?\s*(.*)$")


def build_report(run_dir: Path, protected, violations):
    """Findings export (review gap G-G): one structured object per run, for
    roll-ups across projects, trend dashboards, and client-facing summaries.
    Parses the same artifacts the validator checks — no second source of truth."""
    import datetime
    docs = run_dir / "docs" if (run_dir / "docs").is_dir() else run_dir
    audit = docs.parent / "AUDIT.md"
    protocol_version = None
    protected = list(protected)
    _, _, _pcfg = load_profile(docs.parent, [])
    if audit.exists():
        m = re.search(r"v\d+\.\d+\.\d+", audit.read_text(encoding="utf-8").splitlines()[0])
        protocol_version = m.group(0) if m else None
    check_config_preflight(docs.parent, [], protected, _pcfg)  # mirror the auto-load; violations already counted
    report = {
        "schema": "code-audit-framework/report@1",
        "generated_at": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
        "run_dir": str(run_dir),
        "protocol_version": protocol_version,
        "protected_areas": protected,
        "loops": [],
        "metrics": [],
        "summary": {},
    }
    backlog = docs / "BACKLOG.md"
    if backlog.exists():
        text = backlog.read_text(encoding="utf-8")
        for sec in re.split(r"(?m)^##\s+(?=Loop\b)", text)[1:]:
            first = (sec.splitlines() or [""])[0]
            hm = LOOP_HEAD_RE.match(first.strip())
            mode_m = re.search(r"(?mi)(?:^|\|)\s*-?\s*Mode:\s*(\S+)", sec)
            proto_m = PROTO_LINE_RE.search(sec)
            appr_m = APPROVED_RE.search(sec)
            loop = {
                "loop": int(hm.group(1)) if hm else None,
                "date": (hm.group(2).strip() or None) if hm else None,
                "protocol": f"v{'.'.join(proto_m.groups())}" if proto_m else None,
                "mode": mode_m.group(1) if mode_m else None,
                "approved": ([] if appr_m.group(1).strip().lower() == "none"
                             else [norm(x) for x in appr_m.group(1).split(",") if x.strip()]) if appr_m else None,
                "no_significant_findings": bool(NO_FINDINGS_RE.search(sec)),
                "tasks": [],
            }
            for header, rows, _ in parse_tables(sec):
                ii, st_i = col(header, "id"), col(header, "status")
                if ii is None or st_i is None or col(header, "metric") is not None:
                    continue
                sev_i, vec_i, d_i, v_i = (col(header, "sev"), col(header, "vector"),
                                          col(header, "description"), col(header, "verify"))
                for r in rows:
                    def cell(ix):
                        return r[ix] if ix is not None and ix < len(r) else ""
                    if cell(ii):
                        loop["tasks"].append({
                            "id": cell(ii), "severity": cell(sev_i), "status": cell(st_i),
                            "vector": cell(vec_i), "description": cell(d_i), "verify": cell(v_i),
                        })
            report["loops"].append(loop)
    # Provenance in runner mode (no target AUDIT.md to read a version from):
    # fall back to the protocol the artifact itself declares in its Scope line.
    # Copy mode (AUDIT.md present) still wins. Surfaced by the first live
    # runner-mode field run, 2026-06-12.
    if report["protocol_version"] is None:
        for lp in report["loops"]:
            if lp.get("protocol"):
                report["protocol_version"] = lp["protocol"]
                break
    handoff = docs / "HANDOFF.md"
    if handoff.exists():
        text = handoff.read_text(encoding="utf-8")
        for header, rows, _ in parse_tables(text):
            mi, fi = col(header, "metric"), col(header, "final")
            if mi is None or fi is None:
                continue
            bi, di, ti, vi, si = (col(header, "baseline"), col(header, "delta"),
                                  col(header, "target"), col(header, "verify"), col(header, "status"))
            for r in rows:
                def cell(ix):
                    return r[ix] if ix is not None and ix < len(r) else ""
                if cell(mi):
                    entry = {
                        "metric": cell(mi), "baseline": cell(bi), "final": cell(fi),
                        "delta": cell(di), "target": cell(ti), "verify": cell(vi), "status": cell(si),
                    }
                    # Computed delta when both ends parse as numbers (review §2):
                    # the reported Delta cell is echoed, never trusted as math.
                    num = lambda s: (re.search(r"-?\d+(?:\.\d+)?", s) or [None]) and re.search(r"-?\d+(?:\.\d+)?", s)  # noqa: E731
                    b_m, f_m = num(entry["baseline"]), num(entry["final"])
                    if b_m and f_m:
                        entry["delta_computed"] = round(float(f_m.group(0)) - float(b_m.group(0)), 6)
                    report["metrics"].append(entry)
    tasks = [t for l in report["loops"] for t in l["tasks"]]
    by = lambda key: {k: sum(1 for t in tasks if t[key] == k)  # noqa: E731
                      for k in sorted({t[key] for t in tasks if t[key]})}
    vio_codes = {}
    for c, _, _ in violations:
        vio_codes[c] = vio_codes.get(c, 0) + 1
    report["summary"] = {
        "loops": len(report["loops"]),
        "tasks": len(tasks),
        "tasks_by_status": by("status"),
        "tasks_by_severity": by("severity"),
        "metrics_measured": sum(1 for m in report["metrics"] if m["status"] == "MEASURED"),
        "violations": len(violations),
        "violations_by_code": dict(sorted(vio_codes.items())),
        "clean": not violations,
    }
    report["violations"] = [{"code": c, "file": s, "detail": d} for c, s, d in violations]
    return report


def to_sarif(report):
    """Minimal SARIF 2.1.0 for GitHub code scanning (review F5, optional output)."""
    rules, results = {}, []
    for v in report["violations"]:
        rules.setdefault(v["code"], {"id": v["code"], "name": v["code"],
                                     "shortDescription": {"text": v["code"]}})
        results.append({
            "ruleId": v["code"],
            "level": "error",
            "message": {"text": v["detail"]},
            "locations": [{"physicalLocation": {"artifactLocation": {"uri": f"docs/{v['file']}"
                          if not v["file"].endswith("AUDIT.md") else v["file"]}}}],
        })
    return {
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {"driver": {
                "name": "code-audit-framework",
                "informationUri": "https://github.com/cyberskill-official/code-audit-framework",
                "version": (report.get("protocol_version") or "unknown").lstrip("v"),
                "rules": list(rules.values()),
            }},
            "results": results,
        }],
    }


def load_fixture_meta(fdir: Path):
    """fixture.yaml is intentionally flat key: value (no YAML dependency)."""
    meta = {"id": fdir.name, "expect": "pass", "expected_violations": [], "protected_areas": [], "exercises_rules": []}
    f = fdir / "fixture.yaml"
    if f.exists():
        for line in f.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if ":" not in line or line.startswith("#"):
                continue
            k, _, v = line.partition(":")
            k, v = k.strip(), v.strip()
            if k in ("expected_violations", "protected_areas", "exercises_rules"):
                meta[k] = [x.strip() for x in v.strip("[]").split(",") if x.strip()]
            elif k in ("id", "expect", "description"):
                meta[k] = v
    if meta["expect"] not in ("pass", "fail"):  # F-6: a typo must not silently change semantics
        raise SystemExit(f"fixture {fdir.name}: expect '{meta['expect']}' must be 'pass' or 'fail'")
    return meta


def check_registry(fixture_dirs):
    """Registry drift guard (improve/BLINDSPOTS.md BS-10): rules.json must
    agree with fixtures/ on disk, in both directions. A rule referencing a
    missing fixture means its coverage silently died; an unregistered fixture
    means coverage the registry does not own."""
    problems = []
    reg = HERE / "rules.json"
    if not reg.exists():
        return ["rules.json missing — registry is mandatory"]
    try:
        data = json.loads(reg.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        return [f"rules.json unparseable: {e}"]
    on_disk = {d.name for d in fixture_dirs}
    referenced = set()
    for rule in data.get("rules", []):
        for fx in rule.get("fixtures_exercising", []):
            referenced.add(fx)
            if fx not in on_disk:
                problems.append(f"rules.json: {rule.get('rule_id', '?')} references missing fixture '{fx}'")
    for d in sorted(on_disk - referenced):
        problems.append(f"fixture '{d}' exists on disk but is registered under no rule in rules.json")
    return problems


def run_all(as_json=False):
    results, ok = [], True
    fixtures = sorted(d for d in FIXTURES.iterdir() if d.is_dir())
    registry_problems = check_registry(fixtures)
    ok &= not registry_problems
    for fdir in fixtures:
        meta = load_fixture_meta(fdir)
        violations = validate_run(fdir, protected=meta["protected_areas"])
        codes = sorted({c for c, _, _ in violations})
        expected = sorted(set(meta["expected_violations"]))
        if meta["expect"] == "pass":
            fixture_ok = not violations
            verdict_note = "clean" if fixture_ok else f"unexpected: {codes}"
        else:  # expect: fail — the validator MUST catch exactly the planted faults
            fixture_ok = codes == expected
            verdict_note = "trapped as expected" if fixture_ok else f"expected {expected}, got {codes}"
        ok &= fixture_ok
        results.append({
            "fixture": meta["id"], "expect": meta["expect"],
            "violations": [f"{c} [{s}] {d}" for c, s, d in violations],
            "codes": codes, "ok": fixture_ok, "note": verdict_note,
        })
    summary = {"fixtures": len(results), "passed": sum(r["ok"] for r in results), "all_ok": ok,
               "registry_problems": registry_problems, "results": results}
    if as_json:
        print(json.dumps(summary, indent=2))
    else:
        for p in registry_problems:
            print(f"[FAIL] REGISTRY DRIFT — {p}")
        for r in results:
            print(f"[{'PASS' if r['ok'] else 'FAIL'}] {r['fixture']:32s} expect={r['expect']:4s} → {r['note']}")
        print(f"\n{summary['passed']}/{summary['fixtures']} fixtures OK — "
              + ("ALL GREEN" if ok else "REGRESSIONS PRESENT"))
    return 0 if ok else 1


# M-2 — severity-policy gate. Violation codes carry fixed severities so client
# CI can ratchet strictness (--fail-on) instead of facing a binary red wall.
# The DEFAULT remains "any violation fails": the policy gate is opt-in.
VIOLATION_SEVERITY = {
    "R8-SECRET": "Critical", "R3-PROTECTED": "Critical", "GATED-UNAPPROVED-EXEC": "Critical",
    "R1-NO-OUTPUT": "High", "R1-UNLINKED-OUTPUT": "High", "R1-NO-REASON": "High",
    "R1-GUI-TOOL": "High", "R2-UNCITED": "High", "R2-NONNUMERIC": "High",
    "TEMPLATE-NONCONFORMANT": "High", "CONFIG-PLACEHOLDER": "High", "CONFIG-BAD-ENUM": "High",
    "MALFORMED-FILE": "High", "WAIVER-EXPIRED": "High", "MISSING-FILE": "High",
    "TARGET-HEALTH-UNVERIFIED": "High",
    "R5-BAD-STATUS": "Medium", "R5-BAD-SEV": "Medium", "R5-BAD-ID": "Medium",
    "R6-NO-ROOTCAUSE": "Medium", "P5-NO-STOP-REASON": "Medium", "HANDOFF-BAD-MSTATUS": "Medium",
}
SEVERITY_RANK = {"Critical": 3, "High": 2, "Medium": 1, "any": 0}


def gate_violations(violations, fail_on):
    """Return the subset of violations at or above the fail threshold.
    Unknown codes (e.g. future additions) conservatively count as High."""
    if fail_on == "any":
        return violations
    floor = SEVERITY_RANK[fail_on]
    return [v for v in violations
            if SEVERITY_RANK.get(VIOLATION_SEVERITY.get(v[0], "High"), 2) >= floor]


def compare_reports(prev_path, curr_path):
    """M-5 — run-over-run comparison: regressions between two report@1 files.
    Informational (continuity for multi-loop engagements); the exit gate stays
    --run's job."""
    prev = json.loads(Path(prev_path).read_text(encoding="utf-8"))
    curr = json.loads(Path(curr_path).read_text(encoding="utf-8"))
    p_tasks = {t["id"]: t for l in prev.get("loops", []) for t in l.get("tasks", [])}
    c_tasks = {t["id"]: t for l in curr.get("loops", []) for t in l.get("tasks", [])}
    reopened = [tid for tid, t in c_tasks.items()
                if p_tasks.get(tid, {}).get("status") == "DONE" and t.get("status") != "DONE"]
    pv = prev.get("summary", {}).get("violations_by_code", {})
    cv = curr.get("summary", {}).get("violations_by_code", {})
    new_codes = {c: n for c, n in cv.items() if n > pv.get(c, 0)}
    cleared = {c: n for c, n in pv.items() if cv.get(c, 0) < n}
    p_metrics = {m["metric"]: m for m in prev.get("metrics", [])}
    metric_changes = []
    for m in curr.get("metrics", []):
        pm = p_metrics.get(m["metric"])
        if pm and "delta_computed" in m and "delta_computed" in pm and m["delta_computed"] != pm["delta_computed"]:
            metric_changes.append({"metric": m["metric"],
                                   "prev_delta": pm["delta_computed"], "curr_delta": m["delta_computed"]})
    return {
        "schema": "code-audit-framework/comparison@1",
        "prev": prev.get("run_dir"), "curr": curr.get("run_dir"),
        "clean_transition": f"{prev.get('summary', {}).get('clean')} -> {curr.get('summary', {}).get('clean')}",
        "reopened_tasks": reopened,
        "violations_increased": new_codes,
        "violations_cleared": cleared,
        "metric_delta_changes": metric_changes,
    }


def emit_feedback(run_dir: Path, report, waived, run_id=None):
    """M-7 — feedback-record skeleton (schemas/feedback.v1.json): the machine
    half filled from the run, the human adjudication half left empty. One run
    -> one record; records are the calibration substrate (TESTING-PROTOCOL)."""
    import datetime
    rid = run_id or f"{datetime.date.today().isoformat()}-{run_dir.resolve().name}"
    s = report.get("summary", {})
    lines = [
        "# feedback@1 — fill the adjudication fields, then file in the field-data repo",
        "# (sanitize: client -> code name, no code excerpts, R8 applies here too)",
        f"run_id: {rid}",
        f"protocol_version: {report.get('protocol_version') or 'null  # no AUDIT.md found at target root'}",
        f"validator_version: {'.'.join(str(x) for x in CURRENT_PROTOCOL)}",
        "agent: { cli: FILLME, model: FILLME }",
        "config: { mode: FILLME, depth: FILLME, loop_budget: FILLME, stack: [FILLME] }",
        f"report_ref: reports/{rid}.json",
        "retro_score: FILLME            # RETROSPECTIVE.md total /20",
        "retro_items: {}                # only items scored < 2, with a one-line note",
        "validator_false_positives: []  # each is a G-fixture candidate: {code, detail_excerpt, why_wrong}",
        "validator_misses: []           # each is a B-fixture + FAILURE_LOG candidate: {description, candidate_code}",
        "denylist_gaps: []              # {list: GUI_TOOLS|SECRET_PATTERNS, value: ...} -> audit-profile/pattern-pack",
        "fabrication_check: { sampled: 0, mismatched: 0 }   # TESTING-PROTOCOL tier 3",
        "cross_model: null              # tier 4 runs only",
        f"waivers: {{ active: {len(waived)}, expired_flagged: {s.get('violations_by_code', {}).get('WAIVER-EXPIRED', 0)} }}",
        f"violations_total: {s.get('violations', 0)}",
        "narrative: >",
        "  FILLME — one paragraph: what surprised, what the client said, what the numbers miss.",
    ]
    return "\n".join(lines) + "\n"


def run_batch(targets_file, out_dir, fail_on):
    """M-3 — fleet runner: validate every target listed in a YAML-lite file
    (`- path: ...` with optional `name:`/`protected:`), write one report@1
    each into out_dir, then aggregate. Exit 1 if any run fails the policy."""
    entries, cur = [], None
    for raw in Path(targets_file).read_text(encoding="utf-8").splitlines():
        s = raw.strip()
        if not s or s.startswith("#"):
            continue
        if s.startswith("- "):
            cur = {}
            entries.append(cur)
            s = s[2:].strip()
        if cur is None or ":" not in s:
            continue
        k, _, v = s.partition(":")
        cur[k.strip()] = v.strip().strip("\"'")
    out = Path(out_dir)
    out.mkdir(parents=True, exist_ok=True)
    paths, any_fail = [], False
    for e in entries:
        target = Path(e.get("path", ""))
        name = e.get("name") or target.resolve().name
        protected = [p for p in e.get("protected", "").split(",") if p]
        if not target.exists():
            print(f"[FAIL] {name}: path does not exist: {target}")
            any_fail = True
            continue
        waived = []
        v = validate_run(target, protected=protected, waived_out=waived)
        gated = gate_violations(v, fail_on)
        report = build_report(target, protected, v)
        report["waived"] = waived
        report["summary"]["waived"] = len(waived)
        rp = out / f"{name}.json"
        rp.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        paths.append(str(rp))
        status = "PASS" if not gated else "FAIL"
        any_fail |= bool(gated)
        print(f"[{status}] {name}: {len(v)} violation(s), {len(waived)} waived → {rp}")
    if paths:
        agg = aggregate_reports(paths)
        (out / "portfolio.json").write_text(json.dumps(agg, indent=2) + "\n", encoding="utf-8")
        t = agg["totals"]
        print(f"\nportfolio: {t['clean_runs']}/{t['runs']} clean, {t['violations']} active, "
              f"{t['waived']} waived → {out / 'portfolio.json'}")
    return 1 if any_fail else 0


def aggregate_reports(paths):
    """Portfolio roll-up over per-run report JSONs (architect review §3.2)."""
    import datetime
    runs, by_code, by_sev = [], {}, {}
    for p in paths:
        r = json.loads(Path(p).read_text(encoding="utf-8"))
        s = r.get("summary", {})
        runs.append({
            "run_dir": r.get("run_dir"), "protocol_version": r.get("protocol_version"),
            "clean": s.get("clean"), "violations": s.get("violations", 0),
            "waived": len(r.get("waived", [])), "tasks": s.get("tasks", 0),
            "loops": s.get("loops", 0),
        })
        for code, n in s.get("violations_by_code", {}).items():
            by_code[code] = by_code.get(code, 0) + n
        for sev, n in s.get("tasks_by_severity", {}).items():
            by_sev[sev] = by_sev.get(sev, 0) + n
    return {
        "schema": "code-audit-framework/portfolio@1",
        "generated_at": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
        "runs": runs,
        "totals": {
            "runs": len(runs),
            "clean_runs": sum(1 for r in runs if r["clean"]),
            "violations": sum(r["violations"] for r in runs),
            "waived": sum(r["waived"] for r in runs),
            "violations_by_code": dict(sorted(by_code.items())),
            "tasks_by_severity": dict(sorted(by_sev.items())),
        },
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--run", help="validate one run directory (containing docs/)")
    ap.add_argument("--all", action="store_true", help="run the full fixture suite")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--report", choices=["json", "sarif"],
                    help="with --run: emit a structured findings report instead of plain violations")
    ap.add_argument("--aggregate", nargs="+", metavar="REPORT_JSON",
                    help="portfolio roll-up over per-run --report json files")
    ap.add_argument("--batch", metavar="TARGETS_YAML",
                    help="fleet runner: validate every listed target, write reports + portfolio.json")
    ap.add_argument("--batch-out", default="audit-reports", help="output directory for --batch (default: audit-reports)")
    ap.add_argument("--compare", nargs=2, metavar=("PREV_JSON", "CURR_JSON"),
                    help="run-over-run comparison of two report files (informational)")
    ap.add_argument("--emit-feedback", action="store_true",
                    help="with --run: print a feedback@1 record skeleton for this run")
    ap.add_argument("--run-id", default=None, help="with --emit-feedback: override the generated run id")
    ap.add_argument("--fail-on", choices=["any", "Critical", "High", "Medium"], default="any",
                    help="exit-code policy: fail only on violations at/above this severity (default: any)")
    ap.add_argument("--protected", default="", help="comma-separated protected paths (extends the target AUDIT.md's PROTECTED_AREAS)")
    args = ap.parse_args()
    if args.all:
        sys.exit(run_all(as_json=args.json))
    if args.batch:
        if not Path(args.batch).is_file():
            print(f"usage error: targets file not found: {args.batch}", file=sys.stderr)
            sys.exit(2)
        sys.exit(run_batch(args.batch, args.batch_out, args.fail_on))
    if args.compare:
        missing = [p for p in args.compare if not Path(p).is_file()]
        if missing:
            print(f"usage error: report file(s) not found: {', '.join(missing)}", file=sys.stderr)
            sys.exit(2)
        print(json.dumps(compare_reports(*args.compare), indent=2))
        sys.exit(0)
    if args.aggregate:
        missing = [p for p in args.aggregate if not Path(p).is_file()]
        if missing:
            print(f"usage error: report file(s) not found: {', '.join(missing)}", file=sys.stderr)
            sys.exit(2)
        agg = aggregate_reports(args.aggregate)
        if args.json:
            print(json.dumps(agg, indent=2))
        else:
            t = agg["totals"]
            print(f"{'Run':40s} {'proto':8s} {'clean':5s} {'viol':>4s} {'waived':>6s} {'tasks':>5s}")
            for r in agg["runs"]:
                print(f"{str(r['run_dir'])[:40]:40s} {str(r['protocol_version']):8s} "
                      f"{'yes' if r['clean'] else 'NO':5s} {r['violations']:4d} {r['waived']:6d} {r['tasks']:5d}")
            print(f"\n{t['clean_runs']}/{t['runs']} runs clean — {t['violations']} active violation(s), "
                  f"{t['waived']} waived — by code: {t['violations_by_code'] or '{}'}")
        sys.exit(0)
    if args.run:
        run_path = Path(args.run)
        if not run_path.exists():
            print(f"usage error: --run path does not exist: {run_path}", file=sys.stderr)
            sys.exit(2)
        protected = [p for p in args.protected.split(",") if p]
        waived = []
        v = validate_run(run_path, protected=protected, waived_out=waived)
        gated = gate_violations(v, args.fail_on)
        if args.emit_feedback:
            report = build_report(run_path, protected, v)
            report["waived"] = waived
            print(emit_feedback(run_path, report, waived, run_id=args.run_id))
        elif args.report:
            report = build_report(run_path, protected, v)
            report["waived"] = waived
            report["summary"]["waived"] = len(waived)
            print(json.dumps(to_sarif(report) if args.report == "sarif" else report, indent=2))
        elif args.json:
            print(json.dumps([{"code": c, "file": s, "detail": d} for c, s, d in v], indent=2))
        else:
            for c, s, d in v:
                sev = VIOLATION_SEVERITY.get(c, "High")
                print(f"VIOLATION {c} [{s}] ({sev}) {d}")
            for w in waived:
                print(f"WAIVED    {w['code']} [{w['file']}] until {w['expires']} — {w['reason']} (approved by {w['approved_by']})")
            tail = "CLEAN — no violations" if not v else f"{len(v)} violation(s)"
            if v and not gated:
                tail += f" — all below --fail-on {args.fail_on}, exit 0"
            print(tail + (f" ({len(waived)} waived)" if waived else ""))
        sys.exit(0 if not gated else 1)
    ap.print_help()
    sys.exit(2)


if __name__ == "__main__":
    main()
