//! FR-AI-018 §3 — Proptest strategies for cache isolation property tests.

use proptest::prelude::*;

pub fn any_tenant_id() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_:\\-.]{1,32}".prop_map(String::from)
}

pub fn any_tenant_pair() -> impl Strategy<Value = (String, String)> {
    (any_tenant_id(), any_tenant_id()).prop_filter("tenants must differ", |(a, b)| a != b)
}

pub fn any_prompt() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ?!.,]{0,200}".prop_map(String::from)
}

pub fn any_model() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("chat.fast".to_string()),
        Just("chat.smart".to_string()),
        Just("embed.standard".to_string()),
        "[a-z]{4,12}\\.[a-z]{4,8}".prop_map(String::from),
    ]
}

pub fn any_persona_handle() -> impl Strategy<Value = String> {
    "[a-z\\-]{4,16}@\\d+\\.\\d+\\.\\d+".prop_map(String::from)
}

pub fn any_cache_op() -> impl Strategy<Value = (String, String, String)> {
    (any_prompt(), any_model(), any_persona_handle())
}

pub fn adversarial_tenant_strings() -> Vec<&'static str> {
    vec![
        "",
        "\x00",
        "\x1ftenant",
        "tenant\x1fid",
        "tenant\u{202E}id",
        "tenant\u{200D}id",
        "tenant'; DROP TABLE--",
        "../../../etc/passwd",
        "\u{FEFF}tenant",
        "TENANT",
    ]
}
