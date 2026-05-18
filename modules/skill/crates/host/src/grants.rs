//! Content-hash-keyed capability grants.
//!
//! Persists in $HOME/.cyberos/grants.json (override via CYBEROS_GRANTS_PATH).
//! Schema:
//!   {
//!     "version": 1,
//!     "grants": {
//!       "<skill-name>": {
//!         "<sha256-of-skill-md>": {
//!           "granted_caps": ["read_file", "write_file"],
//!           "granted_at_unix_ms": 1715126400000,
//!           "operator": "stephen"
//!         }
//!       }
//!     }
//!   }
//!
//! Different content hash for the same skill name = re-approval required.

use crate::capabilities::Capability;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GrantsFile {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub grants: HashMap<String, HashMap<String, GrantEntry>>,
}

fn default_version() -> u32 { 1 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantEntry {
    pub granted_caps: Vec<String>,
    pub granted_at_unix_ms: u64,
    #[serde(default)]
    pub operator: String,
}

pub fn default_grants_path() -> PathBuf {
    if let Ok(p) = std::env::var("CYBEROS_GRANTS_PATH") {
        return PathBuf::from(p);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".cyberos").join("grants.json")
}

pub fn load(path: &Path) -> anyhow::Result<GrantsFile> {
    if !path.exists() {
        return Ok(GrantsFile::default());
    }
    let bytes = fs::read(path)?;
    let parsed: GrantsFile = serde_json::from_slice(&bytes)?;
    Ok(parsed)
}

pub fn save(path: &Path, grants: &GrantsFile) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let bytes = serde_json::to_vec_pretty(grants)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, &bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

pub fn record_grant(
    path: &Path,
    skill_name: &str,
    skill_md_sha256: &str,
    caps: &[Capability],
    operator: &str,
) -> anyhow::Result<()> {
    let mut g = load(path).unwrap_or_default();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
    let entry = GrantEntry {
        granted_caps: caps.iter().map(|c| c.to_string()).collect(),
        granted_at_unix_ms: now,
        operator: operator.to_owned(),
    };
    g.grants
        .entry(skill_name.to_owned())
        .or_default()
        .insert(skill_md_sha256.to_owned(), entry);
    save(path, &g)?;
    Ok(())
}

pub fn is_granted(
    path: &Path,
    skill_name: &str,
    skill_md_sha256: &str,
    required: &Capability,
) -> bool {
    let g = match load(path) {
        Ok(g) => g,
        Err(_) => return false,
    };
    let by_skill = match g.grants.get(skill_name) {
        Some(m) => m,
        None => return false,
    };
    let entry = match by_skill.get(skill_md_sha256) {
        Some(e) => e,
        None => return false,
    };
    let req = required.to_string();
    entry.granted_caps.iter().any(|granted| granted == &req || granted == &required.name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn fixture_path() -> (TempDir, PathBuf) {
        let td = TempDir::new().unwrap();
        let p = td.path().join("grants.json");
        (td, p)
    }

    #[test]
    fn round_trip_empty() {
        let (_t, p) = fixture_path();
        let g = load(&p).unwrap();
        assert!(g.grants.is_empty());
        save(&p, &g).unwrap();
        assert!(p.is_file());
    }

    #[test]
    fn record_then_load() {
        let (_t, p) = fixture_path();
        let cap = Capability { name: "read_file".to_owned(), argument: None };
        record_grant(&p, "vietnam-mst-validate", "abc123", &[cap.clone()], "stephen").unwrap();
        assert!(is_granted(&p, "vietnam-mst-validate", "abc123", &cap));
        // Wrong hash → not granted (force re-approval on edit)
        assert!(!is_granted(&p, "vietnam-mst-validate", "different", &cap));
        // Wrong skill → not granted
        assert!(!is_granted(&p, "other-skill", "abc123", &cap));
    }
}
