"""TASK-CHAT-011 — privacy-preserving mobile push payloads."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class PushPayload:
    provider: str
    title: str
    sender: str
    data: dict[str, str]


def build_privacy_payload(
    *,
    provider: str,
    channel_name: str,
    sender_display: str,
    message_id: str,
    tenant_id: str,
) -> PushPayload:
    if provider not in {"apns", "fcm"}:
        raise ValueError("provider must be apns or fcm")
    return PushPayload(
        provider=provider,
        title=channel_name[:80],
        sender=sender_display[:80],
        data={"message_id": message_id, "tenant_id": tenant_id},
    )
