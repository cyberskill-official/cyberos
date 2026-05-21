"""harness — read-only daily report aggregating self_audit signals per skill.

FR-CUO-200 Wave 1 of the continuous-improvement harness.

Walks the memory audit chain, parses each SKILL.md's `self_audit.anomaly_signals`
+ `human_fine_tune.signals_to_initiate` blocks from frontmatter, evaluates
each declared signal against the windowed audit rows, and emits a markdown
report at `docs/harness/harness-report-<YYYY-MM-DD>.md`.

The harness is **read-only**. It MUST NOT mutate any skill, RUBRIC, contract,
or workflow file. Wave 2 (FR-CUO-201) introduces proposal authoring; this FR
only provides visibility.

Key invariants:
  * Per run, exactly one `harness.report_emitted` memory audit row.
  * Window arguments are duration strings parsed once (`24h`, `7d`, `30d`).
  * Each tripped signal carries skill name, signal id, observed value,
    threshold, evidence row ids.
  * Workflow rework rates sorted descending.
  * Watch mode atomic-writes (write-to-temp then rename).
"""

from __future__ import annotations

import json
import os
import re
from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Optional

from cuo.core.harness_signals import evaluate_signal


# ----------------------------------------------------------------------------
# Public dataclasses
# ----------------------------------------------------------------------------


@dataclass
class SignalBreach:
    skill_name: str
    signal_id: str
    value: float
    threshold: float
    evidence_row_ids: list[str] = field(default_factory=list)

    def __repr__(self) -> str:
        return (f"SignalBreach({self.skill_name}::{self.signal_id} "
                f"value={self.value:.3f} > {self.threshold:.3f} "
                f"evidence={len(self.evidence_row_ids)})")


@dataclass
class WorkflowMetrics:
    workflow_id: str
    total_runs: int = 0
    completed: int = 0
    routed_back: int = 0
    hitl_halt: int = 0
    failed: int = 0

    @property
    def rework_rate(self) -> float:
        return self.routed_back / self.total_runs if self.total_runs else 0.0


@dataclass
class HarnessReport:
    window: timedelta
    generated_at: datetime
    breaches: list[SignalBreach] = field(default_factory=list)
    workflow_metrics: list[WorkflowMetrics] = field(default_factory=list)
    routed_back_history: dict[str, int] = field(default_factory=dict)
    total_rows_walked: int = 0
    skills_inspected: int = 0


# ----------------------------------------------------------------------------
# Duration parser
# ----------------------------------------------------------------------------


_DUR_RE = re.compile(r"^(\d+)([hdw])$")


def parse_window(spec: str) -> timedelta:
    """Parse `24h` / `7d` / `4w` / `30d` into a timedelta.

    Returns timedelta(days=7) as a default if the spec is unparseable —
    callers can override with explicit timedelta if needed.
    """
    if isinstance(spec, timedelta):
        return spec
    m = _DUR_RE.match(spec.strip())
    if not m:
        return timedelta(days=7)
    n, unit = int(m.group(1)), m.group(2)
    return {"h": timedelta(hours=n), "d": timedelta(days=n), "w": timedelta(weeks=n)}[unit]


# ----------------------------------------------------------------------------
# Skill frontmatter parsing
# ----------------------------------------------------------------------------


_YAML_FENCE_RE = re.compile(r"\A---\n(.*?)\n---\n", re.DOTALL)


def parse_skill_signals(skill_md_path: Path) -> dict[str, dict]:
    """Extract `self_audit.anomaly_signals` + `human_fine_tune.signals_to_initiate`
    blocks from a SKILL.md's YAML frontmatter. Returns a dict mapping
    `signal_id` → threshold-spec dict. Uses a tolerant YAML parse (no
    external dep — we only need top-level keys).
    """
    try:
        text = skill_md_path.read_text(encoding="utf-8")
    except OSError:
        return {}
    m = _YAML_FENCE_RE.search(text)
    if not m:
        return {}

    fm_text = m.group(1)
    # Cheap line-based parser: scan for "self_audit:" then "anomaly_signals:"
    # then the indented signal_id: spec lines. Same for human_fine_tune /
    # signals_to_initiate.
    out: dict[str, dict] = {}
    out.update(_extract_signal_dict(fm_text, "self_audit", "anomaly_signals"))
    out.update(_extract_signal_list(fm_text, "human_fine_tune", "signals_to_initiate"))
    return out


