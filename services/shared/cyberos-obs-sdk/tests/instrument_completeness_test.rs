use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn rust_axum_routers_are_red_instrumented() {
    let services = services_root();
    let required = [
        "memory/src/main.rs",
        "auth/src/handlers.rs",
        "email/src/bin/server.rs",
        "mcp-gateway/src/router.rs",
        "obs-collector/src/ingress.rs",
        "obs-collector/src/grafana_proxy.rs",
    ];

    let mut missing = Vec::new();
    for relative in required {
        let path = services.join(relative);
        let body = fs::read_to_string(&path).expect("read router file");
        if body.contains("Router::new()") && !body.contains("cyberos_obs_sdk::red::RedLayer") {
            missing.push(relative.to_string());
        }
    }

    assert!(
        missing.is_empty(),
        "router files missing RED layer instrumentation: {missing:?}"
    );
}

#[test]
fn non_rust_chat_service_is_not_scanned_by_rust_macro_lint() {
    let services = services_root();
    assert!(services.join("chat/cyberos_chat").is_dir());
    assert!(
        !services.join("chat/Cargo.toml").exists(),
        "chat is not a Rust axum crate in this workspace"
    );
}

fn services_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .ancestors()
        .find(|path| {
            path.join("auth/Cargo.toml").exists() && path.join("memory/Cargo.toml").exists()
        })
        .expect("services root")
        .to_path_buf()
}
