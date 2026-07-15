"""TASK-CHAT-006/007 — Slack and Zalo importers."""

from __future__ import annotations

import json
import unicodedata
import zipfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable


@dataclass(frozen=True)
class ImportedMessage:
    source: str
    source_id: str
    channel: str
    sender: str
    text: str
    ts: str


@dataclass
class ImportCheckpoint:
    seen_ids: set[str] = field(default_factory=set)


SLACK_IMPORT_STEPS: tuple[str, ...] = (
    "load_export",
    "validate_workspace",
    "map_users",
    "map_channels",
    "normalize_messages",
    "dedupe_checkpoint",
    "write_batches",
    "persist_checkpoint",
)


class SlackImporter:
    """Idempotent Slack export importer.

    Accepts rows from Slack JSON exports and emits normalized message records.
    """

    def __init__(self, checkpoint: ImportCheckpoint | None = None) -> None:
        self.checkpoint = checkpoint or ImportCheckpoint()

    def import_rows(self, channel: str, rows: Iterable[dict]) -> list[ImportedMessage]:
        imported: list[ImportedMessage] = []
        for row in rows:
            source_id = str(row.get("client_msg_id") or row.get("ts") or "")
            if not source_id or source_id in self.checkpoint.seen_ids:
                continue
            self.checkpoint.seen_ids.add(source_id)
            imported.append(
                ImportedMessage(
                    source="slack",
                    source_id=source_id,
                    channel=channel,
                    sender=str(row.get("user") or row.get("username") or "unknown"),
                    text=unicodedata.normalize("NFC", str(row.get("text") or "")),
                    ts=str(row.get("ts") or ""),
                )
            )
        return imported


def import_zalo_bundle(bundle: Path, checkpoint: ImportCheckpoint | None = None) -> list[ImportedMessage]:
    checkpoint = checkpoint or ImportCheckpoint()
    imported: list[ImportedMessage] = []
    with zipfile.ZipFile(bundle) as zf:
        for name in sorted(zf.namelist()):
            if name.endswith("/") or not (name.endswith(".json") or name.endswith(".txt")):
                continue
            raw = zf.read(name).decode("utf-8-sig")
            rows = _parse_zalo_payload(raw, source_name=name)
            for row in rows:
                source_id = row.source_id
                if source_id in checkpoint.seen_ids:
                    continue
                checkpoint.seen_ids.add(source_id)
                imported.append(row)
    return imported


def _parse_zalo_payload(raw: str, *, source_name: str) -> list[ImportedMessage]:
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError:
        payload = [
            {"id": f"{source_name}:{idx}", "sender": "unknown", "text": line, "ts": ""}
            for idx, line in enumerate(raw.splitlines())
            if line.strip()
        ]
    if isinstance(payload, dict):
        payload = payload.get("messages", [])
    rows: list[ImportedMessage] = []
    for idx, item in enumerate(payload):
        text = unicodedata.normalize("NFC", str(item.get("text") or item.get("body") or ""))
        rows.append(
            ImportedMessage(
                source="zalo",
                source_id=str(item.get("id") or f"{source_name}:{idx}"),
                channel=str(item.get("conversation") or source_name),
                sender=str(item.get("sender") or item.get("from") or "unknown"),
                text=text,
                ts=str(item.get("ts") or item.get("timestamp") or ""),
            )
        )
    return rows
