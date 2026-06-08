"""Parse node."""

from __future__ import annotations

import time
import unicodedata
import uuid

from ..persona import get_persona, validate_agent_persona_claim
from ..state import CUO_STATE_V, CuoState


def parse_query(
    query: str,
    *,
    tenant_id: str,
    subject_id: str,
    persona_key: str,
    agent_persona: str,
    request_id: str | None = None,
    trace_id: str | None = None,
) -> CuoState:
    get_persona(persona_key)
    version = validate_agent_persona_claim(agent_persona, persona_key)
    rid = request_id or str(uuid.uuid4())
    trace = trace_id or uuid.uuid4().hex + uuid.uuid4().hex[:0]
    return {
        "cuo_state_v": CUO_STATE_V,
        "query": query,
        "normalized_query": unicodedata.normalize("NFC", query),
        "tenant_id": tenant_id,
        "subject_id": subject_id,
        "persona_key": persona_key,
        "persona_version": version,
        "request_id": rid,
        "trace_id": trace[:32].lower(),
        "ts_ns_start": time.time_ns(),
        "cascade_taken": False,
        "rule_scores": [],
        "audit_rows": [],
        "errors": [],
    }