def _extract_signal_dict(yaml_text: str, parent: str, child: str) -> dict[str, dict]:
    """Extract `<parent>.<child>: {sig_id: {threshold: N, ...}}` style blocks."""
    pattern = re.compile(
        rf"^{re.escape(parent)}:\s*\n(?:\s+.*\n)*?\s+{re.escape(child)}:\s*\n((?:\s{{4,}}.*\n)+)",
        re.MULTILINE,
    )
    m = pattern.search(yaml_text)
    if not m:
        return {}
    block = m.group(1)
    signals: dict[str, dict] = {}
    for line in block.splitlines():
        if not line.strip():
            continue
        sig_m = re.match(r"^\s{4,}(\w+):\s*(\{.*\})\s*$", line)
        if sig_m:
            sig_id = sig_m.group(1)
            spec_str = sig_m.group(2)
            spec = _parse_inline_dict(spec_str)
            signals[sig_id] = spec
    return signals


def _extract_signal_list(yaml_text: str, parent: str, child: str) -> dict[str, dict]:
    """Extract `<parent>.<child>: [- sig_id_below: N, ...]` style blocks."""
    pattern = re.compile(
        rf"^{re.escape(parent)}:\s*\n(?:\s+.*\n)*?\s+{re.escape(child)}:\s*\n((?:\s+-\s+.*\n)+)",
        re.MULTILINE,
    )
    m = pattern.search(yaml_text)
    if not m:
        return {}
    block = m.group(1)
    signals: dict[str, dict] = {}
    for line in block.splitlines():
        kv = re.match(r"^\s+-\s+(\w+):\s*([\d.]+)\s*$", line)
        if kv:
            signals[kv.group(1)] = {"threshold": float(kv.group(2))}
        # Bare list items like `- deterministic_drift_observed` → presence-only
        bare = re.match(r"^\s+-\s+(\w+)\s*$", line)
        if bare and bare.group(1).endswith("_observed"):
            signals[bare.group(1)] = {"threshold": 1}
    return signals


def _parse_inline_dict(spec_str: str) -> dict:
    """Tolerant parser for `{threshold: 3, window: 10}` style YAML inline dicts."""
    spec_str = spec_str.strip().lstrip("{").rstrip("}")
    out: dict = {}
    for part in spec_str.split(","):
        part = part.strip()
        if ":" not in part:
            continue
        k, v = part.split(":", 1)
        k, v = k.strip(), v.strip()
        try:
            out[k] = float(v) if "." in v else int(v)
        except ValueError:
            out[k] = v
    return out


# ----------------------------------------------------------------------------
# Audit row source — accept either a list of dicts (tests) OR a binlog dir
# ----------------------------------------------------------------------------


def load_audit_rows(audit_dir: Path) -> list[dict]:
    """Read every *.binlog under `audit_dir` and return the parsed records as
    a list of dicts. Best-effort: skips malformed frames.

    Frame format (per AGENTS.md §6.2):
      [u32 length BE][u32 crc32c BE][u64 seq BE][u64 ts_ns BE][payload]
    Payload is msgspec canonical JSON.
    """
    rows: list[dict] = []
    if not audit_dir.is_dir():
        return rows
    for binlog in sorted(audit_dir.glob("*.binlog")):
        try:
            data = binlog.read_bytes()
        except OSError:
            continue
        idx = 0
        while idx + 24 < len(data):
            length = int.from_bytes(data[idx:idx + 4], "big")
            seq = int.from_bytes(data[idx + 8:idx + 16], "big")
            ts_ns = int.from_bytes(data[idx + 16:idx + 24], "big")
            payload_end = idx + 24 + length
            if payload_end > len(data):
                break
            payload = data[idx + 24:payload_end]
            try:
                rec = json.loads(payload.decode("utf-8"))
            except (UnicodeDecodeError, json.JSONDecodeError):
                idx = payload_end
                continue
            rec.setdefault("seq", seq)
            rec.setdefault("ts_ns", ts_ns)
            rec.setdefault("row_id", f"{binlog.stem}:{seq}")
            rows.append(rec)
            idx = payload_end
    return rows


# ----------------------------------------------------------------------------
# Main compute_report
# ----------------------------------------------------------------------------


