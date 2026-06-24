use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::git_service::GitAlias;

/// Time-based recency multipliers matching the README algorithm.
const MULTIPLIER_1H: f64 = 4.0;
const MULTIPLIER_1D: f64 = 2.0;
const MULTIPLIER_1W: f64 = 0.5;
const MULTIPLIER_OLD: f64 = 0.25;
/// If the total score across all aliases exceeds this threshold,
/// all frequencies are halved to prevent score inflation.
const SCORE_CEILING: f64 = 70_000.0;

/// Precompiled regex patterns for shell history parsing.
fn re_zsh_timestamped() -> &'static regex_lite::Regex {
    static RE: OnceLock<regex_lite::Regex> = OnceLock::new();
    RE.get_or_init(|| regex_lite::Regex::new(r":\s*(\d+):\d+;.*?git\s+([\w-]+)").unwrap())
}
fn re_zsh_plain() -> &'static regex_lite::Regex {
    static RE: OnceLock<regex_lite::Regex> = OnceLock::new();
    RE.get_or_init(|| regex_lite::Regex::new(r"(?m)^git\s+([\w-]+)").unwrap())
}
fn re_git_cmd() -> &'static regex_lite::Regex {
    static RE: OnceLock<regex_lite::Regex> = OnceLock::new();
    RE.get_or_init(|| regex_lite::Regex::new(r"git\s+([\w-]+)").unwrap())
}

/// Entry tracking both total frequency and most-recent timestamp.
struct HistoryEntry {
    frequency: f64,
    last_seen: u64, // Unix seconds
}

/// Ranks aliases by parsing shell history files.
///
/// Scoring formula:
///   Score = TimeMultiplier × Length^(3/5) × Frequency
///
/// Where:
/// - TimeMultiplier is based on the recency of last usage
/// - Length is the character length of the aliased command
/// - Frequency is how often the command appears in history
pub struct RankingService {
    history_cache: HashMap<String, HistoryEntry>,
    last_fetch_time: Option<Instant>,
    cache_ttl: Duration,
    /// When false, the user has opted out and no shell history is read.
    enabled: bool,
}

impl Default for RankingService {
    fn default() -> Self {
        Self::new()
    }
}

impl RankingService {
    pub fn new() -> Self {
        Self {
            history_cache: HashMap::new(),
            last_fetch_time: None,
            cache_ttl: Duration::from_secs(5),
            enabled: true,
        }
    }

    /// Enable or disable shell-history scoring. Disabling also clears any cached
    /// history so opting out also forgets what was already read.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.history_cache.clear();
            self.last_fetch_time = None;
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_scores(
        &mut self,
        aliases: &[GitAlias],
    ) -> Result<HashMap<String, f64>, String> {
        // Respect the user's consent choice: when disabled, never read shell history.
        if !self.enabled {
            return Ok(aliases.iter().map(|a| (a.name.clone(), 0.0)).collect());
        }

        let should_refresh = match self.last_fetch_time {
            Some(t) => t.elapsed() > self.cache_ttl,
            None => true,
        };

        if should_refresh || self.history_cache.is_empty() {
            self.refresh_history_cache();
            self.last_fetch_time = Some(Instant::now());
        }

        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut scores: HashMap<String, f64> = HashMap::new();
        let mut total_score: f64 = 0.0;

        for alias in aliases {
            let entry = self.history_cache.get(&format!("git {}", alias.name));

            let (frequency, last_seen) = match entry {
                Some(e) => (e.frequency, e.last_seen),
                None => {
                    // Fall back to base command matching
                    let base_cmd = alias.command.split_whitespace().next().unwrap_or("");
                    let base_entry = self.history_cache.get(&format!("git {}", base_cmd));
                    match base_entry {
                        Some(e) => (e.frequency * 0.2, e.last_seen),
                        None => {
                            scores.insert(alias.name.clone(), 0.0);
                            continue;
                        }
                    }
                }
            };

            // Time multiplier based on recency
            let elapsed = if last_seen > 0 { now_secs.saturating_sub(last_seen) } else { u64::MAX };
            let time_multiplier = if elapsed <= 3600 {
                MULTIPLIER_1H
            } else if elapsed <= 86400 {
                MULTIPLIER_1D
            } else if elapsed <= 604800 {
                MULTIPLIER_1W
            } else {
                MULTIPLIER_OLD
            };

            // Length factor: command length ^ (3/5)
            let length_factor = (alias.command.len() as f64).powf(0.6);

            let score = (time_multiplier * length_factor * frequency).round();
            total_score += score;
            scores.insert(alias.name.clone(), score);
        }

        // Auto-reset: halve all frequencies when total exceeds ceiling
        if total_score > SCORE_CEILING {
            for entry in self.history_cache.values_mut() {
                entry.frequency /= 2.0;
            }
            // Recalculate scores with halved frequencies
            for (_, score) in scores.iter_mut() {
                *score /= 2.0;
            }
        }

        Ok(scores)
    }

