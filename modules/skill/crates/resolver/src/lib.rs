//! Skill resolver — locates `.skill` bundles from local cache, OCI
//! registry, or HTTPS URL. Phase 1 implements local cache only.

use std::path::{Path, PathBuf};

pub trait Resolver {
    fn resolve(&self, name: &str) -> anyhow::Result<PathBuf>;
}

pub struct LocalResolver {
    pub root: PathBuf,
}

impl LocalResolver {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self { root: root.as_ref().to_path_buf() }
    }
}

impl Resolver for LocalResolver {
    fn resolve(&self, name: &str) -> anyhow::Result<PathBuf> {
        let candidate = self.root.join(name);
        if candidate.join("SKILL.md").is_file() {
            Ok(candidate)
        } else {
            anyhow::bail!("skill '{}' not found under {}", name, self.root.display())
        }
    }
}
