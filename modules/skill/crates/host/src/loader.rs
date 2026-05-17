use crate::registry::{SkillHeader, SkillRegistry};
use cyberos_skill_manifest::{parse_frontmatter, validate_manifest};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::task::JoinSet;
use tracing::{debug, warn};

/// Directory names the loader SHALL NOT descend into when scanning for
/// SKILL.md files. Reflects the 2026-05-17 flat-layout rebuild — skill bundles
/// live at `skill/<name>/`, alongside infra/admin directories that contain no
/// shippable skills. Keeping the skip-list narrow + named (not glob) avoids
/// accidentally swallowing future top-level skill folders.
const EXCLUDED_DIR_NAMES: &[&str] = &[
    "crates",         // Rust workspace
    "toolchain",      // Bun + esbuild authoring toolchain
    "runners",        // legacy Python parity runners
    "tools",          // skill registry + build helpers
    "tests",          // parity + correctness tests
    "tours",          // .tour files
    "docs",           // protocol spec + module docs
    "contracts",      // artefact schemas (template.md, not SKILL.md)
    "_template",      // canonical scaffold (placeholder SKILL.md files would fail validation)
    "_retired",       // Phase-7 soak holding dir
    "_deprecated",    // alternate soak holding dir
    "target",         // cargo build output
    "node_modules",   // bun/npm install dir
    ".git",
    ".github",        // GH workflows + issue templates (e.g. under skill/public/)
    ".cyberos-memory",
    // NOTE: `public` is NOT excluded. The 2026-05-17 rebuild absorbed the
    // legacy public-skills/ tree into skill/public/<vn-name>/. Those 5 VN
    // skill bundles ship a real SKILL.md and SHOULD be discovered by the
    // loader. The walkdir's per-entry dir-name == manifest.name check filters
    // out non-bundle subdirs (skill/public/announcements/, .github/ above).
];

fn should_skip(entry: &walkdir::DirEntry) -> bool {
    if !entry.file_type().is_dir() {
        return false;
    }
    let name = entry.file_name().to_string_lossy();
    EXCLUDED_DIR_NAMES.iter().any(|excluded| name == *excluded)
}

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
            // walkdir + filter_entry skips both the excluded dirs themselves
            // AND everything below them, so the loader does not waste cycles
            // recursing into crates/, contracts/, _template/, etc.
            let walker = walkdir::WalkDir::new(root)
                .min_depth(1)
                .max_depth(6)
                .into_iter()
                .filter_entry(|e| !should_skip(e));
            for entry in walker {
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
