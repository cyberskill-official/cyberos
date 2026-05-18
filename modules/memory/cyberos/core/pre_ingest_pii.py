"""FR-BRAIN-111 — Pre-ingest PII detection gate.

EVERY memory row that's about to be written to Layer 1 passes through this
gate first. The gate runs two detectors:

1. **Presidio-style global PII** — emails, SSNs, credit cards, phone numbers,
   passport numbers, IBAN. Slice-1 ships a regex-based subset; slice-2 wires
   the real `presidio-analyzer` (heavier dep).

2. **VN-PII** — Vietnamese-specific identifiers per the FR-AI-013 corpus:
   CCCD (12-digit citizen ID), MST (10/13-digit tax ID), phone numbers in
   E.164 ``+84`` form.

The gate produces a :class:`PiiReport` listing every hit. Policy:

* When ``policy == "block"``: any hit raises :class:`PiiBlockedError`. The
  caller MUST either redact or explicitly accept the risk by re-submitting
  with the matching ``pii_allowlist`` frontmatter field populated.
* When ``policy == "redact"``: the body is rewritten with each hit replaced
  by a kind-specific sentinel (``[EMAIL]``, ``[CCCD]``, etc.) and the report
  records what was redacted.
* When ``policy == "log"``: hits are returned but the body is unchanged
  (development / migration mode).

The default policy is ``"block"`` per AGENTS.md §11 (PII never enters BRAIN
raw).
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from typing import Literal

PiiPolicy = Literal["block", "redact", "log"]


@dataclass(frozen=True)
class PiiHit:
    """One detection."""

    kind: str          # 'email' | 'cccd' | 'mst' | 'phone' | 'iban' | 'card' | …
    text: str          # the matched substring
    start: int         # byte offset in the original body
    end: int


@dataclass
class PiiReport:
    """Result of one body scan."""

    hits: list[PiiHit] = field(default_factory=list)
    redacted_body: str | None = None
    policy: PiiPolicy = "block"

    @property
    def has_hits(self) -> bool:
        return bool(self.hits)


class PiiBlockedError(ValueError):
    """The ``block`` policy refused this body."""

    def __init__(self, report: PiiReport):
        kinds = sorted({h.kind for h in report.hits})
        super().__init__(f"pre-ingest PII gate blocked: kinds={kinds}, n_hits={len(report.hits)}")
        self.report = report


# ---------------------------------------------------------------------------
# Detector regex bank (slice 1)
# ---------------------------------------------------------------------------

_DETECTORS: dict[str, re.Pattern[str]] = {
    # Email: RFC-5321 simplified — local-part allowed chars + @ + domain.
    "email": re.compile(
        r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}",
    ),
    # CCCD — 12 digits. Avoid matching inside longer digit runs by anchoring
    # to non-digit boundaries.
    "cccd": re.compile(r"(?<!\d)\d{12}(?!\d)"),
    # MST — 10 digits, optionally followed by '-' + 3 digits.
    "mst": re.compile(r"(?<!\d)\d{10}(?:-\d{3})?(?!\d)"),
    # E.164 phone (any country) — '+' + 7..15 digits.
    "phone": re.compile(r"\+\d{7,15}(?!\d)"),
    # IBAN — 15-34 alphanumeric starting with two letters.
    "iban": re.compile(r"\b[A-Z]{2}\d{2}[A-Z0-9]{11,30}\b"),
    # Credit card — 13-19 digit run with optional dashes/spaces (no Luhn check).
    "card": re.compile(r"(?:\d[ -]?){12,18}\d"),
}

# Sentinels for redact policy.
_REDACT_SENTINELS: dict[str, str] = {
    "email": "[EMAIL]",
    "cccd":  "[CCCD]",
    "mst":   "[MST]",
    "phone": "[PHONE]",
    "iban":  "[IBAN]",
    "card":  "[CARD]",
}


def scan_pii(
    body: str,
    *,
    policy: PiiPolicy = "block",
    allowlist: tuple[str, ...] = (),
) -> PiiReport:
    """Scan `body` for PII. See module docstring for policy semantics.

    `allowlist` is a tuple of detector kinds whose hits are IGNORED. Use
    this when the tenant has a legitimate exception (e.g. KYC vendor MSTs
    in CRM rows).
    """
    hits: list[PiiHit] = []
    for kind, rx in _DETECTORS.items():
        if kind in allowlist:
            continue
        for m in rx.finditer(body):
            hits.append(PiiHit(kind=kind, text=m.group(0), start=m.start(), end=m.end()))

    report = PiiReport(hits=hits, policy=policy)

    if policy == "redact" and hits:
        # Apply replacements right-to-left so earlier offsets stay valid.
        out = body
        for h in sorted(hits, key=lambda h: -h.start):
            out = out[: h.start] + _REDACT_SENTINELS[h.kind] + out[h.end :]
        report.redacted_body = out

    if policy == "block" and hits:
        raise PiiBlockedError(report)

    return report


# ---------------------------------------------------------------------------
# Convenience: pre-ingest hook used by writer.py
# ---------------------------------------------------------------------------

def pre_ingest_check(
    path: str,
    body: str,
    *,
    frontmatter: dict | None = None,
    default_policy: PiiPolicy = "block",
) -> PiiReport:
    """Wrap :func:`scan_pii` with frontmatter-driven policy resolution.

    Memory frontmatter MAY carry:
      * ``pii_policy`` — override per-memory (rare; typically operator action).
      * ``pii_allowlist`` — tuple of detector kinds to skip.

    The path is included only for diagnostic logging — never to side-channel
    the body itself.
    """
    fm = frontmatter or {}
    policy: PiiPolicy = fm.get("pii_policy", default_policy)
    raw_allow = fm.get("pii_allowlist", [])
    allowlist: tuple[str, ...] = tuple(str(k) for k in raw_allow) if isinstance(raw_allow, (list, tuple)) else ()
    return scan_pii(body, policy=policy, allowlist=allowlist)
