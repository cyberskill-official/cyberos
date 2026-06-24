use std::sync::RwLock;

mod commands;
mod error;
mod file_service;
mod git_service;
mod group_service;
mod known_repos_service;
mod ranking_service;
mod settings_service;

pub use commands::*;
pub use git_service::GitService;
pub use group_service::GroupService;
pub use known_repos_service::KnownReposService;
pub use ranking_service::RankingService;
pub use settings_service::SettingsService;

/// Shared application state managed by Tauri.
pub struct AppState {
    pub git_service: RwLock<GitService>,
    pub settings_service: RwLock<SettingsService>,
    pub group_service: RwLock<GroupService>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let start = std::time::Instant::now();

    // Log panics to crash.log in the app data directory
    let crash_log_path = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("com.github.zintaen.gam")
        .join("crash.log");

    std::panic::set_hook(Box::new(move |info| {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let msg = format!("[{}] PANIC: {}\n", timestamp, info);
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&crash_log_path)
            .and_then(|mut f| std::io::Write::write_all(&mut f, msg.as_bytes()));
        eprintln!("{}", msg);
    }));

    // Evaluate CLI argument for folder
    let local_path: Option<String> = {
        let args: Vec<String> = std::env::args().skip(1).collect();
        args.iter()
            .find(|a| !a.starts_with('-') && !a.ends_with(".js") && !a.ends_with(".ts") && *a != ".")
            .map(|a| {
                std::path::Path::new(a)
                    .canonicalize()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| a.clone())
            })
            .or_else(|| {
                args.iter()
                    .find(|a| *a == ".")
                    .map(|_| std::env::current_dir().unwrap().to_string_lossy().to_string())
            })
    };

    let settings_service = SettingsService::new();
    // Load the persisted history-ranking consent choice (default: enabled).
    let history_ranking_enabled = settings_service
        .get("historyRankingEnabled")
        .map(|v| v != "false")
        .unwrap_or(true);

    let mut git_service = GitService::new();
    if let Some(ref path) = local_path {
        git_service.set_local_path(Some(path.clone()));
    }
    git_service.set_history_ranking_enabled(history_ranking_enabled);

    tauri::Builder::default()
        .setup(move |app| {
            #[cfg(desktop)]
            {
                app.handle()
                    .plugin(tauri_plugin_updater::Builder::new().build())?;
                app.handle().plugin(tauri_plugin_process::init())?;
            }
            eprintln!("[GAM] Startup completed in {:?}", start.elapsed());
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            git_service: RwLock::new(git_service),
            settings_service: RwLock::new(settings_service),
            group_service: RwLock::new(GroupService::new()),
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_aliases,
            commands::add_alias,
            commands::update_alias,
            commands::delete_alias,
            commands::validate_command,
            commands::export_aliases,
            commands::import_aliases,
            commands::select_folder,
            commands::get_local_path,
            commands::set_local_path,
            commands::open_local_folder,
            commands::open_external,
            commands::get_theme,
            commands::set_theme,
            commands::get_history_ranking_enabled,
            commands::set_history_ranking_enabled,
            commands::get_groups,
            commands::create_group,
            commands::rename_group,
            commands::set_group_color,
            commands::delete_group,
            commands::set_alias_groups,
            commands::get_all_group_assignments,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
