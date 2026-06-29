from __future__ import annotations

import sys
import zipfile
from datetime import datetime, timedelta, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from cyberos_chat.deployment import TenantDeploymentSpec, build_deployment_plan
from cyberos_chat.decommission import ChannelCounts, decommission_ready, decommission_signal
from cyberos_chat.dsar import export_subject_messages
from cyberos_chat.importers import ImportCheckpoint, SLACK_IMPORT_STEPS, SlackImporter, import_zalo_bundle
from cyberos_chat.lumi import parse_lumi_mention, parse_retro_capture, retro_capture_selection
from cyberos_chat.memory_bridge import ChatMessage, MemoryBridge
from cyberos_chat.push import build_privacy_payload
from cyberos_chat.search import SearchIndex, recall_at, vietnamese_bigrams


def test_fargate_deployment_plan_requires_multi_az_and_auth_https():
    plan = build_deployment_plan(
        TenantDeploymentSpec(
            tenant_id="tenant-a",
            region="sg-1",
            image="mattermost:test",
            auth_jwks_url="https://auth.example/.well-known/jwks.json",
        )
    )
    assert plan.fargate["desired_count"] == 2
    assert plan.rds["multi_az"] is True
    assert plan.redis["transit_encryption"] is True


def test_vietnamese_search_bigram_recall_handles_diacritics():
    assert "th" in vietnamese_bigrams("Thành phố Hồ Chí Minh")
    index = SearchIndex()
    index.add("m1", "Hẹn gặp ở Thành phố Hồ Chí Minh")
    index.add("m2", "Invoice ready")
    found = index.search("thanh pho ho chi minh")
    assert found[0] == "m1"
    assert recall_at(found, {"m1"}) >= 0.8


def test_memory_bridge_idempotency_and_p95_sla():
    now = datetime.now(timezone.utc)
    bridge = MemoryBridge(max_lag_ms=5_000)
    rows = [
        bridge.capture(
            ChatMessage(
                id=f"m{i}",
                tenant_id="t1",
                channel_id="c1",
                subject_id="s1",
                body=f"hello {i}",
                created_at=now - timedelta(seconds=1),
            ),
            captured_at=now,
        )
        for i in range(10)
    ]
    bridge.assert_sla(rows)
    assert rows[0].row_kind == "chat.message_captured"
    assert rows[0].memory_path.startswith("memories/facts/")


def test_slack_import_is_checkpointed_and_zalo_zip_is_unicode_normalized(tmp_path):
    assert len(SLACK_IMPORT_STEPS) == 8
    checkpoint = ImportCheckpoint()
    importer = SlackImporter(checkpoint)
    rows = [{"client_msg_id": "a", "text": "Xin chào", "user": "u1", "ts": "1"}]
    assert len(importer.import_rows("general", rows)) == 1
    assert importer.import_rows("general", rows) == []

    bundle = tmp_path / "zalo.zip"
    with zipfile.ZipFile(bundle, "w") as zf:
        zf.writestr("chat.json", '{"messages":[{"id":"z1","sender":"An","text":"Cha\\u0300o","ts":"2"}]}')
    imported = import_zalo_bundle(bundle)
    assert imported[0].text == "Chào"


def test_lumi_mention_retro_capture_and_push_privacy():
    mention = parse_lumi_mention("@lumi summarize this channel")
    assert mention is not None
    assert mention.route_kind == "cuo.route"
    assert parse_retro_capture("@lumi remember the last 12 messages") == 12
    assert retro_capture_selection(["m3", "m2", "m1"], 2, {"m2"}) == ["m2"]

    payload = build_privacy_payload(
        provider="apns",
        channel_name="delivery",
        sender_display="Stephen",
        message_id="m1",
        tenant_id="t1",
    )
    assert payload.title == "delivery"
    assert payload.sender == "Stephen"
    assert "body" not in payload.data


def test_decommission_signal_threshold():
    counts = ChannelCounts(chat=95, slack=4, zalo=1)
    assert decommission_signal(counts) == 0.95
    assert decommission_ready(counts)


def test_dsar_export_filters_subject_and_includes_memory_hashes():
    now = datetime.now(timezone.utc)
    msg = ChatMessage("m1", "t1", "c1", "s1", "secret", now)
    other = ChatMessage("m2", "t1", "c1", "s2", "other", now)
    bridge = MemoryBridge()
    capture = bridge.capture(msg, captured_at=now)
    export = export_subject_messages("s1", [msg, other], [capture])
    assert [m["id"] for m in export.messages] == ["m1"]
    assert export.memory_hashes == [capture.body_hash]
    assert len(export.manifest_hash) == 64
