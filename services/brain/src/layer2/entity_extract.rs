//! Pulls entities out of memory bodies for the l2_entity projection.
//!
//! First-cut implementation: regex-based scan for canonical mention patterns
//! (`@<handle>` for people, `#<slug>` for projects/decisions, `[[link]]` for
//! cross-doc refs). The Phase-3 enhancement uses an embedding model to
//! cluster mentions into canonical entity IDs.

use regex::Regex;
use std::sync::OnceLock;

/// An extracted entity ready for upsert into l2_entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedEntity {
    pub kind: String,         // 'person' | 'project' | 'decision' | 'doc'
    pub name: String,
    pub source_seq: i64,
    pub source_path: String,
}

static PERSON_RE: OnceLock<Regex> = OnceLock::new();
static SLUG_RE: OnceLock<Regex> = OnceLock::new();
static LINK_RE: OnceLock<Regex> = OnceLock::new();

fn person_re() -> &'static Regex {
    PERSON_RE.get_or_init(|| Regex::new(r"@([A-Za-z0-9_.-]{1,38})").unwrap())
}
fn slug_re() -> &'static Regex {
    SLUG_RE.get_or_init(|| Regex::new(r"#([a-z0-9][a-z0-9-]{0,40})").unwrap())
}
fn link_re() -> &'static Regex {
    LINK_RE.get_or_init(|| Regex::new(r"\[\[([^\]\n]{1,80})\]\]").unwrap())
}

/// Run all extractors on `body`. Order-stable and idempotent.
pub fn extract(seq: i64, path: &str, body: &str) -> Vec<ExtractedEntity> {
    let mut out: Vec<ExtractedEntity> = Vec::new();
    let mut seen: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();

    for cap in person_re().captures_iter(body) {
        let name = cap[1].to_string();
        if seen.insert(("person".into(), name.clone())) {
            out.push(ExtractedEntity {
                kind: "person".into(),
                name,
                source_seq: seq,
                source_path: path.to_string(),
            });
        }
    }
    for cap in slug_re().captures_iter(body) {
        let name = cap[1].to_string();
        // Heuristic: slugs starting with a year prefix → decision; else project.
        let kind = if name.starts_with("dec-") || name.starts_with("decision-") {
            "decision".to_string()
        } else {
            "project".to_string()
        };
        if seen.insert((kind.clone(), name.clone())) {
            out.push(ExtractedEntity {
                kind,
                name,
                source_seq: seq,
                source_path: path.to_string(),
            });
        }
    }
    for cap in link_re().captures_iter(body) {
        let name = cap[1].to_string();
        if seen.insert(("doc".into(), name.clone())) {
            out.push(ExtractedEntity {
                kind: "doc".into(),
                name,
                source_seq: seq,
                source_path: path.to_string(),
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_people_via_at_handle() {
        let ents = extract(1, "memo.md", "Met with @stephen and @lan-tran today.");
        assert!(ents.iter().any(|e| e.kind == "person" && e.name == "stephen"));
        assert!(ents.iter().any(|e| e.kind == "person" && e.name == "lan-tran"));
    }

    #[test]
    fn extracts_projects_and_decisions() {
        let ents = extract(2, "log.md", "Working on #cyberos-wave1; locked #dec-070 today.");
        assert!(ents.iter().any(|e| e.kind == "project" && e.name == "cyberos-wave1"));
        assert!(ents.iter().any(|e| e.kind == "decision" && e.name == "dec-070"));
    }

    #[test]
    fn extracts_wiki_links_as_docs() {
        let ents = extract(3, "spec.md", "Per [[BRAIN_AUTOSYNC_DESIGN]] §3.");
        assert!(ents.iter().any(|e| e.kind == "doc" && e.name == "BRAIN_AUTOSYNC_DESIGN"));
    }

    #[test]
    fn dedupes_repeated_mentions() {
        let ents = extract(4, "x.md", "@stephen @stephen @stephen");
        let stephen = ents.iter().filter(|e| e.name == "stephen").count();
        assert_eq!(stephen, 1, "duplicate mentions in one body must collapse");
    }
}
