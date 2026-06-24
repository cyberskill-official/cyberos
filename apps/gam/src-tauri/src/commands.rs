use std::path::Path;

use serde::{Deserialize, Serialize};
use tauri::State;
use tauri_plugin_dialog::DialogExt;

use crate::file_service::FileService;
use crate::git_service::{GitAlias, ValidationResult};
use crate::AppState;

/// Standard IPC result returned by all Tauri commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResult<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T: Serialize> IpcResult<T> {
    fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn err(msg: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg),
        }
    }
}

// ── Alias CRUD ──────────────────────────────────────────────

#[tauri::command]
pub fn get_aliases(state: State<'_, AppState>, scope: String) -> IpcResult<Vec<GitAlias>> {
    let mut git = state.git_service.write().unwrap_or_else(|e| e.into_inner());
    match git.get_aliases(&scope) {
        Ok(aliases) => IpcResult::ok(aliases),
        Err(e) => IpcResult::err(e),
    }
}

#[tauri::command]
pub fn add_alias(
    state: State<'_, AppState>,
    name: String,
    command: String,
    scope: String,
    local_path: Option<String>,
) -> IpcResult<bool> {
    let mut git = state.git_service.write().unwrap_or_else(|e| e.into_inner());
    match git.add_alias(&name, &command, &scope, local_path.as_deref()) {
        Ok(()) => IpcResult::ok(true),
        Err(e) => IpcResult::err(e),
    }
}

#[tauri::command]
pub fn update_alias(
    state: State<'_, AppState>,
    old_name: String,
    name: String,
    command: String,
    scope: String,
    local_path: Option<String>,
) -> IpcResult<bool> {
    let mut git = state.git_service.write().unwrap_or_else(|e| e.into_inner());
    match git.update_alias(&old_name, &name, &command, &scope, local_path.as_deref()) {
        Ok(()) => IpcResult::ok(true),
        Err(e) => IpcResult::err(e),
    }
}

#[tauri::command]
pub fn delete_alias(
    state: State<'_, AppState>,
    name: String,
    scope: String,
    local_path: Option<String>,
) -> IpcResult<bool> {
    let mut git = state.git_service.write().unwrap_or_else(|e| e.into_inner());
    match git.delete_alias(&name, &scope, local_path.as_deref()) {
        Ok(()) => IpcResult::ok(true),
        Err(e) => IpcResult::err(e),
    }
}

// ── Validation ──────────────────────────────────────────────

#[tauri::command]
pub fn validate_command(
    state: State<'_, AppState>,
    command: String,
) -> IpcResult<ValidationResult> {
    let git = state.git_service.read().unwrap_or_else(|e| e.into_inner());
    IpcResult::ok(git.validate_command(&command))
}

// ── Local Scope Folder Selection ─────────────────────────────

#[tauri::command]
pub async fn select_folder(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<IpcResult<String>, String> {


    let file_path = app
        .dialog()
        .file()
        .set_title("Select Git Repository")
        .blocking_pick_folder();

    match file_path {
        Some(path) => {
            let path_str = path.into_path().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
            let mut git = state.git_service.write().unwrap_or_else(|e| e.into_inner());
            git.set_local_path(Some(path_str.clone()));
            Ok(IpcResult::ok(path_str))
        }
        None => {
            let git = state.git_service.read().unwrap_or_else(|e| e.into_inner());
            let current = git.get_local_path().unwrap_or_default();
            Ok(IpcResult::ok(current))
        }
    }
}

#[tauri::command]
pub fn get_local_path(state: State<'_, AppState>) -> IpcResult<String> {
    let git = state.git_service.read().unwrap_or_else(|e| e.into_inner());
    IpcResult::ok(git.get_local_path().unwrap_or_default())
}

#[tauri::command]
pub fn set_local_path(
    state: State<'_, AppState>,
    path: String,
) -> IpcResult<String> {
    if !path.is_empty() && !Path::new(&path).exists() {
        return IpcResult::err(format!("Directory does not exist: {}", path));
    }
    let mut git = state.git_service.write().unwrap_or_else(|e| e.into_inner());
    git.set_local_path(if path.is_empty() { None } else { Some(path.clone()) });
    IpcResult::ok(path)
}

// ── Import / Export ─────────────────────────────────────────

#[tauri::command]
pub async fn export_aliases(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    aliases: Vec<GitAlias>,
) -> Result<IpcResult<String>, String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let default_name = format!("gam-{}.json", now);

    let file_path = app
        .dialog()
        .file()
        .set_title("Export Git Aliases")
        .add_filter("JSON Files", &["json"])
        .add_filter("All Files", &["*"])
        .set_file_name(&default_name)
        .blocking_save_file();

    match file_path {
        Some(path) => {
            let path_str = path.into_path().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
            let group_data = state.group_service.read().unwrap_or_else(|e| e.into_inner()).get_data();
            match FileService::export_aliases(&aliases, &path_str, Some(&group_data)) {
                Ok(p) => Ok(IpcResult::ok(p)),
                Err(e) => Ok(IpcResult::err(e)),
            }
        }
        None => Ok(IpcResult::err("Export cancelled".to_string())),
    }
}

