"""FR-AI-011/012 — Presidio sidecar for PII redaction.

FastAPI service wrapping Microsoft Presidio Analyzer + Anonymizer.
Binds to 127.0.0.1:5050 ONLY — never 0.0.0.0 (FR-AI-011 §1 #15).

Usage:
    python presidio_sidecar.py
    # or: uvicorn presidio_sidecar:app --host 127.0.0.1 --port 5050
"""

from __future__ import annotations

import logging
from typing import List

from fastapi import FastAPI, HTTPException
from fastapi.exceptions import RequestValidationError
from fastapi.responses import JSONResponse
from pydantic import BaseModel

logger = logging.getLogger(__name__)

# ── Pydantic models ──────────────────────────────────────────────────────────


class RedactRequest(BaseModel):
    text: str
    extra_entities: List[str] = []


class RedactItem(BaseModel):
    entity: str
    start: int
    end: int
    original: str


class RedactResponse(BaseModel):
    redacted_text: str
    items: List[RedactItem]


# ── App setup ────────────────────────────────────────────────────────────────

app = FastAPI(title="Presidio PII Redaction Sidecar", version="0.1.0")


# ISS-003 fix: FastAPI's default 422 handler echoes the request body in the
# response, which can leak prompt fragments (FR-AI-011 §1 #12).
@app.exception_handler(RequestValidationError)
async def custom_validation_handler(request, exc):
    return JSONResponse(
        status_code=422, content={"detail": "validation_error"}
    )


# ── Presidio engines ─────────────────────────────────────────────────────────

DEFAULT_ENTITIES = [
    "CREDIT_CARD",
    "US_SSN",
    "EMAIL_ADDRESS",
    "PHONE_NUMBER",
    "PERSON",
    "LOCATION",
    "IP_ADDRESS",
    "IBAN_CODE",
    "US_BANK_NUMBER",
    "MEDICAL_LICENSE",
]


def _create_analyzer():
    """Create AnalyzerEngine; separated for testability."""
    from presidio_analyzer import AnalyzerEngine

    return AnalyzerEngine()


def _create_anonymizer():
    """Create AnonymizerEngine; separated for testability."""
    from presidio_anonymizer import AnonymizerEngine

    return AnonymizerEngine()


# Lazy-init: engines are created on first request to keep startup fast.
_analyzer = None
_anonymizer = None


def _get_analyzer():
    global _analyzer
    if _analyzer is None:
        _analyzer = _create_analyzer()
    return _analyzer


def _get_anonymizer():
    global _anonymizer
    if _anonymizer is None:
        _anonymizer = _create_anonymizer()
    return _anonymizer


# ── VN recognizer registration (FR-AI-012) ───────────────────────────────────

# Idempotency guard against double-registration.
_VN_REGISTERED = False


def register_vn_recognizers(analyzer):
    """§1 #13: refuse to start if any registration fails. Idempotent."""
    global _VN_REGISTERED
    if _VN_REGISTERED:
        logger.warning("register_vn_recognizers called twice; ignoring")
        return
    from recognizers import VN_RECOGNIZERS

    for rec in VN_RECOGNIZERS:
        try:
            analyzer.registry.add_recognizer(rec)
            pattern_count = len(getattr(rec, "PATTERNS", []))
            logger.info(
                "registered %s v%s (%d patterns)",
                rec.__class__.__name__,
                rec.VERSION,
                pattern_count,
            )
        except Exception as e:
            raise RuntimeError(
                f"recognizer_registration_failed: {rec.__class__.__name__}: {e}"
            )
    _VN_REGISTERED = True


def _ensure_vn_recognizers():
    """Register VN recognizers on first analyzer access."""
    analyzer = _get_analyzer()
    register_vn_recognizers(analyzer)
    return analyzer


def reset_vn_for_tests():
    """Test-only: reset the registration guard. NOT for production use."""
    global _VN_REGISTERED
    _VN_REGISTERED = False


# ── Redaction logic ──────────────────────────────────────────────────────────


def _build_placeholder_operator(entity_type: str):
    """Return an OperatorConfig that replaces with <ENTITY_TYPE_N>."""
    from presidio_anonymizer.entities import OperatorConfig

    counter = {"n": 0}

    def _replace(original_text: str, params):
        counter["n"] += 1
        return f"<{entity_type}_{counter['n']}>"

    return OperatorConfig("custom", {"lambda": _replace})


@app.post("/redact", response_model=RedactResponse)
async def redact_endpoint(req: RedactRequest):
    """Redact PII from text using Presidio Analyzer + Anonymizer."""
    try:
        analyzer = _ensure_vn_recognizers()
        anonymizer = _get_anonymizer()

        entities = DEFAULT_ENTITIES + req.extra_entities

        # Support both EN and VI for VN recognizers.
        results = analyzer.analyze(
            text=req.text,
            language="en",
            entities=entities,
        )
        vi_results = analyzer.analyze(
            text=req.text,
            language="vi",
            entities=entities,
        )
        # Merge: deduplicate by (start, end, entity_type), prefer higher score.
        seen = {}
        for r in results + vi_results:
            key = (r.start, r.end, r.entity_type)
            if key not in seen or r.score > seen[key].score:
                seen[key] = r
        results = list(seen.values())

        # Sort by start offset for deterministic placeholder assignment (§1 #11).
        results.sort(key=lambda r: r.start)

        operators = {e: _build_placeholder_operator(e) for e in entities}

        anonymized = anonymizer.anonymize(
            text=req.text,
            analyzer_results=results,
            operators=operators,
        )

        items = []
        for r in results:
            items.append(
                RedactItem(
                    entity=r.entity_type,
                    start=r.start,
                    end=r.end,
                    original=req.text[r.start : r.end],
                )
            )

        return RedactResponse(redacted_text=anonymized.text, items=items)

    except HTTPException:
        raise
    except Exception:
        # GENERIC error message; do NOT echo the prompt (§1 #12).
        logger.exception("redaction_internal_error")
        raise HTTPException(
            status_code=500, detail="redaction_internal_error"
        )


# ── Health + version endpoints ────────────────────────────────────────────────


@app.get("/health")
async def health():
    return {"status": "ok"}


@app.get("/recognizers/version")
async def recognizer_versions():
    """§1 #14: version endpoint for FR-AI-013 recall-gate."""
    from recognizers import VN_RECOGNIZERS

    return {
        rec.supported_entities[0]: rec.VERSION for rec in VN_RECOGNIZERS
    }


# ── Entrypoint ───────────────────────────────────────────────────────────────

if __name__ == "__main__":
    import uvicorn

    # §1 #15: bind to 127.0.0.1 ONLY — never 0.0.0.0.
    uvicorn.run(app, host="127.0.0.1", port=5050)
