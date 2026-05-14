//! Strongly-typed Rust model of the Anthropic Agent Skills SKILL.md
//! frontmatter. Per the open spec at agentskills.io/specification.
//!
//! Only `name` and `description` are spec-required. CyberOS treats
//! `metadata.version` (SemVer) as additionally required for registry-
//! resolved skills (enforced at resolve time, not parse time).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    pub name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
    #[serde(default, rename = "allowed-tools", skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<AllowedTools>,
    /// Catch-all for forward-compat. We do NOT use `deny_unknown_fields`
    /// at the top level because the open spec is intentionally permissive
    /// of agent-specific extensions. Unknown fields land here.
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllowedTools {
    Inline(String),
    List(Vec<String>),
}

impl AllowedTools {
    pub fn as_vec(&self) -> Vec<&str> {
        match self {
            AllowedTools::Inline(s) => s.split_whitespace().collect(),
            AllowedTools::List(v) => v.iter().map(String::as_str).collect(),
        }
    }
}

/// Parse the YAML frontmatter at the head of a SKILL.md file.
/// Returns (manifest, body_offset) — body_offset is where the markdown begins.
pub fn parse_frontmatter(bytes: &[u8]) -> anyhow::Result<(SkillManifest, usize)> {
    const DELIM: &[u8] = b"---\n";
    anyhow::ensure!(
        bytes.starts_with(DELIM),
        "SKILL.md must start with '---'"
    );
    let rest = &bytes[DELIM.len()..];
    let end = memchr::memmem::find(rest, DELIM)
        .ok_or_else(|| anyhow::anyhow!("SKILL.md missing closing '---'"))?;
    let yaml = std::str::from_utf8(&rest[..end])?;
    let manifest: SkillManifest = serde_yaml::from_str(yaml)?;
    let body_offset = DELIM.len() + end + DELIM.len();
    Ok((manifest, body_offset))
}

/// Validate manifest against the spec rules:
/// - name: 1-64 chars, [a-z0-9-]
/// - description: 1-1024 chars
/// - reserved names rejected
pub fn validate_manifest(m: &SkillManifest) -> anyhow::Result<()> {
    if m.name.is_empty() || m.name.len() > 64 {
        anyhow::bail!("name must be 1-64 chars (got {})", m.name.len());
    }
    if !m.name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        anyhow::bail!("name must contain only [a-z0-9-]");
    }
    if m.name == "anthropic" || m.name == "claude" {
        anyhow::bail!("name '{}' is reserved", m.name);
    }
    if m.description.is_empty() || m.description.len() > 1024 {
        anyhow::bail!("description must be 1-1024 chars (got {})", m.description.len());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_frontmatter() {
        let src = b"---\nname: hello\ndescription: Say hi\n---\nbody text";
        let (m, off) = parse_frontmatter(src).unwrap();
        assert_eq!(m.name, "hello");
        assert_eq!(m.description, "Say hi");
        assert_eq!(&src[off..], b"body text");
    }

    #[test]
    fn rejects_missing_frontmatter() {
        let src = b"just markdown";
        assert!(parse_frontmatter(src).is_err());
    }

    #[test]
    fn rejects_reserved_name() {
        let src = b"---\nname: anthropic\ndescription: x\n---\n";
        let (m, _) = parse_frontmatter(src).unwrap();
        assert!(validate_manifest(&m).is_err());
    }

    #[test]
    fn parses_allowed_tools_inline_and_list() {
        let inline = b"---\nname: a\ndescription: x\nallowed-tools: read_file write_file\n---\n";
        let (m, _) = parse_frontmatter(inline).unwrap();
        assert_eq!(m.allowed_tools.unwrap().as_vec(), vec!["read_file", "write_file"]);

        let list = b"---\nname: a\ndescription: x\nallowed-tools:\n  - read_file\n  - write_file\n---\n";
        let (m, _) = parse_frontmatter(list).unwrap();
        assert_eq!(m.allowed_tools.unwrap().as_vec(), vec!["read_file", "write_file"]);
    }
}
