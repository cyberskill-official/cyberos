"""
Phase 0 consent (TASK-IMP-061 / AGENTS.md §19).

``personnel-requires-consent`` fails open personnel memories and passes when
``consent.consent_event`` resolves to ``meta/consent/<id>.md``.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from cyberos.core.invariants import check_personnel_requires_consent


@pytest.fixture(autouse=True)
def _exempt_sandbox_path(monkeypatch, tmp_path):
    monkeypatch.setenv("CYBEROS_HOST_MOUNT_PREFIX", str(tmp_path))


def _init_store(tmp_path: Path, name: str = "store") -> Path:
    store = tmp_path / ".cyberos" / "memory" / name
    (store / "audit").mkdir(parents=True)
    (store / "memories" / "people").mkdir(parents=True)
    (store / "meta" / "consent").mkdir(parents=True)
    (store / "manifest.json").write_text(json.dumps({"schema_version": 1}))
    return store


def _write_person(
    store: Path,
    *,
    name: str,
    has_consent: bool | None,
    consent_event: str | None,
    classification: str = "personnel",
) -> Path:
    consent_lines = ["consent:"]
    if has_consent is None:
        consent_lines = ["consent: {}"]
    else:
        consent_lines.append(f"  has_consent: {'true' if has_consent else 'false'}")
        if consent_event is None:
            consent_lines.append("  consent_event: null")
        else:
            consent_lines.append(f"  consent_event: {consent_event}")
        consent_lines.append("  consent_scope: [personnel]")
    body = "\n".join(
        [
            "---",
            f"memory_id: mem_{name}",
            "scope: memories/people",
            f"classification: {classification}",
            "kind: person",
            "authority: human-edited",
            "version: 1",
            "created_at: 2026-07-25T00:00:00Z",
            "created_by: subject:tester",
            "last_updated_at: 2026-07-25T00:00:00Z",
            "updated_by: subject:tester",
            *consent_lines,
            "tags: [test]",
            "---",
            "",
            f"# PERSON {name}",
            "",
        ]
    )
    path = store / "memories" / "people" / f"{name}.md"
    path.write_text(body, encoding="utf-8")
    return path


def test_empty_store_passes(tmp_path: Path) -> None:
    store = _init_store(tmp_path, "empty")
    passed, details = check_personnel_requires_consent(store)
    assert passed, details
    assert "no personnel-gated" in details


def test_operational_without_consent_passes(tmp_path: Path) -> None:
    store = _init_store(tmp_path, "ops")
    (store / "memories" / "facts").mkdir(parents=True, exist_ok=True)
    (store / "memories" / "facts" / "FACT-001.md").write_text(
        "---\nmemory_id: mem_fact\nscope: memories/facts\n"
        "classification: operational\nkind: fact\n---\n\n# fact\n",
        encoding="utf-8",
    )
    passed, details = check_personnel_requires_consent(store)
    assert passed, details


def test_personnel_missing_consent_fails(tmp_path: Path) -> None:
    store = _init_store(tmp_path, "no-consent")
    _write_person(store, name="bob", has_consent=False, consent_event=None)
    passed, details = check_personnel_requires_consent(store)
    assert not passed
    assert "has_consent" in details


def test_personnel_null_event_fails(tmp_path: Path) -> None:
    store = _init_store(tmp_path, "null-event")
    _write_person(store, name="bob", has_consent=True, consent_event=None)
    passed, details = check_personnel_requires_consent(store)
    assert not passed
    assert "consent_event" in details


def test_personnel_unresolved_event_fails(tmp_path: Path) -> None:
    store = _init_store(tmp_path, "unresolved")
    _write_person(store, name="bob", has_consent=True, consent_event="evt_missing")
    passed, details = check_personnel_requires_consent(store)
    assert not passed
    assert "unresolved" in details


def test_personnel_with_consent_file_passes(tmp_path: Path) -> None:
    store = _init_store(tmp_path, "ok-file")
    (store / "meta" / "consent" / "evt_bob_v1.md").write_text(
        "---\nscope: meta/consent\n---\n\n# CONSENT evt_bob_v1\n",
        encoding="utf-8",
    )
    _write_person(store, name="bob", has_consent=True, consent_event="evt_bob_v1")
    passed, details = check_personnel_requires_consent(store)
    assert passed, details
    assert "1 personnel-gated" in details


def test_personnel_with_audit_id_passes(tmp_path: Path) -> None:
    store = _init_store(tmp_path, "ok-audit")
    (store / "audit" / "2026-07.jsonl").write_text(
        json.dumps({"audit_id": "evt_audit_1", "op": "consent.recorded"}) + "\n",
        encoding="utf-8",
    )
    _write_person(store, name="bob", has_consent=True, consent_event="evt_audit_1")
    passed, details = check_personnel_requires_consent(store)
    assert passed, details


def test_readme_is_not_a_consent_event(tmp_path: Path) -> None:
    store = _init_store(tmp_path, "readme")
    (store / "meta" / "consent" / "README.md").write_text("# docs\n", encoding="utf-8")
    _write_person(store, name="bob", has_consent=True, consent_event="README")
    passed, details = check_personnel_requires_consent(store)
    assert not passed
    assert "unresolved" in details
