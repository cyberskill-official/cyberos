use crate::loader::Loader;
use crate::registry::{ActivatedSkill, SkillHeader, SkillRegistry};
use std::sync::Arc;

pub struct Activator {
    registry: Arc<SkillRegistry>,
    loader: Loader,
}

impl Activator {
    pub fn new(registry: Arc<SkillRegistry>, loader: Loader) -> Self {
        Self { registry, loader }
    }

    pub async fn activate(&self, header: &SkillHeader) -> anyhow::Result<Arc<ActivatedSkill>> {
        let body = self.loader.load_body(header).await?;
        let activated = Arc::new(ActivatedSkill {
            header: header.clone(),
            body,
            invocations: Default::default(),
            last_used_unix_ms: Default::default(),
            runtime: Default::default(),
        });
        Ok(activated)
    }
}
