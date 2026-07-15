"""TASK-CHAT-008/009 — Lumi mention and retro-capture parser."""

from __future__ import annotations

import re
from dataclasses import dataclass


_MENTION_RE = re.compile(r"(?:^|\s)@lumi\b(?P<prompt>.*)", re.IGNORECASE | re.DOTALL)
_RETRO_RE = re.compile(r"remember\s+the\s+last\s+(?P<count>\d{1,3})\s+messages?", re.IGNORECASE)


@dataclass(frozen=True)
class LumiMention:
    prompt: str
    route_kind: str
    requires_memory_capture: bool = True


def parse_lumi_mention(body: str) -> LumiMention | None:
    match = _MENTION_RE.search(body)
    if not match:
        return None
    prompt = match.group("prompt").strip(" :\n\t")
    if not prompt:
        prompt = "help"
    return LumiMention(prompt=prompt, route_kind="cuo.route")


def parse_retro_capture(body: str) -> int | None:
    mention = parse_lumi_mention(body)
    if mention is None:
        return None
    match = _RETRO_RE.search(mention.prompt)
    if not match:
        return None
    count = int(match.group("count"))
    if not 1 <= count <= 100:
        raise ValueError("retro capture count must be between 1 and 100")
    return count


def retro_capture_selection(message_ids_newest_first: list[str], count: int, opted_in_ids: set[str]) -> list[str]:
    if count < 1:
        raise ValueError("count must be positive")
    candidates = message_ids_newest_first[:count]
    return [message_id for message_id in candidates if message_id in opted_in_ids]
