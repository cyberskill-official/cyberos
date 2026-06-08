//! FR-AI-021 §1 #8 — JSON schema validation for versioned CLI output.

use serde::Serialize;
use std::path::Path;

/// Validate a value against a versioned JSON schema file.
pub fn validate_output<T: Serialize>(
    command: &str,
    version: &str,
    value: &T,
) -> Result<(), String> {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("cli")
        .join("json_schemas")
        .join(format!("{command}.{version}.json"));
    let schema_str = std::fs::read_to_string(&schema_path)
        .map_err(|e| format!("read schema {}: {e}", schema_path.display()))?;
    let schema: serde_json::Value =
        serde_json::from_str(&schema_str).map_err(|e| format!("parse schema: {e}"))?;
    let json = serde_json::to_value(value).map_err(|e| format!("serialise output: {e}"))?;

    let compiled =
        jsonschema::JSONSchema::compile(&schema).map_err(|e| format!("compile schema: {e}"))?;

    if let Err(errors) = compiled.validate(&json) {
        let msgs: Vec<String> = errors.map(|e| e.to_string()).collect();
        return Err(msgs.join("; "));
    }
    Ok(())
}