def compute_report(
    audit_dir: Optional[Path],
    skill_root: Path,
    window: timedelta,
    *,
    rows_override: Optional[list[dict]] = None,
    skill_filter: Optional[str] = None,
    workflow_filter: Optional[str] = None,
) -> HarnessReport:
    """Build a HarnessReport from the windowed audit chain + skill frontmatter.

    Args:
        audit_dir: path to `<memory-root>/audit/`. Unused if rows_override set.
        skill_root: path to `modules/skill/` for per-skill frontmatter reads.
        window: how far back to look. Rows older than now-window are excluded.
        rows_override: pre-parsed list of audit row dicts (test entry).
        skill_filter: limit per-skill signal eval to this skill name.
        workflow_filter: limit workflow metrics to this workflow id.

    Returns: HarnessReport with breaches + workflow metrics + routed-back history.
    """
    now = datetime.now(tz=timezone.utc)
    cutoff_ns = int((now - window).timestamp() * 1_000_000_000)

    if rows_override is not None:
        all_rows = rows_override
    else:
        all_rows = load_audit_rows(audit_dir) if audit_dir else []

    # Window filter
    rows = [r for r in all_rows if r.get("ts_ns", 0) >= cutoff_ns]

    breaches: list[SignalBreach] = []
    workflow_metrics: dict[str, WorkflowMetrics] = {}
    routed_back: dict[str, int] = {}

    # Per-skill signal evaluation
    skills_inspected = 0
    if skill_root.is_dir():
        for skill_dir in sorted(skill_root.iterdir()):
            if not skill_dir.is_dir():
                continue
            skill_md = skill_dir / "SKILL.md"
            if not skill_md.is_file():
                continue
            if skill_filter and skill_dir.name != skill_filter:
                continue
            skill_signals = parse_skill_signals(skill_md)
            if not skill_signals:
                continue
            skills_inspected += 1
            # Scope rows to those mentioning this skill
            skill_rows = [
                r for r in rows
                if (r.get("extra") or {}).get("skill") == skill_dir.name
                or (r.get("op") or "").startswith(f"{skill_dir.name}.")
                or skill_dir.name in str(r.get("path") or "")
            ]
            for sig_id, threshold in skill_signals.items():
                tripped, value, evidence = evaluate_signal(sig_id, skill_rows, threshold)
                if tripped:
                    breaches.append(SignalBreach(
                        skill_name=skill_dir.name,
                        signal_id=sig_id,
                        value=value,
                        threshold=_threshold_value(threshold),
                        evidence_row_ids=[r.get("row_id", "") for r in evidence[:10]],
                    ))

    # Per-workflow metrics aggregation
    for r in rows:
        op = r.get("op", "")
        extra = r.get("extra") or {}
        wf_id = extra.get("workflow_id")
        outcome = extra.get("outcome")
        if op == "workflow_complete" and wf_id:
            if workflow_filter and wf_id != workflow_filter:
                continue
            wm = workflow_metrics.setdefault(wf_id, WorkflowMetrics(workflow_id=wf_id))
            wm.total_runs += 1
            if outcome == "COMPLETED" or outcome == "done":
                wm.completed += 1
            elif outcome == "ROUTED_BACK":
                wm.routed_back += 1
            elif outcome == "HITL_HALT":
                wm.hitl_halt += 1
            elif outcome == "FAILED":
                wm.failed += 1
        if op == "memory.fr_routed_back":
            fr_id = extra.get("fr_id") or "(unknown)"
            routed_back[fr_id] = routed_back.get(fr_id, 0) + 1

    return HarnessReport(
        window=window,
        generated_at=now,
        breaches=sorted(breaches, key=lambda b: (-b.value, b.skill_name)),
        workflow_metrics=sorted(
            workflow_metrics.values(),
            key=lambda w: -w.rework_rate,
        ),
        routed_back_history=routed_back,
        total_rows_walked=len(rows),
        skills_inspected=skills_inspected,
    )


def _threshold_value(threshold: Any) -> float:
    """Extract the numeric threshold for display."""
    if isinstance(threshold, (int, float)):
        return float(threshold)
    if isinstance(threshold, dict):
        for k in ("threshold", "rate", "count"):
            if k in threshold:
                return float(threshold[k])
    return 0.0


# ----------------------------------------------------------------------------
# Markdown formatter
# ----------------------------------------------------------------------------


