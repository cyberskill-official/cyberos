//! FR-MCP-003 — SEP-986 tool-name validator.

/// Validate `cyberos.{module}.{verb}_{noun}` names.
pub fn validate_tool_name(name: &str) -> Result<(), String> {
    let parts: Vec<&str> = name.split('.').collect();
    if parts.len() != 3 || parts[0] != "cyberos" {
        return Err("tool_name_must_match_cyberos_module_verb_noun".into());
    }
    validate_segment(parts[1], "module")?;
    let Some((verb, noun)) = parts[2].split_once('_') else {
        return Err("tool_action_must_be_verb_noun".into());
    };
    validate_segment(verb, "verb")?;
    validate_segment(noun, "noun")?;
    Ok(())
}

fn validate_segment(segment: &str, label: &str) -> Result<(), String> {
    if segment.is_empty()
        || !segment
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(format!("invalid_{label}_segment"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sep986_accepts_canonical_name() {
        assert!(validate_tool_name("cyberos.memory.search_memory").is_ok());
    }

    #[test]
    fn sep986_rejects_non_cyberos_prefix() {
        assert!(validate_tool_name("other.memory.search_memory").is_err());
    }
}
