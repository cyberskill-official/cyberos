use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A named group for organizing aliases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasGroup {
    pub id: String,
    pub name: String,
    pub color: String,
}

/// Persisted data: groups + alias→group assignments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct GroupData {
    pub groups: Vec<AliasGroup>,
    /// Maps alias name → vec of group IDs.
    pub assignments: HashMap<String, Vec<String>>,
}


/// Manages alias groups stored in a JSON file in the app data directory.
pub struct GroupService {
    config_path: PathBuf,
    data: GroupData,
}

impl Default for GroupService {
    fn default() -> Self {
        Self::new()
    }
}

impl GroupService {
    pub fn new() -> Self {
        let config_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("com.github.zintaen.gam");

        if !config_dir.exists() {
            let _ = fs::create_dir_all(&config_dir);
        }

        let config_path = config_dir.join("groups.json");
        let mut service = Self {
            config_path,
            data: GroupData::default(),
        };
        service.load();
        service
    }

    fn load(&mut self) {
        if let Ok(content) = fs::read_to_string(&self.config_path)
            && let Ok(data) = serde_json::from_str::<GroupData>(&content) {
                self.data = data;
            }
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.data) {
            let _ = fs::write(&self.config_path, json);
        }
    }

    fn generate_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("g-{:x}", ts)
    }

    // ── Queries ──

    pub fn get_groups(&self) -> Vec<AliasGroup> {
        self.data.groups.clone()
    }

    pub fn get_alias_groups(&self, alias_name: &str) -> Vec<String> {
        self.data
            .assignments
            .get(alias_name)
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_all_assignments(&self) -> HashMap<String, Vec<String>> {
        self.data.assignments.clone()
    }

    pub fn get_data(&self) -> GroupData {
        self.data.clone()
    }

    // ── Mutations ──

    pub fn create_group(&mut self, name: &str, color: &str) -> AliasGroup {
        let group = AliasGroup {
            id: Self::generate_id(),
            name: name.to_string(),
            color: color.to_string(),
        };
        self.data.groups.push(group.clone());
        self.save();
        group
    }

    pub fn rename_group(&mut self, group_id: &str, new_name: &str) -> Result<(), String> {
        let group = self
            .data
            .groups
            .iter_mut()
            .find(|g| g.id == group_id)
            .ok_or_else(|| format!("Group not found: {}", group_id))?;
        group.name = new_name.to_string();
        self.save();
        Ok(())
    }

    pub fn set_group_color(&mut self, group_id: &str, color: &str) -> Result<(), String> {
        let group = self
            .data
            .groups
            .iter_mut()
            .find(|g| g.id == group_id)
            .ok_or_else(|| format!("Group not found: {}", group_id))?;
        group.color = color.to_string();
        self.save();
        Ok(())
    }

    pub fn delete_group(&mut self, group_id: &str) -> Result<(), String> {
        let before = self.data.groups.len();
        self.data.groups.retain(|g| g.id != group_id);
        if self.data.groups.len() == before {
            return Err(format!("Group not found: {}", group_id));
        }

        // Remove this group from all assignments
        for ids in self.data.assignments.values_mut() {
            ids.retain(|id| id != group_id);
        }
        // Clean up empty assignments
        self.data.assignments.retain(|_, ids| !ids.is_empty());

        self.save();
        Ok(())
    }

    pub fn set_alias_groups(
        &mut self,
        alias_name: &str,
        group_ids: Vec<String>,
    ) {
        if group_ids.is_empty() {
            self.data.assignments.remove(alias_name);
        } else {
            self.data
                .assignments
                .insert(alias_name.to_string(), group_ids);
        }
        self.save();
    }

    pub fn import_data(&mut self, incoming: GroupData) {
        // Merge groups (skip duplicates by ID)
        for group in incoming.groups {
            if !self.data.groups.iter().any(|g| g.id == group.id) {
                self.data.groups.push(group);
            }
        }
        // Merge assignments (incoming overrides)
        for (alias, ids) in incoming.assignments {
            self.data.assignments.insert(alias, ids);
        }
        self.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_group_service() -> (GroupService, PathBuf) {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir =
            std::env::temp_dir().join(format!("gam_test_groups_{}_{}", std::process::id(), id));
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let config_path = dir.join("groups.json");
        let service = GroupService {
            config_path,
            data: GroupData::default(),
        };
        (service, dir)
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn starts_with_empty_groups() {
        let (svc, dir) = temp_group_service();
        assert!(svc.get_groups().is_empty());
        cleanup(&dir);
    }

    #[test]
    fn create_group_returns_group() {
        let (mut svc, dir) = temp_group_service();
        let group = svc.create_group("Deployment", "#ff6b35");
        assert_eq!(group.name, "Deployment");
        assert_eq!(group.color, "#ff6b35");
        assert!(group.id.starts_with("g-"));
        cleanup(&dir);
    }

    #[test]
    fn create_group_persists() {
        let (mut svc, dir) = temp_group_service();
        svc.create_group("Review", "#22d3ee");
        assert_eq!(svc.get_groups().len(), 1);
        cleanup(&dir);
    }

    #[test]
    fn rename_group_updates_name() {
        let (mut svc, dir) = temp_group_service();
        let group = svc.create_group("Old", "#fff");
        svc.rename_group(&group.id, "New").unwrap();
        assert_eq!(svc.get_groups()[0].name, "New");
        cleanup(&dir);
    }

    #[test]
    fn rename_nonexistent_group_errors() {
        let (mut svc, dir) = temp_group_service();
        let result = svc.rename_group("nonexistent", "New");
        assert!(result.is_err());
        cleanup(&dir);
    }

    #[test]
    fn delete_group_removes_it() {
        let (mut svc, dir) = temp_group_service();
        let g1 = svc.create_group("A", "#aaa");
        let g2 = svc.create_group("B", "#bbb");
        svc.delete_group(&g1.id).unwrap();
        let groups = svc.get_groups();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].id, g2.id);
        cleanup(&dir);
    }

    #[test]
    fn delete_group_removes_assignments() {
        let (mut svc, dir) = temp_group_service();
        let g = svc.create_group("Deploy", "#f00");
        svc.set_alias_groups("co", vec![g.id.clone()]);
        svc.delete_group(&g.id).unwrap();
        assert!(svc.get_alias_groups("co").is_empty());
        cleanup(&dir);
    }

    #[test]
    fn set_and_get_alias_groups() {
        let (mut svc, dir) = temp_group_service();
        let g1 = svc.create_group("A", "#aaa");
        let g2 = svc.create_group("B", "#bbb");
        svc.set_alias_groups("co", vec![g1.id.clone(), g2.id.clone()]);
        let groups = svc.get_alias_groups("co");
        assert_eq!(groups.len(), 2);
        assert!(groups.contains(&g1.id));
        assert!(groups.contains(&g2.id));
        cleanup(&dir);
    }

    #[test]
    fn set_empty_groups_removes_assignment() {
        let (mut svc, dir) = temp_group_service();
        let g = svc.create_group("A", "#aaa");
        svc.set_alias_groups("co", vec![g.id.clone()]);
        svc.set_alias_groups("co", vec![]);
        assert!(svc.get_alias_groups("co").is_empty());
        cleanup(&dir);
    }

    #[test]
    fn get_all_assignments_returns_map() {
        let (mut svc, dir) = temp_group_service();
        let g = svc.create_group("A", "#aaa");
        svc.set_alias_groups("co", vec![g.id.clone()]);
        svc.set_alias_groups("st", vec![g.id.clone()]);
        let all = svc.get_all_assignments();
        assert_eq!(all.len(), 2);
        cleanup(&dir);
    }

    #[test]
    fn import_data_merges_groups() {
        let (mut svc, dir) = temp_group_service();
        svc.create_group("Existing", "#111");

        let incoming = GroupData {
            groups: vec![AliasGroup {
                id: "g-incoming".into(),
                name: "Imported".into(),
                color: "#222".into(),
            }],
            assignments: {
                let mut m = HashMap::new();
                m.insert("co".to_string(), vec!["g-incoming".to_string()]);
                m
            },
        };
        svc.import_data(incoming);
        assert_eq!(svc.get_groups().len(), 2);
        assert_eq!(svc.get_alias_groups("co"), vec!["g-incoming"]);
        cleanup(&dir);
    }

    #[test]
    fn import_data_skips_duplicate_ids() {
        let (mut svc, dir) = temp_group_service();
        let existing = svc.create_group("Original", "#111");

        let incoming = GroupData {
            groups: vec![AliasGroup {
                id: existing.id.clone(),
                name: "Duplicate".into(),
                color: "#999".into(),
            }],
            assignments: HashMap::new(),
        };
        svc.import_data(incoming);
        // Should not duplicate; still 1 group with original name
        assert_eq!(svc.get_groups().len(), 1);
        assert_eq!(svc.get_groups()[0].name, "Original");
        cleanup(&dir);
    }

    #[test]
    fn set_group_color_updates() {
        let (mut svc, dir) = temp_group_service();
        let g = svc.create_group("CI", "#aaa");
        svc.set_group_color(&g.id, "#00ff00").unwrap();
        assert_eq!(svc.get_groups()[0].color, "#00ff00");
        cleanup(&dir);
    }

    #[test]
    fn load_restores_persisted_data() {
        let id;
        let dir;
        let config_path;
        {
            let (mut svc, d) = temp_group_service();
            dir = d;
            config_path = svc.config_path.clone();
            let g = svc.create_group("Persisted", "#abc");
            id = g.id;
            svc.set_alias_groups("st", vec![id.clone()]);
        }
        // Create a new service instance pointing to the same file
        let mut svc2 = GroupService {
            config_path,
            data: GroupData::default(),
        };
        svc2.load();
        assert_eq!(svc2.get_groups().len(), 1);
        assert_eq!(svc2.get_groups()[0].name, "Persisted");
        assert_eq!(svc2.get_alias_groups("st"), vec![id]);
        cleanup(&dir);
    }
}
