"""TASK-CHAT-010 — Slack/Zalo decommission signal."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class ChannelCounts:
    chat: int
    slack: int
    zalo: int


def decommission_signal(counts: ChannelCounts) -> float:
    total = counts.chat + counts.slack + counts.zalo
    if total == 0:
        return 1.0
    return counts.chat / total


def decommission_ready(counts: ChannelCounts, *, threshold: float = 0.95) -> bool:
    return decommission_signal(counts) >= threshold