    fn refresh_history_cache(&mut self) {
        let mut new_cache: HashMap<String, HistoryEntry> = HashMap::new();
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        if cfg!(not(target_os = "windows")) {
            // Try zsh history (format: ": timestamp:0;command")
            let zsh_path = home.join(".zsh_history");
            if let Ok(content) = fs::read_to_string(&zsh_path) {
                Self::parse_zsh_history(&content, &mut new_cache);
            }

            // Try bash history (plain lines or with timestamps)
            let bash_path = home.join(".bash_history");
            if let Ok(content) = fs::read_to_string(&bash_path) {
                Self::parse_bash_history(&content, &mut new_cache);
            }

            // Try Fish history (~/.local/share/fish/fish_history)
            let fish_path = dirs::data_dir()
                .unwrap_or_else(|| home.join(".local").join("share"))
                .join("fish")
                .join("fish_history");
            if let Ok(content) = fs::read_to_string(&fish_path) {
                Self::parse_fish_history(&content, &mut new_cache);
            }
        } else {
            // PowerShell history on Windows
            let ps_path = home
                .join("AppData")
                .join("Roaming")
                .join("Microsoft")
                .join("Windows")
                .join("PowerShell")
                .join("PSReadLine")
                .join("ConsoleHost_history.txt");
            if let Ok(content) = fs::read_to_string(&ps_path) {
                Self::parse_plain_history(&content, &mut new_cache);
            }
        }

        self.history_cache = new_cache;
    }

    /// Parse zsh history with timestamps: ": 1234567890:0;git checkout main"
    fn parse_zsh_history(content: &str, cache: &mut HashMap<String, HistoryEntry>) {
        let re = re_zsh_timestamped();
        for cap in re.captures_iter(content) {
            if let (Some(ts), Some(cmd)) = (cap.get(1), cap.get(2)) {
                let timestamp: u64 = ts.as_str().parse().unwrap_or(0);
                let key = format!("git {}", cmd.as_str());
                let entry = cache.entry(key).or_insert(HistoryEntry {
                    frequency: 0.0,
                    last_seen: 0,
                });
                entry.frequency += 1.0;
                if timestamp > entry.last_seen {
                    entry.last_seen = timestamp;
                }
            }
        }

        // Also match plain "git xxx" lines without the timestamp prefix
        let re_plain = re_zsh_plain();
        for cap in re_plain.captures_iter(content) {
            if let Some(cmd) = cap.get(1) {
                let key = format!("git {}", cmd.as_str());
                let entry = cache.entry(key).or_insert(HistoryEntry {
                    frequency: 0.0,
                    last_seen: 0,
                });
                entry.frequency += 1.0;
            }
        }
    }

    /// Parse plain history (PowerShell) — no timestamps available.
    fn parse_plain_history(content: &str, cache: &mut HashMap<String, HistoryEntry>) {
        let re = re_git_cmd();
        for cap in re.captures_iter(content) {
            if let Some(m) = cap.get(1) {
                let key = format!("git {}", m.as_str());
                let entry = cache.entry(key).or_insert(HistoryEntry {
                    frequency: 0.0,
                    last_seen: 0,
                });
                entry.frequency += 1.0;
            }
        }
    }

