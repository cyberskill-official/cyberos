"""CUO HTTP triage endpoint — serves `obs.triage-alert@1` to obs-router (TASK-OBS-007 §1 #2).

obs-router's `cuo_triage.rs` POSTs one alert per fired Alertmanager alert and expects a triage verdict
back within 5 seconds. CUO has no long-running HTTP server otherwise — skills are invoked in-process via
`select_invoker()` — so this module is the thin HTTP front door that maps {skill, alert} → a skill
invocation → the verdict contract obs-router parses.

The request/response contract is fixed by `services/obs-router/src/cuo_triage.rs`:

    POST /  {"skill":"obs.triage-alert@1","alert":{"name","severity","fingerprint","trace_id","summary"}}
    200     {"confidence":0.82,"summary":"...","suspected_cause":"...","suggested_runbook":{"url":"..."}|null}

Safe degradation is deliberate, not a bug. The skill's own guardrail (SKILL.md §5) says: if triage cannot
reach its inputs (no invoker, no metrics, no runbook corpus), return a LOW confidence with a summary that
says so — obs-router then pages on-call, which is the correct outcome for an uncertain alert. So an
unavailable invoker or a failed invocation returns HTTP 200 with confidence 0.0, not a 5xx. obs-router
routes confidence < 0.70 to PagerDuty either way; the explicit low-confidence body is honest about why.

The pure request handler (`handle_triage_request`) takes an injected invoker, so it is unit-tested with a
fake invoker — no LLM, no network. `select_invoker` (LLM/subprocess) is only reached by the live server.
"""

from __future__ import annotations

import argparse
import json
import os
import tempfile
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

# The invocation handle obs-router sends, and the skill directory under skill_root it maps to.
SKILL_HANDLE = "obs.triage-alert@1"
SKILL_DIR_NAME = "obs-triage-alert"
# A response body larger than this is refused before parsing (a fired alert is small).
MAX_BODY_BYTES = 64 * 1024


def alert_to_inputs(alert: dict) -> dict:
    """Map the obs-router alert object into the skill's input dict.

    The skill reads the alert under an ``alert`` key (its SKILL.md §1 input shape). Pass the fields
    through verbatim so the skill sees exactly what obs-router sent — no enrichment happens here.
    """
    return {
        "alert": {
            "name": str(alert.get("name", "")),
            "severity": str(alert.get("severity", "")),
            "fingerprint": str(alert.get("fingerprint", "")),
            "trace_id": str(alert.get("trace_id", "")),
            "summary": str(alert.get("summary", "")),
        }
    }


def _clamp_confidence(value) -> float:
    """Coerce the skill's confidence into a [0, 1] float; anything unparseable is 0.0 (page)."""
    try:
        c = float(value)
    except (TypeError, ValueError):
        return 0.0
    if c < 0.0:
        return 0.0
    if c > 1.0:
        return 1.0
    return c


def _normalize_runbook(runbook) -> dict | None:
    """Reduce the skill's runbook (``{title, url}``) to what obs-router reads (``url``).

    obs-router's ``RunbookDto`` only reads ``url``; a runbook with no usable url becomes ``null`` so the
    router does not surface an empty link. Never fabricate a url.
    """
    if not isinstance(runbook, dict):
        return None
    url = runbook.get("url")
    if not isinstance(url, str) or not url.strip():
        return None
    out = {"url": url}
    title = runbook.get("title")
    if isinstance(title, str) and title.strip():
        out["title"] = title
    return out


def extract_triage(output: dict) -> dict:
    """Project a skill output dict onto the obs-router response contract.

    Missing or malformed fields degrade safely: confidence to 0.0, text to empty, runbook to null. The
    confidence is clamped to [0, 1] because it directly gates routing (>= 0.70 posts to CHAT, not pages).
    """
    if not isinstance(output, dict):
        return safe_degrade("skill output was not an object")
    return {
        "confidence": _clamp_confidence(output.get("confidence")),
        "summary": str(output.get("summary", "")),
        "suspected_cause": str(output.get("suspected_cause", "")),
        "suggested_runbook": _normalize_runbook(output.get("suggested_runbook")),
    }


def safe_degrade(reason: str) -> dict:
    """The verdict to return when triage could not run with its inputs (SKILL.md §5).

    Confidence 0.0 routes obs-router to PagerDuty - the correct outcome for an alert triage could not
    assess. The summary names the reason so the on-call engineer knows triage ran blind.
    """
    return {
        "confidence": 0.0,
        "summary": f"triage ran without its inputs: {reason}",
        "suspected_cause": "",
        "suggested_runbook": None,
    }


