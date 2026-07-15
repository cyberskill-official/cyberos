//! TASK-APP-001 — CyberOS operations from the desktop UI: build the distributable payload,
//! list candidate projects, check installed-vs-available, and init/update a project.
//!
//! Doctrine (§1 #2): every operation shells out to the canonical `tools/install`
//! scripts — the UI never reimplements init logic, so UI and CLI cannot diverge.
//! Blocking process calls run on the blocking pool (`spawn_blocking`), never on the
//! async runtime threads. Output (stdout+stderr) is returned verbatim to the UI and a
//! non-zero exit surfaces as `ok: false` (§1 #3).

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpsSettings {
    /// Absolute path of the CyberOS checkout the operations run against.
    pub checkout: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct OpResult {
    pub ok: bool,
    pub output: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ProjectInfo {
    pub path: String,
    pub name: String,
    /// Contents of `<project>/.cyberos/VERSION` when the project is already initialised.
    pub installed_version: Option<String>,
}

// ── settings (persisted at ~/.cyberos/desktop-ops.json) ─────────────────────

fn settings_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cyberos")
        .join("desktop-ops.json")
}

fn default_checkout() -> String {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Projects/CyberSkill/cyberos")
        .to_string_lossy()
        .into_owned()
}

#[tauri::command]
pub async fn ops_get_settings() -> Result<OpsSettings, String> {
    match std::fs::read(settings_path()) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| format!("settings parse: {e}")),
        Err(_) => Ok(OpsSettings { checkout: default_checkout() }),
    }
}

#[tauri::command]
pub async fn ops_set_settings(settings: OpsSettings) -> Result<(), String> {
    let p = settings_path();
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("create settings dir: {e}"))?;
    }
    let body = serde_json::to_vec_pretty(&settings).map_err(|e| format!("settings encode: {e}"))?;
    std::fs::write(&p, body).map_err(|e| format!("settings write: {e}"))
}

// ── validation (§1 #4, #5) ──────────────────────────────────────────────────

fn require_checkout(checkout: &str) -> Result<PathBuf, String> {
    let root = PathBuf::from(checkout);
    if !root.join("tools/install/build.sh").is_file() {
        return Err(format!(
            "not a CyberOS checkout (missing tools/install/build.sh): {checkout}"
        ));
    }
    Ok(root)
}

fn require_payload(root: &Path) -> Result<PathBuf, String> {
    let init = root.join("dist/cyberos/init.sh");
    if !init.is_file() {
        return Err("payload not built yet — run \"Build payload\" first (dist/cyberos/init.sh missing)".into());
    }
    Ok(init)
}

fn require_project(root: &Path, project: &str) -> Result<PathBuf, String> {
    let p = PathBuf::from(project);
    if !p.join(".git").exists() {
        return Err(format!("not a git repository: {project}"));
    }
    let canon_root = root.canonicalize().map_err(|e| format!("checkout path: {e}"))?;
    let canon_p = p.canonicalize().map_err(|e| format!("project path: {e}"))?;
    if canon_p == canon_root {
        return Err("refusing to init the CyberOS checkout itself (§1 #5)".into());
    }
    Ok(canon_p)
}

// ── process runner ──────────────────────────────────────────────────────────

async fn run_bash(script: PathBuf, args: Vec<String>, cwd: PathBuf) -> Result<OpResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let out = Command::new("bash")
            .arg(&script)
            .args(&args)
            .current_dir(&cwd)
            .output()
            .map_err(|e| format!("spawn bash {}: {e}", script.display()))?;
        let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
        let err = String::from_utf8_lossy(&out.stderr);
        if !err.trim().is_empty() {
            if !text.is_empty() {
                text.push('\n');
            }
            text.push_str(&err);
        }
        Ok(OpResult { ok: out.status.success(), output: text })
    })
    .await
    .map_err(|e| format!("blocking task: {e}"))?
}

// ── commands (§1 #1) ────────────────────────────────────────────────────────

/// Build the distributable payload: `bash tools/install/build.sh` in the checkout.
#[tauri::command]
pub async fn ops_build(checkout: String) -> Result<OpResult, String> {
    let root = require_checkout(&checkout)?;
    let script = root.join("tools/install/build.sh");
    run_bash(script, vec![], root).await
}

/// Installed-vs-available: `bash dist/cyberos/init.sh --check <project>`.
#[tauri::command]
pub async fn ops_check(checkout: String, project: String) -> Result<OpResult, String> {
    let root = require_checkout(&checkout)?;
    let init = require_payload(&root)?;
    let p = require_project(&root, &project)?;
    run_bash(init, vec!["--check".into(), p.to_string_lossy().into_owned()], root).await
}

/// Init or update a project: `bash dist/cyberos/init.sh <project>`. Idempotent by design.
#[tauri::command]
pub async fn ops_init(checkout: String, project: String) -> Result<OpResult, String> {
    let root = require_checkout(&checkout)?;
    let init = require_payload(&root)?;
    let p = require_project(&root, &project)?;
    run_bash(init, vec![p.to_string_lossy().into_owned()], root).await
}

/// Candidate projects: git repositories at `~/Projects/*` and `~/Projects/*/*`,
/// with their installed CyberOS version when present. The UI also accepts a
/// manually typed path, so this list is a convenience, not a constraint.
#[tauri::command]
pub async fn ops_list_projects() -> Result<Vec<ProjectInfo>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let base = home.join("Projects");
        let mut found: Vec<ProjectInfo> = Vec::new();
        let push_if_repo = |dir: &Path, found: &mut Vec<ProjectInfo>| {
            if dir.join(".git").exists() {
                let installed = std::fs::read_to_string(dir.join(".cyberos/VERSION"))
                    .ok()
                    .map(|s| s.trim().to_string());
                found.push(ProjectInfo {
                    path: dir.to_string_lossy().into_owned(),
                    name: dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
                    installed_version: installed,
                });
            }
        };
        if let Ok(level1) = std::fs::read_dir(&base) {
            for e1 in level1.flatten() {
                let p1 = e1.path();
                if !p1.is_dir() {
                    continue;
                }
                push_if_repo(&p1, &mut found);
                if !p1.join(".git").exists() {
                    if let Ok(level2) = std::fs::read_dir(&p1) {
                        for e2 in level2.flatten() {
                            let p2 = e2.path();
                            if p2.is_dir() {
                                push_if_repo(&p2, &mut found);
                            }
                        }
                    }
                }
            }
        }
        found.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(found)
    })
    .await
    .map_err(|e| format!("blocking task: {e}"))?
}
