//! TASK-SKILL-101/103/104 — capability broker and skill audit rows.

use crate::frontmatter::SkillFrontmatter;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityPolicy {
    pub skill_name: String,
    pub version: Option<String>,
    pub allowed_tools: BTreeSet<String>,
    pub allowed_memory_scopes: BTreeSet<String>,
}

impl CapabilityPolicy {
    pub fn from_frontmatter(frontmatter: &SkillFrontmatter) -> Self {
        let mut tools = frontmatter
            .allowed_tools
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        tools.extend(frontmatter.allowed_mcp_tools.iter().cloned());
        Self {
            skill_name: frontmatter.name.clone(),
            version: frontmatter
                .metadata
                .as_ref()
                .and_then(|m| m.version.clone()),
            allowed_tools: tools,
            allowed_memory_scopes: flatten_scopes(frontmatter.allowed_memory_scopes.as_ref()),
        }
    }

    pub fn authorize_tool(&self, requested: &str) -> Result<(), CapabilityError> {
        if self.allowed_tools.contains(requested)
            || self.allowed_tools.iter().any(|tool| {
                tool.strip_suffix(".*")
                    .map(|prefix| requested.starts_with(prefix))
                    .unwrap_or(false)
            })
        {
            Ok(())
        } else {
            Err(CapabilityError::ToolDenied {
                skill_name: self.skill_name.clone(),
                requested: requested.to_string(),
            })
        }
    }

    pub fn authorize_memory_scope(&self, requested: &str) -> Result<(), CapabilityError> {
        if self.allowed_memory_scopes.contains(requested)
            || self.allowed_memory_scopes.iter().any(|scope| {
                scope
                    .strip_suffix('*')
                    .map(|prefix| requested.starts_with(prefix))
                    .unwrap_or(false)
            })
        {
            Ok(())
        } else {
            Err(CapabilityError::MemoryScopeDenied {
                skill_name: self.skill_name.clone(),
                requested: requested.to_string(),
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CapabilityError {
    #[error("tool denied for {skill_name}: {requested}")]
    ToolDenied {
        skill_name: String,
        requested: String,
    },
    #[error("memory scope denied for {skill_name}: {requested}")]
    MemoryScopeDenied {
        skill_name: String,
        requested: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvocationAuditRow {
    pub row_kind: &'static str,
    pub skill_name: String,
    pub invocation_id: String,
    pub tool: Option<String>,
    pub memory_scope: Option<String>,
    pub outcome: Option<String>,
    pub ts_ns: u128,
}

pub fn invocation_started(
    skill_name: impl Into<String>,
    invocation_id: impl Into<String>,
) -> InvocationAuditRow {
    InvocationAuditRow {
        row_kind: "skill.invocation_started",
        skill_name: skill_name.into(),
        invocation_id: invocation_id.into(),
        tool: None,
        memory_scope: None,
        outcome: None,
        ts_ns: now_ns(),
    }
}

pub fn invocation_completed(
    skill_name: impl Into<String>,
    invocation_id: impl Into<String>,
    outcome: impl Into<String>,
) -> InvocationAuditRow {
    InvocationAuditRow {
        row_kind: "skill.invocation_completed",
        skill_name: skill_name.into(),
        invocation_id: invocation_id.into(),
        tool: None,
        memory_scope: None,
        outcome: Some(outcome.into()),
        ts_ns: now_ns(),
    }
}

fn now_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

fn flatten_scopes(value: Option<&serde_yaml::Value>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    fn walk(value: &serde_yaml::Value, out: &mut BTreeSet<String>) {
        match value {
            serde_yaml::Value::String(s) => {
                out.insert(s.clone());
            }
            serde_yaml::Value::Sequence(seq) => {
                for item in seq {
                    walk(item, out);
                }
            }
            serde_yaml::Value::Mapping(map) => {
                for (_k, v) in map {
                    walk(v, out);
                }
            }
            _ => {}
        }
    }
    if let Some(value) = value {
        walk(value, &mut out);
    }
    out
}
