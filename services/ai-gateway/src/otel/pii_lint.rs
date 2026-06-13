//! Source gate that rejects unapproved OTel attribute keys.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::attributes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintFailure {
    pub path: PathBuf,
    pub line: usize,
    pub key: String,
}

pub fn lint_no_unknown_attribute_keys(root: &Path) -> Result<(), Vec<LintFailure>> {
    let mut failures = Vec::new();
    let allowed: BTreeSet<&'static str> = attributes::APPROVED_ATTRIBUTE_KEYS
        .iter()
        .copied()
        .collect();
    visit_rs_files(root, &mut |path| {
        if path.ends_with(Path::new("otel/pii_lint.rs")) {
            return;
        }
        let Ok(source) = std::fs::read_to_string(path) else {
            return;
        };
        for (idx, line) in source.lines().enumerate() {
            for key in extract_keyvalue_literals(line) {
                if !allowed.contains(key.as_str()) {
                    failures.push(LintFailure {
                        path: path.to_path_buf(),
                        line: idx + 1,
                        key,
                    });
                }
            }
        }
    });

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures)
    }
}

fn visit_rs_files(root: &Path, visitor: &mut impl FnMut(&Path)) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_rs_files(&path, visitor);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            visitor(&path);
        }
    }
}

fn extract_keyvalue_literals(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let marker = "KeyValue::new(";
    let mut rest = line;
    while let Some(offset) = rest.find(marker) {
        rest = &rest[offset + marker.len()..];
        let trimmed = rest.trim_start();
        let Some(after_quote) = trimmed.strip_prefix('"') else {
            continue;
        };
        let Some(end) = after_quote.find('"') else {
            continue;
        };
        out.push(after_quote[..end].to_string());
        rest = &after_quote[end + 1..];
    }
    out
}
