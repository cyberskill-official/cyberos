use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Persists known repo paths to a JSON file in the app data directory.
pub struct KnownReposService {
    config_path: PathBuf,
    known_paths: HashSet<String>,
}

impl Default for KnownReposService {
    fn default() -> Self {
        Self::new()
    }
}

impl KnownReposService {
    pub fn new() -> Self {
        let config_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("com.github.zintaen.gam");

        if !config_dir.exists() {
            let _ = fs::create_dir_all(&config_dir);
        }

        let config_path = config_dir.join("known-repos.json");
        let mut service = Self {
            config_path,
            known_paths: HashSet::new(),
        };
        service.load();
        service
    }

    fn load(&mut self) {
        if let Ok(data) = fs::read_to_string(&self.config_path)
            && let Ok(paths) = serde_json::from_str::<Vec<String>>(&data) {
                // Filter out paths that no longer exist
                self.known_paths = paths
                    .into_iter()
                    .filter(|p| Path::new(p).exists())
                    .collect();
            }
    }

    fn save(&self) {
        let paths: Vec<&String> = self.known_paths.iter().collect();
        if let Ok(json) = serde_json::to_string_pretty(&paths) {
            let _ = fs::write(&self.config_path, json);
        }
    }

    pub fn add(&mut self, repo_path: &str) {
        if self.known_paths.insert(repo_path.to_string()) {
            self.save();
        }
    }

    pub fn get_all(&self) -> Vec<String> {
        self.known_paths.iter().cloned().collect()
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, repo_path: &str) {
        if self.known_paths.remove(repo_path) {
            self.save();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_service() -> (KnownReposService, PathBuf) {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("gam_test_repos_{}_{}", std::process::id(), id));
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let config_path = dir.join("known-repos.json");
        let service = KnownReposService {
            config_path: config_path.clone(),
            known_paths: HashSet::new(),
        };
        (service, dir)
    }

    fn cleanup(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn new_service_has_empty_paths() {
        let (svc, dir) = temp_service();
        assert!(svc.get_all().is_empty());
        cleanup(&dir);
    }

    #[test]
    fn add_stores_path() {
        let (mut svc, dir) = temp_service();
        svc.add("/tmp/repo1");
        assert!(svc.get_all().contains(&"/tmp/repo1".to_string()));
        cleanup(&dir);
    }

    #[test]
    fn add_deduplicates() {
        let (mut svc, dir) = temp_service();
        svc.add("/tmp/repo1");
        svc.add("/tmp/repo1");
        let all = svc.get_all();
        assert_eq!(all.iter().filter(|p| *p == "/tmp/repo1").count(), 1);
        cleanup(&dir);
    }

    #[test]
    fn remove_deletes_path() {
        let (mut svc, dir) = temp_service();
        svc.add("/tmp/repo1");
        svc.add("/tmp/repo2");
        svc.remove("/tmp/repo1");
        let all = svc.get_all();
        assert!(!all.contains(&"/tmp/repo1".to_string()));
        assert!(all.contains(&"/tmp/repo2".to_string()));
        cleanup(&dir);
    }

    #[test]
    fn remove_nonexistent_is_noop() {
        let (mut svc, dir) = temp_service();
        svc.remove("/tmp/nonexistent");
        assert!(svc.get_all().is_empty());
        cleanup(&dir);
    }

    #[test]
    fn save_creates_json_file() {
        let (mut svc, dir) = temp_service();
        svc.add("/tmp/repo1");
        assert!(svc.config_path.exists(), "Config file was not created at {:?}", svc.config_path);
        cleanup(&dir);
    }
}