    /// Parse bash history — handles both plain and timestamped format.
    /// Timestamped format: lines starting with `#1234567890` followed by the command on the next line.
    fn parse_bash_history(content: &str, cache: &mut HashMap<String, HistoryEntry>) {
        let lines: Vec<&str> = content.lines().collect();
        let git_re = re_git_cmd();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];

            // Check for timestamp prefix: #1234567890
            if line.starts_with('#')
                && let Ok(ts) = line[1..].trim().parse::<u64>() {
                    // Next line is the command
                    if i + 1 < lines.len() {
                        if let Some(cap) = git_re.captures(lines[i + 1])
                            && let Some(cmd) = cap.get(1) {
                                let key = format!("git {}", cmd.as_str());
                                let entry = cache.entry(key).or_insert(HistoryEntry {
                                    frequency: 0.0,
                                    last_seen: 0,
                                });
                                entry.frequency += 1.0;
                                if ts > entry.last_seen {
                                    entry.last_seen = ts;
                                }
                            }
                        i += 2;
                        continue;
                    }
                }

            // Plain line (no timestamp)
            if let Some(cap) = git_re.captures(line)
                && let Some(cmd) = cap.get(1) {
                    let key = format!("git {}", cmd.as_str());
                    let entry = cache.entry(key).or_insert(HistoryEntry {
                        frequency: 0.0,
                        last_seen: 0,
                    });
                    entry.frequency += 1.0;
                }