def handle_triage_request(
    payload: dict,
    *,
    invoker,
    skill_root: Path,
    output_dir: Path,
) -> tuple[int, dict]:
    """Pure request handler: validate, invoke the skill, project the verdict.

    Args:
        payload: the decoded POST body.
        invoker: an `Invoker` (real or fake). ``None`` means none is available — a safe-degrade 200.
        skill_root: the `skill/` module root (contains `obs-triage-alert/SKILL.md`).
        output_dir: scratch directory for the invoker's step-output file.

    Returns:
        (http_status, body_dict). Validation failures are 4xx; everything else is 200 (including the
        safe-degrade verdict), because an unsure triage is a low-confidence answer, not a server error.
    """
    if not isinstance(payload, dict):
        return 400, {"error": "body must be a JSON object"}
    skill = payload.get("skill")
    if skill != SKILL_HANDLE:
        return 400, {"error": f"unsupported skill {skill!r}; this endpoint serves {SKILL_HANDLE}"}
    alert = payload.get("alert")
    if not isinstance(alert, dict):
        return 400, {"error": "missing or malformed 'alert' object"}
    if not str(alert.get("name", "")).strip():
        return 400, {"error": "alert.name is required"}

    if invoker is None:
        return 200, safe_degrade("no skill invoker available (set ANTHROPIC_API_KEY or build cyberos-skill)")

    inputs = alert_to_inputs(alert)
    try:
        result = invoker.invoke(
            SKILL_DIR_NAME,
            inputs,
            skill_root,
            output_dir,
            1,
            file_prefix="obs-triage_",
        )
    except Exception as e:  # noqa: BLE001 - any invoker failure degrades to a page, never a 5xx
        return 200, safe_degrade(f"invoker raised {type(e).__name__}: {e}")

    if getattr(result, "status", "FAILED") != "OK":
        notes = "; ".join(getattr(result, "notes", []) or [])
        return 200, safe_degrade(f"invoker status={getattr(result, 'status', '?')}: {notes}")

    return 200, extract_triage(getattr(result, "output", {}) or {})


def _resolve_skill_root(explicit: str | None) -> Path:
    """Find the `skill/` module root: an explicit flag, else `$CYBEROS_ROOT/modules/skill`, else walk up."""
    if explicit:
        return Path(explicit).resolve()
    env_root = os.environ.get("CYBEROS_ROOT")
    if env_root:
        cand = Path(env_root).resolve() / "modules" / "skill"
        if (cand / "MODULE.md").is_file():
            return cand
    cur = Path(__file__).resolve()
    for _ in range(8):
        cand = cur / "modules" / "skill"
        if (cand / "MODULE.md").is_file():
            return cand
        if cur.parent == cur:
            break
        cur = cur.parent
    raise RuntimeError("could not locate skill/ root; pass --skill-root or set CYBEROS_ROOT")


def _make_handler(skill_root: Path, output_dir: Path, invoker):
    """Build a request-handler class bound to the server's invoker + roots."""

    class TriageHandler(BaseHTTPRequestHandler):
        def _send(self, status: int, body: dict) -> None:
            data = json.dumps(body).encode("utf-8")
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)

        def do_GET(self) -> None:  # noqa: N802 - BaseHTTPRequestHandler dispatch name
            if self.path == "/healthz":
                self._send(200, {"status": "ok", "skill": SKILL_HANDLE})
            else:
                self._send(404, {"error": "not found"})

        def do_POST(self) -> None:  # noqa: N802 - BaseHTTPRequestHandler dispatch name
            length = int(self.headers.get("Content-Length", 0) or 0)
            if length > MAX_BODY_BYTES:
                self._send(413, {"error": "request body too large"})
                return
            raw = self.rfile.read(length) if length else b""
            try:
                payload = json.loads(raw.decode("utf-8")) if raw else {}
            except (UnicodeDecodeError, json.JSONDecodeError) as e:
                self._send(400, {"error": f"invalid JSON: {e}"})
                return
            status, body = handle_triage_request(
                payload, invoker=invoker, skill_root=skill_root, output_dir=output_dir
            )
            self._send(status, body)

        def log_message(self, *_args) -> None:  # silence default stderr access log
            pass

    return TriageHandler


def serve(host: str, port: int, *, skill_root: Path, invoker=None) -> None:
    """Run the triage server. When ``invoker`` is None, select one at request time per environment."""
    if invoker is None:
        from cuo.core.invoker import select_invoker

        try:
            invoker = select_invoker(prefer="auto")
        except RuntimeError as e:
            # No invoker on this host - the server still answers, returning safe-degrade verdicts so
            # obs-router pages rather than failing to reach triage at all.
            print(f"[triage] warning: {e}\n[triage] serving safe-degrade verdicts (confidence 0.0)")
            invoker = None

    output_dir = Path(tempfile.gettempdir()) / "obs-triage-out"
    output_dir.mkdir(parents=True, exist_ok=True)
    handler = _make_handler(skill_root, output_dir, invoker)
    httpd = ThreadingHTTPServer((host, port), handler)
    print(f"[triage] obs.triage-alert endpoint on http://{host}:{port} (skill_root={skill_root})")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        httpd.shutdown()


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="CUO obs.triage-alert HTTP endpoint for obs-router")
    parser.add_argument("--host", default="0.0.0.0")
    parser.add_argument("--port", type=int, default=8731)
    parser.add_argument("--skill-root", default=None, help="path to modules/skill (default: autodetect)")
    args = parser.parse_args(argv)
    skill_root = _resolve_skill_root(args.skill_root)
    serve(args.host, args.port, skill_root=skill_root)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
