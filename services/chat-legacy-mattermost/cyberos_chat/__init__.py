"""CyberOS CHAT service helpers.

The Mattermost fork stays as the deployable chat server. This package holds
CyberOS-specific control-plane logic for FR-CHAT-003..012: tenant deployment
plans, Vietnamese search normalization, logical-replication memory capture,
imports, Lumi routing, privacy-preserving push, and DSAR export.
"""

from .deployment import DeploymentPlan, TenantDeploymentSpec, build_deployment_plan
from .decommission import ChannelCounts, decommission_ready, decommission_signal
from .dsar import DsarExport, export_subject_messages
from .lumi import LumiMention, parse_lumi_mention, parse_retro_capture, retro_capture_selection
from .memory_bridge import ChatMessage, MemoryBridge, MemoryCaptureRow
from .push import PushPayload, build_privacy_payload
from .search import SearchIndex, normalize_vietnamese, vietnamese_bigrams

__all__ = [
    "ChatMessage",
    "ChannelCounts",
    "DeploymentPlan",
    "DsarExport",
    "LumiMention",
    "MemoryBridge",
    "MemoryCaptureRow",
    "PushPayload",
    "SearchIndex",
    "TenantDeploymentSpec",
    "build_deployment_plan",
    "build_privacy_payload",
    "decommission_ready",
    "decommission_signal",
    "export_subject_messages",
    "normalize_vietnamese",
    "parse_lumi_mention",
    "parse_retro_capture",
    "retro_capture_selection",
    "vietnamese_bigrams",
]
