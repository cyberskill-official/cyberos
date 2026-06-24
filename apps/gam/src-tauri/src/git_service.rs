use std::process::Command;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::known_repos_service::KnownReposService;
use crate::ranking_service::RankingService;

/// Patterns in alias commands that are considered dangerous.
static DANGEROUS_PATTERNS: &[(&str, &str)] = &[
    (r"rm\s+(-rf|-fr|--recursive)", "Contains recursive delete (rm -rf)"),
    (r"push\s+(?:\S.*)?--force", "Contains force push (--force)"),
    (r"push\s+(?:\S.*)?-f\b", "Contains force push (-f)"),
    (r"reset\s+--hard", "Contains hard reset (reset --hard)"),
    (r"clean\s+(?:\S.*)?-fd", "Contains force clean (clean -fd)"),
    (r"branch\s+(?:\S.*)?-D", "Contains force branch delete (-D)"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitAlias {
    pub name: String,
    pub command: String,
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

pub struct GitService {
    local_path: Option<String>,
    known_repos_service: KnownReposService,
    ranking_service: RankingService,
}

impl Default for GitService {
    fn default() -> Self {
        Self::new()
    }
}

impl GitService {
    pub fn new() -> Self {
        Self {
            local_path: None,
            known_repos_service: KnownReposService::new(),
            ranking_service: RankingService::new(),
        }
    }

    pub fn set_local_path(&mut self, path: Option<String>) {
        if let Some(ref p) = path {
            self.known_repos_service.add(p);
        }
        self.local_path = path;
    }

    pub fn get_local_path(&self) -> Option<String> {
        self.local_path.clone()
    }

    /// Toggle whether alias ranking reads the user's shell history.
    pub fn set_history_ranking_enabled(&mut self, enabled: bool) {
        self.ranking_service.set_enabled(enabled);
    }

    pub fn is_history_ranking_enabled(&self) -> bool {
        self.ranking_service.is_enabled()
    }

    fn exec_git(&self, args: &[&str], cwd: Option<&str>) -> Result<String, String> {
        let mut cmd = Command::new("git");
        cmd.args(args);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    // git config returns exit code 1 when no aliases exist
                    if output.status.code() == Some(1)
                        && args.contains(&"--get-regexp")
                    {
                        Ok(String::new())
                    } else if stderr.is_empty() {
                        Err(format!("Git command failed with exit code {:?}", output.status.code()))
                    } else {
                        Err(stderr)
                    }
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Err("Git is not installed or not found in PATH".to_string())
                } else {
                    Err(e.to_string())
                }
            }
        }
    }

    pub fn get_aliases(&mut self, scope: &str) -> Result<Vec<GitAlias>, String> {
        let mut aliases: Vec<GitAlias> = Vec::new();

        if scope == "global" || scope == "all" {
            match self.exec_git(
                &["config", "--global", "--get-regexp", r"^alias\."],
                None,
            ) {
                Ok(output) => {
                    aliases.extend(Self::parse_alias_output(&output, "global", None));
                }
                Err(_) => {
                    // No global aliases or no global config
                }
            }
        }

        if scope == "local" || scope == "all" {
            let paths_to_scan: Vec<String> = if scope == "all" || self.local_path.is_none() {
                self.known_repos_service.get_all()
            } else {
                self.local_path.iter().cloned().collect()
            };

            for scan_path in &paths_to_scan {
                match self.exec_git(
                    &["config", "--local", "--get-regexp", r"^alias\."],
                    Some(scan_path),
                ) {
                    Ok(output) => {
                        aliases.extend(Self::parse_alias_output(
                            &output,
                            "local",
                            Some(scan_path.clone()),
                        ));
                    }
                    Err(_) => {
                        // Ignore errors for individual paths
                    }
                }
            }
        }

        // Sort alphabetically
        aliases.sort_by_key(|a| a.name.to_lowercase());

        // Rank aliases based on telemetry history
        if let Ok(scores) = self.ranking_service.get_scores(&aliases) {
            for alias in &mut aliases {
                alias.score = Some(*scores.get(&alias.name).unwrap_or(&0.0));
            }
        }

        Ok(aliases)
    }

    fn parse_alias_output(
        output: &str,
        scope: &str,
        local_path: Option<String>,
    ) -> Vec<GitAlias> {
        if output.is_empty() {
            return Vec::new();
        }

        output
            .lines()
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                let first_space = line.find(' ')?;
                let key = &line[..first_space];
                let command = &line[first_space + 1..];
                let name = key.strip_prefix("alias.").unwrap_or(key);

                Some(GitAlias {
                    name: name.to_string(),
                    command: command.to_string(),
                    scope: scope.to_string(),
                    local_path: local_path.clone(),
                    score: None,
                })
            })
            .collect()
    }

    pub fn add_alias(
        &mut self,
        name: &str,
        command: &str,
        scope: &str,
        local_path: Option<&str>,
    ) -> Result<(), String> {
        // Defense-in-depth: validate alias name even though frontend also checks
        Self::validate_alias_name(name)?;

        // Check if alias already exists
        let existing = self.get_aliases(scope)?;
        if existing.iter().any(|a| a.name == name) {
            return Err(format!(
                "Alias \"{}\" already exists in {} config",
                name, scope
            ));
        }

        let target_path = if scope == "local" {
            local_path
                .map(|s| s.to_string())
                .or_else(|| self.local_path.clone())
        } else {
            None
        };

        self.exec_git(
            &["config", &format!("--{}", scope), &format!("alias.{}", name), command],
            target_path.as_deref(),
        )?;

        Ok(())
    }

    pub fn update_alias(
        &mut self,
        old_name: &str,
        name: &str,
        command: &str,
        scope: &str,
        local_path: Option<&str>,
    ) -> Result<(), String> {
        Self::validate_alias_name(name)?;

        let target_path = if scope == "local" {
            local_path
                .map(|s| s.to_string())
                .or_else(|| self.local_path.clone())
        } else {
            None
        };

        // If name changed, delete old one first
        if old_name != name {
            let _ = self.exec_git(
                &[
                    "config",
                    &format!("--{}", scope),
                    "--unset",
                    &format!("alias.{}", old_name),
                ],
                target_path.as_deref(),
            );
        }

        self.exec_git(
            &["config", &format!("--{}", scope), &format!("alias.{}", name), command],
            target_path.as_deref(),
        )?;

        Ok(())
    }

    pub fn delete_alias(
        &mut self,
        name: &str,
        scope: &str,
        local_path: Option<&str>,
    ) -> Result<(), String> {
        let target_path = if scope == "local" {
            local_path
                .map(|s| s.to_string())
                .or_else(|| self.local_path.clone())
        } else {
            None
        };

        self.exec_git(
            &[
                "config",
                &format!("--{}", scope),
                "--unset",
                &format!("alias.{}", name),
            ],
            target_path.as_deref(),
        )?;

        Ok(())
    }

    pub fn validate_command(&self, command: &str) -> ValidationResult {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        if command.trim().is_empty() {
            errors.push("Command cannot be empty".to_string());
            return ValidationResult {
                valid: false,
                warnings,
                errors,
            };
        }

        // Check for dangerous patterns (precompiled once via OnceLock)
        fn dangerous_patterns() -> &'static [(regex_lite::Regex, &'static str)] {
            static PATTERNS: OnceLock<Vec<(regex_lite::Regex, &'static str)>> = OnceLock::new();
            PATTERNS.get_or_init(|| {
                DANGEROUS_PATTERNS
                    .iter()
                    .filter_map(|(pat, msg)| {
                        regex_lite::Regex::new(pat).ok().map(|re| (re, *msg))
                    })
                    .collect()
            })
        }

        for (re, message) in dangerous_patterns() {
            if re.is_match(command) {
                warnings.push(message.to_string());
            }
        }

        // Check for shell commands (starting with !)
        if command.starts_with('!') {
            warnings.push(
                "This is a shell command alias (starts with !). Use with caution.".to_string(),
            );
        }

        ValidationResult {
            valid: errors.is_empty(),
            warnings,
            errors,
        }
    }

    /// Validates that an alias name matches `^[a-zA-Z][\w-]*$`.
    /// Defense-in-depth: the frontend enforces the same rule.
    fn validate_alias_name(name: &str) -> Result<(), String> {
        fn alias_name_re() -> &'static regex_lite::Regex {
            static RE: OnceLock<regex_lite::Regex> = OnceLock::new();
            RE.get_or_init(|| regex_lite::Regex::new(r"^[a-zA-Z][\w-]*$").unwrap())
        }

        if name.is_empty() {
            return Err("Alias name cannot be empty".to_string());
        }
        if !alias_name_re().is_match(name) {
            return Err(format!(
                "Invalid alias name \"{}\": must start with a letter and contain only letters, numbers, hyphens, underscores",
                name
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_alias_output_parses_global_aliases() {
        let output = "alias.co checkout\nalias.st status -sb\n";
        let aliases = GitService::parse_alias_output(output, "global", None);

        assert_eq!(aliases.len(), 2);
        assert_eq!(aliases[0].name, "co");
        assert_eq!(aliases[0].command, "checkout");
        assert_eq!(aliases[0].scope, "global");
        assert!(aliases[0].local_path.is_none());
    }

    #[test]
    fn parse_alias_output_handles_local_path() {
        let output = "alias.lg log --oneline --graph\n";
        let aliases = GitService::parse_alias_output(output, "local", Some("/tmp/repo".to_string()));

        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].scope, "local");
        assert_eq!(aliases[0].local_path, Some("/tmp/repo".to_string()));
    }

    #[test]
    fn parse_alias_output_handles_empty_input() {
        let aliases = GitService::parse_alias_output("", "global", None);
        assert!(aliases.is_empty());
    }

    #[test]
    fn parse_alias_output_skips_empty_lines() {
        let output = "alias.co checkout\n\n\nalias.st status\n";
        let aliases = GitService::parse_alias_output(output, "global", None);
        assert_eq!(aliases.len(), 2);
    }

    #[test]
    fn parse_alias_output_handles_multiword_commands() {
        let output = "alias.lg log --oneline --graph --decorate --all\n";
        let aliases = GitService::parse_alias_output(output, "global", None);
        assert_eq!(aliases[0].command, "log --oneline --graph --decorate --all");
    }

    #[test]
    fn validate_empty_command() {
        let svc = GitService::new();
        let result = svc.validate_command("");
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn validate_whitespace_only_command() {
        let svc = GitService::new();
        let result = svc.validate_command("   ");
        assert!(!result.valid);
    }

    #[test]
    fn validate_safe_command() {
        let svc = GitService::new();
        let result = svc.validate_command("checkout -b");
        assert!(result.valid);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn validate_force_push_warning() {
        let svc = GitService::new();
        let result = svc.validate_command("push origin --force");
        assert!(result.valid);
        assert!(!result.warnings.is_empty());
        assert!(result.warnings.iter().any(|w| w.contains("force push")));
    }

    #[test]
    fn validate_rm_rf_warning() {
        let svc = GitService::new();
        let result = svc.validate_command("!rm -rf /tmp/test");
        assert!(result.valid);
        assert!(result.warnings.iter().any(|w| w.contains("recursive delete")));
    }

    #[test]
    fn validate_shell_command_warning() {
        let svc = GitService::new();
        let result = svc.validate_command("!echo hello");
        assert!(result.valid);
        assert!(result.warnings.iter().any(|w| w.contains("shell command")));
    }

    #[test]
    fn validate_reset_hard_warning() {
        let svc = GitService::new();
        let result = svc.validate_command("reset --hard HEAD~1");
        assert!(result.valid);
        assert!(result.warnings.iter().any(|w| w.contains("hard reset")));
    }

    #[test]
    fn new_service_has_no_local_path() {
        let svc = GitService::new();
        assert!(svc.get_local_path().is_none());
    }

    #[test]
    fn set_and_get_local_path() {
        let mut svc = GitService::new();
        svc.set_local_path(Some("/tmp/test-repo".to_string()));
        assert_eq!(svc.get_local_path(), Some("/tmp/test-repo".to_string()));
    }

    #[test]
    fn clear_local_path() {
        let mut svc = GitService::new();
        svc.set_local_path(Some("/tmp/test-repo".to_string()));
        svc.set_local_path(None);
        assert!(svc.get_local_path().is_none());
    }

    #[test]
    fn validate_alias_name_accepts_valid() {
        assert!(GitService::validate_alias_name("co").is_ok());
        assert!(GitService::validate_alias_name("checkout-main").is_ok());
        assert!(GitService::validate_alias_name("st2").is_ok());
        assert!(GitService::validate_alias_name("my_alias").is_ok());
    }

    #[test]
    fn validate_alias_name_rejects_empty() {
        assert!(GitService::validate_alias_name("").is_err());
    }

    #[test]
    fn validate_alias_name_rejects_special_chars() {
        assert!(GitService::validate_alias_name("co;rm").is_err());
        assert!(GitService::validate_alias_name("co\ngit push").is_err());
        assert!(GitService::validate_alias_name("co && echo").is_err());
    }

    #[test]
    fn validate_alias_name_rejects_numeric_start() {
        assert!(GitService::validate_alias_name("1co").is_err());
        assert!(GitService::validate_alias_name("-dash").is_err());
    }

    #[test]
    fn validate_alias_name_rejects_injection() {
        assert!(GitService::validate_alias_name("co\ngit push --force").is_err());
        assert!(GitService::validate_alias_name("alias.evil").is_err());
    }

    // --- Integration tests: exercise the real `git` subprocess (exec_git) ---
    //
    // These operate in local scope against a throwaway git repo in a temp dir, so
    // they never read or write the user's global ~/.gitconfig. They require `git`
    // on PATH, which both CI and dev machines have. Each test gets its own unique
    // temp repo, so they are safe to run in parallel.

    /// A unique temp git repo that removes itself on drop. Dependency-free.
    struct TempRepo {
        path: std::path::PathBuf,
    }

    impl TempRepo {
        fn new() -> Self {
            use std::sync::atomic::{AtomicU32, Ordering};
            static COUNTER: AtomicU32 = AtomicU32::new(0);

            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!(
                "gam-it-{}-{}-{}",
                std::process::id(),
                unique,
                nanos
            ));
            std::fs::create_dir_all(&dir).expect("create temp dir");

            let initialized = std::process::Command::new("git")
                .args(["init", "--quiet"])
                .current_dir(&dir)
                .status()
                .expect("run git init")
                .success();
            assert!(initialized, "git init failed in temp repo");

            Self { path: dir }
        }

        fn path_str(&self) -> String {
            self.path.to_string_lossy().to_string()
        }
    }

    impl Drop for TempRepo {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    fn local_service(repo: &TempRepo) -> GitService {
        let mut svc = GitService::new();
        svc.set_local_path(Some(repo.path_str()));
        svc
    }

    #[test]
    fn integration_add_then_get_local_alias() {
        let repo = TempRepo::new();
        let mut svc = local_service(&repo);

        svc.add_alias("co", "checkout", "local", None).expect("add");

        let aliases = svc.get_aliases("local").expect("get");
        let co = aliases.iter().find(|a| a.name == "co").expect("co present");
        assert_eq!(co.command, "checkout");
        assert_eq!(co.scope, "local");
    }

    #[test]
    fn integration_get_on_fresh_repo_is_empty() {
        let repo = TempRepo::new();
        let mut svc = local_service(&repo);

        let aliases = svc.get_aliases("local").expect("get");
        assert!(aliases.is_empty(), "fresh repo had {} aliases", aliases.len());
    }

    #[test]
    fn integration_add_duplicate_is_rejected() {
        let repo = TempRepo::new();
        let mut svc = local_service(&repo);

        svc.add_alias("st", "status -sb", "local", None).expect("add");
        let err = svc
            .add_alias("st", "status", "local", None)
            .expect_err("duplicate should fail");
        assert!(err.contains("already exists"), "unexpected error: {err}");
    }

    #[test]
    fn integration_update_renames_and_changes_command() {
        let repo = TempRepo::new();
        let mut svc = local_service(&repo);

        svc.add_alias("ci", "commit", "local", None).expect("add");
        svc.update_alias("ci", "cm", "commit -v", "local", None)
            .expect("update");

        let aliases = svc.get_aliases("local").expect("get");
        assert!(aliases.iter().all(|a| a.name != "ci"), "old name still present");
        let cm = aliases.iter().find(|a| a.name == "cm").expect("new name present");
        assert_eq!(cm.command, "commit -v");
    }

    #[test]
    fn integration_delete_removes_alias() {
        let repo = TempRepo::new();
        let mut svc = local_service(&repo);

        svc.add_alias("br", "branch", "local", None).expect("add");
        svc.delete_alias("br", "local", None).expect("delete");

        let aliases = svc.get_aliases("local").expect("get");
        assert!(aliases.iter().all(|a| a.name != "br"), "br was not removed");
    }

    #[test]
    fn integration_multiword_command_roundtrips() {
        let repo = TempRepo::new();
        let mut svc = local_service(&repo);

        let command = "log --oneline --graph --decorate --all";
        svc.add_alias("lg", command, "local", None).expect("add");

        let aliases = svc.get_aliases("local").expect("get");
        let lg = aliases.iter().find(|a| a.name == "lg").expect("lg present");
        assert_eq!(lg.command, command);
    }

    #[test]
    fn integration_invalid_name_is_rejected_before_git() {
        let repo = TempRepo::new();
        let mut svc = local_service(&repo);

        let err = svc
            .add_alias("co;rm -rf", "checkout", "local", None)
            .expect_err("invalid name should fail");
        assert!(err.contains("Invalid alias name"), "unexpected error: {err}");
    }
}
