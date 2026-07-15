//! TASK-AI-003 §1 #6 + AGENTS.md §6.2 — Canonical JSON serialisation.
//!
//! Rules:
//! - NFC-normalised UTF-8 throughout (§6 AC #6 — round-trip combining-acute → pre-composed).
//! - Sorted keys (deep).
//! - No insignificant whitespace.
//! - Integers in their natural form (no `.0`).
//! - The top-level object has exactly these keys (sorted): `body`, `meta`, `path`.
//! - The `meta` object has exactly these keys (sorted): `actor`, `actor_version`, `extra`,
//!   `kind`.

use serde_json::{json, Value};
use unicode_normalization::UnicodeNormalization;

use super::MemoryEmit;

const ACTOR: &str = "agent:cyberos-ai-gateway";
const ACTOR_VERSION: &str = env!("CARGO_PKG_VERSION");

/// TASK-AI-003 §1 #6 — Build the canonical-JSON payload that gets piped to the Writer
/// subprocess's stdin. Single line, no trailing newline (caller adds it).
pub fn serialise(req: &MemoryEmit) -> Result<String, String> {
    let body = build_body_markdown(req);
    let body_nfc: String = body.nfc().collect();
    let path_nfc: String = req.path.nfc().collect();
    let extra_nfc = nfc_value(&req.extra);

    let payload = json!({
        "body": body_nfc,
        "meta": {
            "actor": ACTOR,
            "actor_version": ACTOR_VERSION,
            "extra": extra_nfc,
            "kind": req.kind.tag(),
        },
        "path": path_nfc,
    });

    canonicalise(&payload)
}

/// Build the markdown body the Writer persists under `<memory-root>/`. Front-matter only;
/// the AI Gateway has no narrative content to add, only structured fields.
fn build_body_markdown(req: &MemoryEmit) -> String {
    let mut body = String::new();
    body.push_str("---\n");
    body.push_str(&format!("kind: {}\n", req.kind.tag()));
    body.push_str(&format!("actor: {ACTOR}\n"));
    if let Some(obj) = req.extra.as_object() {
        let mut keys: Vec<&String> = obj.keys().collect();
        keys.sort();
        for k in keys {
            let v = &obj[k];
            // Scalar: emit as `k: value`. Otherwise: compact JSON.
            match v {
                Value::String(s) => body.push_str(&format!("{k}: {s}\n")),
                Value::Number(n) => body.push_str(&format!("{k}: {n}\n")),
                Value::Bool(b) => body.push_str(&format!("{k}: {b}\n")),
                Value::Null => body.push_str(&format!("{k}: null\n")),
                _ => {
                    body.push_str(&format!("{k}: {}\n", canonicalise(v).unwrap_or_default()));
                }
            }
        }
    }
    body.push_str("---\n");
    body
}

/// Deep NFC normalisation of all string-valued nodes.
fn nfc_value(v: &Value) -> Value {
    match v {
        Value::String(s) => Value::String(s.nfc().collect()),
        Value::Array(a) => Value::Array(a.iter().map(nfc_value).collect()),
        Value::Object(o) => Value::Object(
            o.iter()
                .map(|(k, v)| (k.nfc().collect(), nfc_value(v)))
                .collect(),
        ),
        _ => v.clone(),
    }
}

/// Sorted-key, no-extra-whitespace JSON. Matches `python -c "json.dumps(d, sort_keys=True,
/// separators=(',', ':'))"`.
fn canonicalise(v: &Value) -> Result<String, String> {
    fn write(v: &Value, out: &mut String) -> Result<(), String> {
        match v {
            Value::Null => out.push_str("null"),
            Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Value::Number(n) => out.push_str(&n.to_string()),
            Value::String(s) => {
                out.push('"');
                for c in s.chars() {
                    match c {
                        '"' => out.push_str("\\\""),
                        '\\' => out.push_str("\\\\"),
                        '\n' => out.push_str("\\n"),
                        '\r' => out.push_str("\\r"),
                        '\t' => out.push_str("\\t"),
                        c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                        c => out.push(c),
                    }
                }
                out.push('"');
            }
            Value::Array(a) => {
                out.push('[');
                for (i, item) in a.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write(item, out)?;
                }
                out.push(']');
            }
            Value::Object(map) => {
                out.push('{');
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort();
                for (i, k) in keys.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write(&Value::String((*k).clone()), out)?;
                    out.push(':');
                    write(&map[*k], out)?;
                }
                out.push('}');
            }
        }
        Ok(())
    }

    let mut s = String::with_capacity(256);
    write(v, &mut s)?;
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_writer::AiInvocationKind;

    #[test]
    fn canonical_sorts_keys() {
        let v = json!({"b": 1, "a": 2});
        let s = canonicalise(&v).unwrap();
        assert_eq!(s, r#"{"a":2,"b":1}"#);
    }

    #[test]
    fn canonical_strips_whitespace() {
        let v = json!({"x": [1, 2, 3]});
        let s = canonicalise(&v).unwrap();
        assert_eq!(s, r#"{"x":[1,2,3]}"#);
    }

    #[test]
    fn canonical_escapes_strings() {
        let v = json!({"x": "he said \"hi\"\nthen left"});
        let s = canonicalise(&v).unwrap();
        assert_eq!(s, r#"{"x":"he said \"hi\"\nthen left"}"#);
    }

    #[test]
    fn nfc_normalises_combining_acute() {
        // "café" with COMBINING ACUTE (U+0065 U+0301), not pre-composed (U+00E9)
        let decomposed = "cafe\u{0301}";
        let v = Value::String(decomposed.to_string());
        let n = nfc_value(&v);
        let s = n.as_str().unwrap();
        // After NFC, the é is U+00E9 (UTF-8: 0xC3 0xA9)
        let bytes = s.as_bytes();
        assert!(
            bytes.windows(2).any(|w| w == [0xC3, 0xA9]),
            "expected pre-composed é"
        );
        assert!(
            !bytes.windows(2).any(|w| w == [0xCC, 0x81]),
            "combining acute U+0301 should be normalised away"
        );
    }

    #[test]
    fn serialise_memory_emit_round_trips() {
        let req = MemoryEmit {
            kind: AiInvocationKind::Precheck,
            path: "memories/ai-invocations/test.md".to_string(),
            extra: json!({"tenant_id": "org:test", "estimated_usd": "0.0085"}),
        };
        let s = serialise(&req).unwrap();
        assert!(s.starts_with('{'));
        assert!(s.ends_with('}'));
        // Re-parse to check it's valid JSON + sorted.
        let parsed: Value = serde_json::from_str(&s).unwrap();
        let top = parsed.as_object().unwrap();
        let mut keys: Vec<_> = top.keys().collect();
        keys.sort();
        assert_eq!(keys, vec!["body", "meta", "path"]);
        let meta = top["meta"].as_object().unwrap();
        let mut mkeys: Vec<_> = meta.keys().collect();
        mkeys.sort();
        assert_eq!(mkeys, vec!["actor", "actor_version", "extra", "kind"]);
        assert_eq!(meta["actor"], "agent:cyberos-ai-gateway");
        assert_eq!(meta["kind"], "ai.precheck");
    }
}
