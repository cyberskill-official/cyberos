//! FR-PROJ-010 — citation drift detection for MEMORY_LINKs.

use crate::memory_link::{MemoryLink, MemoryLinkType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftReason {
    TargetDeleted,
    TargetSuperseded,
    BrokenBacklink,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriftFinding {
    pub memory_path: String,
    pub reason: DriftReason,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryCitationSnapshot {
    pub existing_paths: BTreeSet<String>,
    pub superseded_paths: BTreeSet<String>,
    pub backlink_paths: BTreeSet<String>,
}

pub fn detect_drift(links: &[MemoryLink], snapshot: &MemoryCitationSnapshot) -> Vec<DriftFinding> {
    let mut findings = Vec::new();
    for link in links.iter().filter(|l| l.removed_at.is_none()) {
        if !snapshot.existing_paths.contains(&link.memory_path) {
            findings.push(DriftFinding {
                memory_path: link.memory_path.clone(),
                reason: DriftReason::TargetDeleted,
            });
            continue;
        }
        if link.link_type != MemoryLinkType::Supersedes
            && snapshot.superseded_paths.contains(&link.memory_path)
        {
            findings.push(DriftFinding {
                memory_path: link.memory_path.clone(),
                reason: DriftReason::TargetSuperseded,
            });
        }
        if !snapshot.backlink_paths.contains(&link.memory_path) {
            findings.push(DriftFinding {
                memory_path: link.memory_path.clone(),
                reason: DriftReason::BrokenBacklink,
            });
        }
    }
    findings
}
