//! Anthropic transpiler — CCSM → Anthropic Agent Skills flat SKILL.md.
//!
//! Per Anthropic's *Complete Guide to Building Skills for Claude* §"YAML
//! frontmatter" + Reference B. The Anthropic loader supports a much smaller
//! frontmatter shape than CyberOS's 33-field CCSM; this transpiler:
//!
//! 1. Copies the **portable** fields verbatim: `name`, `description`,
//!    `license`, `metadata` (custom), `allowed-tools` (renamed from
//!    CyberOS's `allowed_mcp_tools`).
//! 2. **Drops** CyberOS-only governance fields the Anthropic loader
//!    doesn't understand (audit, self_audit, human_fine_tune,
//!    depends_on_contracts, expects/produces, escalation, etc.).
//!    These fields don't affect Anthropic-side routing; CyberOS preserves
//!    them at source so the same skill remains governable at home.
//! 3. **Rejects** any frontmatter containing XML brackets (TASK-SKILL-113
//!    invariant — pre-transpile guard, since Anthropic's loader rejects
//!    them too).
//! 4. Preserves the Markdown body verbatim — the body is the prompt that
//!    Anthropic's loader feeds to the LLM at runtime.
//!
//! Output shape (per Anthropic guide p. 31 Reference B "All optional fields"):
//!
//! ```yaml
//! ---
//! name: <kebab-case>
//! description: <80-1024 chars; WHAT + WHEN + KEY VALUE per TASK-SKILL-111>
//! license: <license string from CyberOS source>
//! allowed-tools: <space-separated string from CyberOS allowed_mcp_tools>
//! metadata:
//!   <custom kv pairs from CyberOS metadata>
//! ---
//!
//! <body markdown, verbatim>
//! ```

use std::path::Path;

use crate::frontmatter::{load_and_validate, FrontmatterError, SkillFrontmatter};

/// Result of transpiling one CCSM SKILL.md to Anthropic-flat form.
#[derive(Debug, Clone)]
pub struct AnthropicSkill {
    pub name: String,
    pub frontmatter_yaml: String,
    pub body: String,
}

impl AnthropicSkill {
    /// The full SKILL.md content (frontmatter + body) as a single string.
    pub fn to_skill_md(&self) -> String {
        format!("---\n{}---\n{}", self.frontmatter_yaml, self.body)
    }
}

/// Transpile one CCSM bundle (modules/skill/<name>/SKILL.md) → Anthropic form.
///
/// Returns Err if the source SKILL.md fails CCSM validation. A failed
/// transpile means the source is broken — fix the source first.
pub fn transpile(bundle: &Path) -> Result<AnthropicSkill, FrontmatterError> {
    let (fm, body) = load_and_validate(bundle)?;
    let frontmatter_yaml = render_anthropic_frontmatter(&fm);
    Ok(AnthropicSkill {
        name: fm.name,
        frontmatter_yaml,
        body,
    })
}

/// Render the Anthropic-flat YAML frontmatter from a CCSM SkillFrontmatter.
fn render_anthropic_frontmatter(fm: &SkillFrontmatter) -> String {
    let mut out = String::new();

    // Required fields (per Anthropic guide p. 31 Reference B).
    out.push_str(&format!("name: {}\n", fm.name));

    // Description — preserve the YAML folded scalar form for readability.
    // Anthropic's parser handles `>-` blocks.
    let flat = fm.description.replace('\n', " ");
    let flat = flat.trim();
    if flat.len() > 80 {
        out.push_str("description: >-\n");
        // Wrap at 78 chars for YAML-pretty output
        let wrapped = wrap_text(flat, 78);
        for line in wrapped.lines() {
            out.push_str(&format!("  {line}\n"));
        }
    } else {
        out.push_str(&format!("description: {flat}\n"));
    }

    // Optional pass-through fields from `extras`. We extract only the
    // Anthropic-portable keys: license, allowed_mcp_tools (→ allowed-tools),
    // metadata. The rest are CyberOS-specific governance + dropped here.
    if let Some(serde_yaml::Value::String(s)) =
        fm.extras.get(serde_yaml::Value::String("license".into()))
    {
        out.push_str(&format!("license: {s}\n"));
    }

    let mut names = fm.allowed_mcp_tools.clone();
    names.extend(fm.allowed_tools.clone());
    if !names.is_empty() {
        // Anthropic format: space-separated string
        out.push_str(&format!("allowed-tools: {}\n", names.join(" ")));
    }

    if let Some(metadata) = fm.metadata.as_ref() {
        if metadata.version.is_some() || !metadata.extras.is_empty() {
            out.push_str("metadata:\n");
            if let Some(version) = metadata.version.as_ref() {
                out.push_str(&format!("  version: {version}\n"));
            }
            for (k, v) in &metadata.extras {
                if let serde_yaml::Value::String(key) = k {
                    match v {
                        serde_yaml::Value::String(s) => {
                            out.push_str(&format!("  {key}: {s}\n"));
                        }
                        serde_yaml::Value::Number(n) => {
                            out.push_str(&format!("  {key}: {n}\n"));
                        }
                        _ => {} // Skip nested structures — Anthropic metadata is flat KV.
                    }
                }
            }
        }
    }

    out
}