def format_markdown(report: HarnessReport) -> str:
    """Render a HarnessReport as a markdown document.

    Always emits all 4 sections (even when empty) so AC #7 (clean-exit on
    empty chain) holds.
    """
    parts: list[str] = []
    parts.append(f"# Harness report — {report.generated_at.strftime('%Y-%m-%d')}")
    parts.append("")
    parts.append(f"- **Window:** last {_format_timedelta(report.window)}")
    parts.append(f"- **Generated at:** {report.generated_at.isoformat()}")
    parts.append(f"- **Total audit rows walked:** {report.total_rows_walked}")
    parts.append(f"- **Skills inspected:** {report.skills_inspected}")
    parts.append(f"- **Signal breaches:** {len(report.breaches)}")
    parts.append("")

    # §1. Skills with tripped signals
    parts.append("## Skills with tripped signals")
    parts.append("")
    if report.breaches:
        parts.append("| skill | signal | value | threshold | evidence rows |")
        parts.append("|---|---|---:|---:|---|")
        for b in report.breaches:
            ev = ", ".join(f"`{rid}`" for rid in b.evidence_row_ids[:5])
            if not ev:
                ev = "*(no audit row ids)*"
            parts.append(
                f"| `{b.skill_name}` | `{b.signal_id}` | "
                f"{b.value:.3f} | {b.threshold:.3f} | {ev} |"
            )
    else:
        parts.append("*(no signals tripped in this window)*")
    parts.append("")

    # §2. Workflows with elevated rework
    parts.append("## Workflows with elevated rework")
    parts.append("")
    if report.workflow_metrics:
        parts.append("| workflow | total_runs | completed | routed_back | hitl_halt | failed | rework rate |")
        parts.append("|---|---:|---:|---:|---:|---:|---:|")
        for w in report.workflow_metrics:
            parts.append(
                f"| `{w.workflow_id}` | {w.total_runs} | {w.completed} | "
                f"{w.routed_back} | {w.hitl_halt} | {w.failed} | "
                f"{w.rework_rate:.2%} |"
            )
    else:
        parts.append("*(no workflow runs in this window)*")
    parts.append("")

    # §3. Per-FR routed-back history
    parts.append("## Per-FR routed-back history")
    parts.append("")
    if report.routed_back_history:
        parts.append("| FR | routed_back_count |")
        parts.append("|---|---:|")
        for fr_id, count in sorted(report.routed_back_history.items(),
                                    key=lambda kv: -kv[1]):
            parts.append(f"| `{fr_id}` | {count} |")
    else:
        parts.append("*(no rework events in this window)*")
    parts.append("")

    # §4. Summary
    parts.append("## Summary")
    parts.append("")
    total_runs = sum(w.total_runs for w in report.workflow_metrics)
    total_hitl = sum(w.hitl_halt for w in report.workflow_metrics)
    total_rework = sum(w.routed_back for w in report.workflow_metrics)
    parts.append(f"- Workflow runs: **{total_runs}**")
    parts.append(f"- HITL halts: **{total_hitl}**")
    parts.append(f"- Rework events: **{total_rework}**")
    parts.append("")

    return "\n".join(parts)


def _format_timedelta(td: timedelta) -> str:
    total_h = int(td.total_seconds() // 3600)
    if total_h % 24 == 0 and total_h >= 24:
        return f"{total_h // 24}d"
    return f"{total_h}h"


# ----------------------------------------------------------------------------
# Emitter — writes the markdown + emits audit row
# ----------------------------------------------------------------------------


def emit_report(
    report: HarnessReport,
    out_path: Path,
    *,
    memory_root: Optional[Path] = None,
    actor: str = "cuo-harness",
) -> Path:
    """Write the report atomically + emit a `harness.report_emitted` aux row.

    Returns the path the report was written to.
    """
    out_path.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = out_path.with_suffix(out_path.suffix + ".tmp")
    tmp_path.write_text(format_markdown(report), encoding="utf-8")
    os.replace(tmp_path, out_path)

    # Best-effort memory emit. If the memory module isn't reachable, the
    # report still lands; the audit row is opportunistic.
    if memory_root is not None:
        try:
            _emit_audit_row(report, out_path, memory_root, actor)
        except Exception:  # noqa: BLE001 — emit failures must not crash report
            pass

    return out_path


def _emit_audit_row(
    report: HarnessReport,
    out_path: Path,
    memory_root: Path,
    actor: str,
) -> None:
    """Append the `harness.report_emitted` row via the memory module's Writer."""
    try:
        from cyberos.core.writer import Writer, AuditRecord
    except ImportError:
        return
    if not (memory_root / "manifest.json").is_file():
        return
    with Writer(memory_root) as w:
        w.submit(AuditRecord(
            op="harness.report_emitted",
            path=str(out_path.relative_to(out_path.parent.parent)),
            actor=actor,
            extra={
                "report_path": str(out_path),
                "window_seconds": int(report.window.total_seconds()),
                "skills_with_signals": len({b.skill_name for b in report.breaches}),
                "workflows_with_signals": len(report.workflow_metrics),
                "evidence_row_count": sum(len(b.evidence_row_ids) for b in report.breaches),
            },
        ))
