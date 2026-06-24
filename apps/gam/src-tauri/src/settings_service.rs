use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Persists app settings (e.g. theme) to a JSON file in the app data directory.
pub struct SettingsService {
    config_path: PathBuf,
    settings: HashMap<String, String>,
}

impl Default for SettingsService {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsService {
    pub fn new() -> Self {
        let config_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("com.github.zintaen.gam");

        if !config_dir.exists() {
            let _ = fs::create_dir_all(&config_dir);
        }

        let config_path = config_dir.join("settings.json");
        let mut service = Self {
            config_path,
            settings: HashMap::new(),
        };
        service.load();
        service
    }

    fn load(&mut self) {
        if let Ok(data) = fs::read_to_string(&self.config_path)
            && let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&data) {
                self.settings = map;
            }
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.settings) {
            let _ = fs::write(&self.config_path, json);
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.settings.get(key).cloned()
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.settings.insert(key.to_string(), value.to_string());
        self.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_settings() -> (SettingsService, PathBuf) {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("gam_test_settings_{}_{}", std::process::id(), id));
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let config_path = dir.join("settings.json");
        let service = SettingsService {
            config_path: config_path.clone(),
            settings: HashMap::new(),
        };
        (service, dir)
    }

    fn cleanup(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn get_returns_none_for_missing_key() {
        let (svc, dir) = temp_settings();
        assert!(svc.get("nonexistent").is_none());
        cleanup(&dir);
    }

    #[test]
    fn set_and_get_roundtrip() {
        let (mut svc, dir) = temp_settings();
        svc.set("theme", "cybercore-dark");
        assert_eq!(svc.get("theme"), Some("cybercore-dark".to_string()));
        cleanup(&dir);
    }

    #[test]
    fn set_overwrites_existing() {
        let (mut svc, dir) = temp_settings();
        svc.set("theme", "sketch-dark");
        svc.set("theme", "gothic-light");
        assert_eq!(svc.get("theme"), Some("gothic-light".to_string()));
        cleanup(&dir);
    }

    #[test]
    fn save_creates_json_file() {
        let (mut svc, dir) = temp_settings();
        svc.set("key", "value");
        assert!(svc.config_path.exists(), "Config file was not created at {:?}", svc.config_path);
        cleanup(&dir);
    }

    #[test]
    fn load_restores_saved_settings() {
        let dir = std::env::temp_dir().join(format!("gam_test_settings_load_{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let config_path = dir.join("settings.json");

        // Save
        {
            let mut svc = SettingsService {
                config_path: config_path.clone(),
                settings: HashMap::new(),
            };
            svc.set("theme", "pixel-dark");
        }

        // Load
        {
            let mut svc = SettingsService {
                config_path,
                settings: HashMap::new(),
            };
            svc.load();
            assert_eq!(svc.get("theme"), Some("pixel-dark".to_string()));
        }

        let _ = fs::remove_dir_all(&dir);
    }
}
