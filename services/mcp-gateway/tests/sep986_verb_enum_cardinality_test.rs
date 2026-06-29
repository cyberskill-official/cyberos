//! AC-01: Sep986Verb enum MUST have exactly 15 variants per DEC-2361 (governance tripwire).

use cyberos_mcp_gateway::naming::validator::Sep986Verb;

#[test]
fn sep986_verb_enum_has_exactly_15_variants() {
    assert_eq!(
        Sep986Verb::all_variants().len(),
        15,
        "Sep986Verb cardinality must be exactly 15 per DEC-2361; adding a verb requires a SEP RFC"
    );
}

#[test]
fn sep986_verb_all_known_verbs_parse() {
    for verb_str in [
        "get", "list", "create", "update", "delete", "send", "fetch", "sync", "validate",
        "generate", "execute", "search", "replay", "accept", "reject",
    ] {
        assert!(
            Sep986Verb::from_verb_str(verb_str).is_some(),
            "expected verb '{verb_str}' to parse"
        );
    }
}

#[test]
fn sep986_verb_round_trip() {
    for variant in Sep986Verb::all_variants() {
        let s = variant.as_str();
        assert_eq!(
            Sep986Verb::from_verb_str(s),
            Some(*variant),
            "round-trip failed for {variant:?}"
        );
    }
}

#[test]
fn sep986_verb_unknown_returns_none() {
    for s in ["retrieve", "read", "post", "put", "GET", "List", ""] {
        assert!(
            Sep986Verb::from_verb_str(s).is_none(),
            "expected '{s}' to NOT parse as a verb"
        );
    }
}
