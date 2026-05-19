use std::fs;
use std::path::Path;

pub fn validate_migrations(migrations_dir: &Path, adr_dir: &Path) -> Result<(), String> {
    let entries = fs::read_dir(migrations_dir).map_err(|e| format!("Failed to read migrations: {}", e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("sql") {
            let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
            
            let touches_roles = content.contains("INSERT INTO roles") || content.contains("UPDATE roles") || content.contains("DELETE FROM roles")
                || content.contains("INSERT INTO role_permissions") || content.contains("UPDATE role_permissions") || content.contains("DELETE FROM role_permissions");
                
            if touches_roles {
                let adr_line = content.lines().find(|l| l.contains("-- ADR: ") && !l.contains("ADR-NNN"));
                if let Some(line) = adr_line {
                    let adr_name = line.split("-- ADR: ").nth(1).unwrap().split_whitespace().next().unwrap();
                    let expected_adr_path = adr_dir.join(format!("{}.md", adr_name));
                    
                    if !expected_adr_path.exists() {
                        return Err(format!("Migration {} touches roles but referenced ADR file {} does not exist", path.display(), expected_adr_path.display()));
                    }
                } else {
                    return Err(format!("Migration {} touches roles but lacks an '-- ADR: ADR-NNN' comment", path.display()));
                }
            }
        }
    }
    
    Ok(())
}
