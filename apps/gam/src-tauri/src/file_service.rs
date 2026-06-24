use std::fs;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::git_service::GitAlias;
use crate::group_service::{AliasGroup, GroupData};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub version: String,
    #[serde(rename = "exportedAt")]
    pub exported_at: String,
    pub aliases: Vec<GitAlias>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<AliasGroup>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignments: Option<std::collections::HashMap<String, Vec<String>>>,
}

/// Handles export and import of alias data to/from JSON files.
pub struct FileService;

impl FileService {
    pub fn export_aliases(
        aliases: &[GitAlias],
        file_path: &str,
        group_data: Option<&GroupData>,
    ) -> Result<String, String> {
        let export_data = ExportData {
            version: "1.0.0".to_string(),
            exported_at: iso8601_now(),
            aliases: aliases.to_vec(),
            groups: group_data.map(|gd| gd.groups.clone()),
            assignments: group_data.map(|gd| gd.assignments.clone()),
        };

        let json = serde_json::to_string_pretty(&export_data)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        fs::write(file_path, &json).map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(file_path.to_string())
    }

    pub fn import_aliases(file_path: &str) -> Result<Vec<GitAlias>, String> {
        // Cap file size to prevent OOM on maliciously large files
        const MAX_IMPORT_SIZE: u64 = 10 * 1024 * 1024; // 10 MB
        let meta = fs::metadata(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        if meta.len() > MAX_IMPORT_SIZE {
            return Err(format!(
                "File too large ({:.1} MB). Maximum import size is 10 MB.",
                meta.len() as f64 / 1_048_576.0
            ));
        }

        let content =
            fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

        let data: ExportData =
            serde_json::from_str(&content).map_err(|_| "Invalid JSON file".to_string())?;

        // Validate structure
        for alias in &data.aliases {
            if alias.name.is_empty() {
                return Err("Invalid alias: missing or invalid \"name\" field".to_string());
            }
            if alias.command.is_empty() {
                return Err(format!(
                    "Invalid alias \"{}\": missing or invalid \"command\" field",
                    alias.name
                ));
            }
        }

        // Default scope to global if not specified
        let aliases: Vec<GitAlias> = data
            .aliases
            .into_iter()
            .map(|mut a| {
                if a.scope.is_empty() {
                    a.scope = "global".to_string();
                }
                a
            })
            .collect();

        Ok(aliases)
    }
}

/// Produce a proper ISO 8601 timestamp (UTC) without the chrono crate.
///
/// Output format: `2026-02-22T16:30:00Z`
fn iso8601_now() -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Manual UTC decomposition (no leap-second precision needed)
    let days = now / 86400;
    let secs_in_day = now % 86400;
    let hours = secs_in_day / 3600;
    let minutes = (secs_in_day % 3600) / 60;
    let seconds = secs_in_day % 60;

    // Days since 1970-01-01 → (year, month, day)
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hours, minutes, seconds
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn iso8601_now_format() {
        let ts = iso8601_now();
        // Should match YYYY-MM-DDTHH:MM:SSZ
        assert!(ts.ends_with('Z'));
        assert_eq!(ts.len(), 20);
        assert_eq!(&ts[4..5], "-");
        assert_eq!(&ts[7..8], "-");
        assert_eq!(&ts[10..11], "T");
        assert_eq!(&ts[13..14], ":");
        assert_eq!(&ts[16..17], ":");
    }

    #[test]
    fn export_creates_valid_json() {
        let tmp = std::env::temp_dir().join("gam_test_export.json");
        let aliases = vec![GitAlias {
            name: "co".to_string(),
            command: "checkout".to_string(),
            scope: "global".to_string(),
            local_path: None,
            score: None,
        }];
        FileService::export_aliases(&aliases, tmp.to_str().unwrap(), None).unwrap();

        let content = fs::read_to_string(&tmp).unwrap();
        let data: ExportData = serde_json::from_str(&content).unwrap();
        assert_eq!(data.version, "1.0.0");
        assert_eq!(data.aliases.len(), 1);
        assert_eq!(data.aliases[0].name, "co");
        assert!(data.exported_at.ends_with('Z'));

        fs::remove_file(&tmp).ok();
    }

    #[test]
    fn import_validates_empty_name() {
        let tmp = std::env::temp_dir().join("gam_test_import_bad.json");
        let json = r#"{"version":"1.0.0","exportedAt":"2026-01-01T00:00:00Z","aliases":[{"name":"","command":"checkout","scope":"global"}]}"#;
        fs::write(&tmp, json).unwrap();

        let err = FileService::import_aliases(tmp.to_str().unwrap()).unwrap_err();
        assert!(err.contains("missing or invalid \"name\""));

        fs::remove_file(&tmp).ok();
    }

    #[test]
    fn import_defaults_scope_to_global() {
        let tmp = std::env::temp_dir().join("gam_test_import_scope.json");
        let json = r#"{"version":"1.0.0","exportedAt":"2026-01-01T00:00:00Z","aliases":[{"name":"co","command":"checkout","scope":""}]}"#;
        fs::write(&tmp, json).unwrap();

        let aliases = FileService::import_aliases(tmp.to_str().unwrap()).unwrap();
        assert_eq!(aliases[0].scope, "global");

        fs::remove_file(&tmp).ok();
    }

    #[test]
    fn import_roundtrip() {
        let tmp = std::env::temp_dir().join("gam_test_roundtrip.json");
        let original = vec![
            GitAlias {
                name: "co".to_string(),
                command: "checkout".to_string(),
                scope: "global".to_string(),
                local_path: None,
                score: None,
            },
            GitAlias {
                name: "st".to_string(),
                command: "status -sb".to_string(),
                scope: "local".to_string(),
                local_path: Some("/tmp/repo".to_string()),
                score: None,
            },
        ];

        FileService::export_aliases(&original, tmp.to_str().unwrap(), None).unwrap();
        let imported = FileService::import_aliases(tmp.to_str().unwrap()).unwrap();

        assert_eq!(imported.len(), 2);
        assert_eq!(imported[0].name, "co");
        assert_eq!(imported[1].command, "status -sb");

        fs::remove_file(&tmp).ok();
    }
}
