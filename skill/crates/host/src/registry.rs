use cyberos_skill_manifest::SkillManifest;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SkillHeader {
    pub manifest: SkillManifest,
    pub skill_dir: PathBuf,
    pub body_offset: usize,
    pub file_size: u64,
}

#[derive(Debug)]
pub struct ActivatedSkill {
    pub header: SkillHeader,
    pub body: String,
    pub invocations: AtomicU64,
    pub last_used_unix_ms: AtomicU64,
    pub runtime: RwLock<RuntimeFlags>,
}

#[derive(Debug, Default)]
pub struct RuntimeFlags {
    pub revoked: bool,
    pub note: Option<String>,
}

pub struct SkillRegistry {
    headers: DashMap<String, Arc<SkillHeader>>,
    activated: DashMap<String, Arc<ActivatedSkill>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            headers: DashMap::with_shard_amount(64),
            activated: DashMap::with_shard_amount(64),
        }
    }

    pub fn insert_header(&self, header: SkillHeader) {
        self.headers.insert(header.manifest.name.clone(), Arc::new(header));
    }

    pub fn get_header(&self, name: &str) -> Option<Arc<SkillHeader>> {
        self.headers.get(name).map(|e| Arc::clone(e.value()))
    }

    pub fn count(&self) -> usize {
        self.headers.len()
    }

    pub fn header_summaries(&self) -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = self.headers
            .iter()
            .map(|e| (e.manifest.name.clone(), e.manifest.description.clone()))
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    pub fn all_headers(&self) -> Vec<Arc<SkillHeader>> {
        let mut out: Vec<_> = self.headers.iter().map(|e| Arc::clone(e.value())).collect();
        out.sort_by(|a, b| a.manifest.name.cmp(&b.manifest.name));
        out
    }
}

impl Default for SkillRegistry {
    fn default() -> Self { Self::new() }
}