/// Word-wrap a single-line string at `width`, on whitespace boundaries.
fn wrap_text(text: &str, width: usize) -> String {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_text_short() {
        assert_eq!(wrap_text("hello", 78), "hello");
    }

    #[test]
    fn wrap_text_long() {
        let long = "the quick brown fox jumps over the lazy dog ".repeat(5);
        let wrapped = wrap_text(long.trim(), 30);
        for line in wrapped.lines() {
            assert!(line.len() <= 30 + 9, "line too long: {line:?}"); // +word-length slack
        }
    }

    #[test]
    fn transpile_synthesised_skill() {
        let tmp = tempfile::TempDir::new().unwrap();
        let skill_md = tmp.path().join("SKILL.md");
        std::fs::write(
            &skill_md,
            r#"---
name: synth-author
description: >-
  Generate a synthetic@1 markdown from a brief. Use when user asks to
  "draft a synth" or "compose the synthetic". Outputs versioned files.
license: Apache-2.0
metadata:
  version: 1.0.0
  module: skill
  stage: cross
allowed_mcp_tools:
  - kb.read
  - memory.search
  - audit.append
untrusted_inputs:
  wrap_in_marker: "untrusted_content"
  injection_scan: required
  on_marker_hit: surface_to_human
---
# Body

This is the skill's instructions.
"#,
        )
        .unwrap();

        let result = transpile(tmp.path()).expect("transpile failed");
        assert_eq!(result.name, "synth-author");
        assert!(result.frontmatter_yaml.contains("name: synth-author"));
        assert!(result.frontmatter_yaml.contains("description: >-"));
        assert!(result.frontmatter_yaml.contains("license: Apache-2.0"));
        assert!(result
            .frontmatter_yaml
            .contains("allowed-tools: kb.read memory.search audit.append"));
        // CyberOS governance fields MUST be dropped
        assert!(!result.frontmatter_yaml.contains("untrusted_inputs"));
        assert!(!result.frontmatter_yaml.contains("wrap_in_marker"));
        // Body MUST be preserved
        assert!(result.body.contains("This is the skill's instructions"));
    }

    #[test]
    fn transpile_rejects_legacy_xml_form() {
        let tmp = tempfile::TempDir::new().unwrap();
        let skill_md = tmp.path().join("SKILL.md");
        std::fs::write(
            &skill_md,
            r#"---
name: legacy-test
description: >-
  Some description that is at least 80 characters long. Use when user asks
  to "do A" or "do B". This is the value.
untrusted_inputs:
  wrap_in: <untrusted_content/>
---
"#,
        )
        .unwrap();
        let result = transpile(tmp.path());
        assert!(
            matches!(result, Err(FrontmatterError::DeprecatedXmlField { .. })),
            "expected DeprecatedXmlField; got {:?}",
            result.err()
        );
    }

    #[test]
    fn anthropic_skill_to_skill_md_reconstructs() {
        let skill = AnthropicSkill {
            name: "demo".into(),
            frontmatter_yaml: "name: demo\ndescription: A demo skill.\n".into(),
            body: "# Demo\nBody content.\n".into(),
        };
        let s = skill.to_skill_md();
        assert!(s.starts_with("---\n"));
        assert!(s.contains("name: demo"));
        assert!(s.contains("---\n# Demo"));
    }
}
