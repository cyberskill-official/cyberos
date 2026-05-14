use crate::registry::{SkillHeader, SkillRegistry};
use cyberos_skill_manifest::{parse_frontmatter, validate_manifest};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::task::JoinSet;
use tracing::{debug, warn};

#[derive(Clone)]
pub struct Loader {
    registry: Arc<SkillRegistry>,
}

impl Loader {
    pub fn new(registry: Arc<SkillRegistry>) -> Self { Self { registry } }

    pub async fn index_roots(&self, roots: &[PathBuf]) -> anyhow::Result<usize> {
        let mut skill_dirs: Vec<PathBuf> = Vec::new();
        for root in roots {
            if !root.is_dir() {
                debug!(?root, "skill root missing, skipping");
                continue;
            }
            for entry in walkdir::WalkDir::new(root).min_depth(1).max_depth(6) {
                let entry = entry?;
                if entry.file_name() == "SKILL.md" {
                    skill_dirs.push(entry.path().parent().unwrap().to_path_buf());
                }
            }
        }

        let mut tasks: JoinSet<anyhow::Result<SkillHeader>> = JoinSet::new();
        for dir in skill_dirs {
            tasks.spawn(async move { Self::index_one(&dir).await });
        }

        let mut count = 0usize;
        while let Some(joined) = tasks.join_next().await {
            match joined {
                Ok(Ok(header)) => {
                    self.registry.insert_header(header);
                    count += 1;
                }
                Ok(Err(e)) => warn!(error = %e, "skipping skill"),
                Err(e) => warn!(error = %e, "loader task panicked"),
            }
        }
        Ok(count)
    }

    async fn index_one(dir: &Path) -> anyhow::Result<SkillHeader> {
        let skill_md = dir.join("SKILL.md");
        let bytes = fs::read(&skill_md).await?;
        let (manifest, body_offset) = parse_frontmatter(&bytes)?;
        validate_manifest(&manifest)?;
        // Directory name must match manifest.name per spec.
        let dir_name = dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if dir_name != manifest.name {
            anyhow::bail!(
                "directory name '{}' must match SKILL.md name '{}'",
                dir_name, manifest.name
            );
        }
        Ok(SkillHeader {
            manifest,
            skill_dir: dir.to_path_buf(),
            body_offset,
            file_size: bytes.len() as u64,
        })
    }

    pub async fn load_body(&self, header: &SkillHeader) -> anyhow::Result<String> {
        let bytes = fs::read(header.skill_dir.join("SKILL.md")).await?;
        let body = std::str::from_utf8(&bytes[header.body_offset..])?.to_owned();
        Ok(body)
    }
}