            i += 1;
        }
    }

    /// Parse Fish shell history (YAML-like format):
    /// ```text
    /// - cmd: git checkout main
    ///   when: 1700000000
    /// ```
    fn parse_fish_history(content: &str, cache: &mut HashMap<String, HistoryEntry>) {
        let git_re = re_git_cmd();
        let mut current_cmd: Option<String> = None;

        for line in content.lines() {
            let trimmed = line.trim();

            if let Some(cmd_part) = trimmed.strip_prefix("- cmd:") {
                current_cmd = Some(cmd_part.trim().to_string());
            } else if let Some(when_part) = trimmed.strip_prefix("when:") {
                if let Some(ref cmd) = current_cmd {
                    let timestamp: u64 = when_part.trim().parse().unwrap_or(0);
                    if let Some(cap) = git_re.captures(cmd)
                        && let Some(m) = cap.get(1) {
                            let key = format!("git {}", m.as_str());
                            let entry = cache.entry(key).or_insert(HistoryEntry {
                                frequency: 0.0,
                                last_seen: 0,
                            });
                            entry.frequency += 1.0;
                            if timestamp > entry.last_seen {
                                entry.last_seen = timestamp;
                            }
                        }
                }
                current_cmd = None;
            }
        }

        // Handle trailing cmd without when
        if let Some(ref cmd) = current_cmd
            && let Some(cap) = git_re.captures(cmd)
                && let Some(m) = cap.get(1) {
                    let key = format!("git {}", m.as_str());
                    let entry = cache.entry(key).or_insert(HistoryEntry {
                        frequency: 0.0,
                        last_seen: 0,
                    });
                    entry.frequency += 1.0;
                }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_service_has_empty_cache() {
        let svc = RankingService::new();
        assert!(svc.history_cache.is_empty());
        assert!(svc.last_fetch_time.is_none());
    }

    #[test]
    fn scores_empty_aliases() {
        let mut svc = RankingService::new();
        let aliases: Vec<GitAlias> = vec![];
        let scores = svc.get_scores(&aliases).unwrap();
        assert!(scores.is_empty());
    }

    #[test]
    fn scores_alias_with_no_history() {
        let mut svc = RankingService::new();
        let aliases = vec![GitAlias {
            name: "co".to_string(),
            command: "checkout".to_string(),
            scope: "global".to_string(),
            local_path: None,
            score: None,
        }];
        let scores = svc.get_scores(&aliases).unwrap();
        assert_eq!(*scores.get("co").unwrap(), 0.0);
    }

    #[test]
    fn parse_zsh_history_extracts_timestamps() {
        let content = ": 1700000000:0;git checkout main\n: 1700000100:0;git status\n";
        let mut cache = HashMap::new();
        RankingService::parse_zsh_history(content, &mut cache);

        let co = cache.get("git checkout").unwrap();
        assert_eq!(co.frequency, 1.0);
        assert_eq!(co.last_seen, 1700000000);

        let st = cache.get("git status").unwrap();
        assert_eq!(st.frequency, 1.0);
        assert_eq!(st.last_seen, 1700000100);
    }

    #[test]
    fn parse_plain_history_counts_frequency() {
        let content = "git commit -m 'test'\ngit commit --amend\nls -la\ngit status\n";
        let mut cache = HashMap::new();
        RankingService::parse_plain_history(content, &mut cache);

        assert_eq!(cache.get("git commit").unwrap().frequency, 2.0);
        assert_eq!(cache.get("git status").unwrap().frequency, 1.0);
        assert!(!cache.contains_key("git ls"));
    }

    #[test]
    fn time_multipliers_are_correct() {
        // Verify the constants match the README
        assert_eq!(MULTIPLIER_1H, 4.0);
        assert_eq!(MULTIPLIER_1D, 2.0);
        assert_eq!(MULTIPLIER_1W, 0.5);
        assert_eq!(MULTIPLIER_OLD, 0.25);
    }

    #[test]
    fn parse_fish_history_extracts_commands_with_timestamps() {
        let content = "- cmd: git checkout main\n  when: 1700000000\n- cmd: git status\n  when: 1700000100\n- cmd: ls -la\n  when: 1700000200\n";
        let mut cache = HashMap::new();
        RankingService::parse_fish_history(content, &mut cache);

        let co = cache.get("git checkout").unwrap();
        assert_eq!(co.frequency, 1.0);
        assert_eq!(co.last_seen, 1700000000);

        let st = cache.get("git status").unwrap();
        assert_eq!(st.frequency, 1.0);
        assert_eq!(st.last_seen, 1700000100);

        assert!(!cache.contains_key("git ls"));
    }

    #[test]
    fn parse_fish_history_handles_no_when() {
        let content = "- cmd: git push origin main\n";
        let mut cache = HashMap::new();
        RankingService::parse_fish_history(content, &mut cache);

        assert_eq!(cache.get("git push").unwrap().frequency, 1.0);
    }

    #[test]
    fn parse_bash_history_with_timestamps() {
        let content = "#1700000000\ngit checkout main\n#1700000100\ngit status\n#1700000200\nls -la\n";
        let mut cache = HashMap::new();
        RankingService::parse_bash_history(content, &mut cache);

        let co = cache.get("git checkout").unwrap();
        assert_eq!(co.frequency, 1.0);
        assert_eq!(co.last_seen, 1700000000);

        let st = cache.get("git status").unwrap();
        assert_eq!(st.frequency, 1.0);
        assert_eq!(st.last_seen, 1700000100);
    }

    #[test]
    fn parse_bash_history_plain_lines() {
        let content = "git commit -m 'test'\ngit push\nls\n";
        let mut cache = HashMap::new();
        RankingService::parse_bash_history(content, &mut cache);

        assert_eq!(cache.get("git commit").unwrap().frequency, 1.0);
        assert_eq!(cache.get("git push").unwrap().frequency, 1.0);
        assert!(!cache.contains_key("git ls"));
    }

    #[test]
    fn disabled_ranking_returns_zero_scores() {
        let mut svc = RankingService::new();
        assert!(svc.is_enabled(), "ranking is enabled by default");

        svc.set_enabled(false);
        assert!(!svc.is_enabled());

        let aliases = vec![crate::git_service::GitAlias {
            name: "co".to_string(),
            command: "checkout".to_string(),
            scope: "global".to_string(),
            local_path: None,
            score: None,
        }];
        let scores = svc.get_scores(&aliases).expect("scores");
        assert_eq!(scores.get("co"), Some(&0.0));
    }
}