#[tauri::command]
pub async fn import_aliases(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<IpcResult<Vec<GitAlias>>, String> {
    let file_path = app
        .dialog()
        .file()
        .set_title("Import Git Aliases")
        .add_filter("JSON Files", &["json"])
        .add_filter("All Files", &["*"])
        .blocking_pick_file();

    match file_path {
        Some(path) => {
            let path_str = path.into_path().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();

            // Also import group data if present in the file
            if let Ok(content) = std::fs::read_to_string(&path_str)
                && let Ok(export_data) = serde_json::from_str::<crate::file_service::ExportData>(&content)
                    && (export_data.groups.is_some() || export_data.assignments.is_some()) {
                        let incoming = crate::group_service::GroupData {
                            groups: export_data.groups.unwrap_or_default(),
                            assignments: export_data.assignments.unwrap_or_default(),
                        };
                        let mut group_svc = state.group_service.write().unwrap_or_else(|e| e.into_inner());
                        group_svc.import_data(incoming);
                    }

            match FileService::import_aliases(&path_str) {
                Ok(aliases) => Ok(IpcResult::ok(aliases)),
                Err(e) => Ok(IpcResult::err(e)),
            }
        }
        None => Ok(IpcResult::err("Import cancelled".to_string())),
    }
}

// ── Open folder / URL ──────────────────────────────────────

#[tauri::command]
pub fn open_local_folder(path: String) -> IpcResult<bool> {
    let p = Path::new(&path);
    if !p.exists() || !p.is_dir() {
        return IpcResult::err(format!("Not a valid directory: {}", path));
    }
    match open::that(&path) {
        Ok(()) => IpcResult::ok(true),
        Err(e) => IpcResult::err(format!("Failed to open folder: {}", e)),
    }
}

#[tauri::command]
pub fn open_external(url: String) -> IpcResult<bool> {
    // Only allow HTTPS URLs — block file://, ssh://, custom protocols
    if !url.starts_with("https://") {
        return IpcResult::err("Only HTTPS URLs are allowed".to_string());
    }
    match open::that(&url) {
        Ok(()) => IpcResult::ok(true),
        Err(e) => IpcResult::err(format!("Failed to open URL: {}", e)),
    }
}

// ── Theme settings ─────────────────────────────────────────

#[tauri::command]
pub fn get_theme(state: State<'_, AppState>) -> IpcResult<String> {
    let settings = state.settings_service.read().unwrap_or_else(|e| e.into_inner());
    let theme = settings
        .get("theme")
        .unwrap_or_else(|| "glassmorphism-dark".to_string());
    IpcResult::ok(theme)
}

#[tauri::command]
pub fn set_theme(state: State<'_, AppState>, theme_id: String) -> IpcResult<bool> {
    let mut settings = state.settings_service.write().unwrap_or_else(|e| e.into_inner());
    settings.set("theme", &theme_id);
    IpcResult::ok(true)
}

// ── History-ranking consent ────────────────────────────────

#[tauri::command]
pub fn get_history_ranking_enabled(state: State<'_, AppState>) -> IpcResult<bool> {
    let git = state.git_service.read().unwrap_or_else(|e| e.into_inner());
    IpcResult::ok(git.is_history_ranking_enabled())
}

#[tauri::command]
pub fn set_history_ranking_enabled(state: State<'_, AppState>, enabled: bool) -> IpcResult<bool> {
    // Persist the choice, then apply it to the live ranking service.
    {
        let mut settings = state.settings_service.write().unwrap_or_else(|e| e.into_inner());
        settings.set("historyRankingEnabled", &enabled.to_string());
    }
    let mut git = state.git_service.write().unwrap_or_else(|e| e.into_inner());
    git.set_history_ranking_enabled(enabled);
    IpcResult::ok(enabled)
}

// ── Group management ───────────────────────────────────────

#[tauri::command]
pub fn get_groups(state: State<'_, AppState>) -> IpcResult<Vec<crate::group_service::AliasGroup>> {
    let group_svc = state.group_service.read().unwrap_or_else(|e| e.into_inner());
    IpcResult::ok(group_svc.get_groups())
}

#[tauri::command]
pub fn create_group(
    state: State<'_, AppState>,
    name: String,
    color: String,
) -> IpcResult<crate::group_service::AliasGroup> {
    let mut group_svc = state.group_service.write().unwrap_or_else(|e| e.into_inner());
    IpcResult::ok(group_svc.create_group(&name, &color))
}

#[tauri::command]
pub fn rename_group(
    state: State<'_, AppState>,
    group_id: String,
    new_name: String,
) -> IpcResult<bool> {
    let mut group_svc = state.group_service.write().unwrap_or_else(|e| e.into_inner());
    match group_svc.rename_group(&group_id, &new_name) {
        Ok(()) => IpcResult::ok(true),
        Err(e) => IpcResult::err(e),
    }
}

#[tauri::command]
pub fn set_group_color(
    state: State<'_, AppState>,
    group_id: String,
    color: String,
) -> IpcResult<bool> {
    let mut group_svc = state.group_service.write().unwrap_or_else(|e| e.into_inner());
    match group_svc.set_group_color(&group_id, &color) {
        Ok(()) => IpcResult::ok(true),
        Err(e) => IpcResult::err(e),
    }
}

#[tauri::command]
pub fn delete_group(state: State<'_, AppState>, group_id: String) -> IpcResult<bool> {
    let mut group_svc = state.group_service.write().unwrap_or_else(|e| e.into_inner());
    match group_svc.delete_group(&group_id) {
        Ok(()) => IpcResult::ok(true),
        Err(e) => IpcResult::err(e),
    }
}

#[tauri::command]
pub fn set_alias_groups(
    state: State<'_, AppState>,
    alias_name: String,
    group_ids: Vec<String>,
) -> IpcResult<bool> {
    let mut group_svc = state.group_service.write().unwrap_or_else(|e| e.into_inner());
    group_svc.set_alias_groups(&alias_name, group_ids);
    IpcResult::ok(true)
}

#[tauri::command]
pub fn get_all_group_assignments(
    state: State<'_, AppState>,
) -> IpcResult<std::collections::HashMap<String, Vec<String>>> {
    let group_svc = state.group_service.read().unwrap_or_else(|e| e.into_inner());
    IpcResult::ok(group_svc.get_all_assignments())
}
