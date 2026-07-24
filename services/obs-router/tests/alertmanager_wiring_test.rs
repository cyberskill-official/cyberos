//! Residual TASK-OBS-007 verification: Alertmanager wiring file exists and
//! points at the obs-router `/alert` webhook (batch/9b-obs adopt).

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    // tests run with CARGO_MANIFEST_DIR = services/obs-router
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn alertmanager_config_wires_obs_router_alert_webhook() {
    let path = repo_root().join("deploy/obs/alertmanager-config.yaml");
    let body = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("missing {}: {e}", path.display()));
    assert!(
        body.contains("cyberos-obs-router"),
        "receiver name must be cyberos-obs-router"
    );
    assert!(
        body.contains("/alert"),
        "webhook URL must target obs-router /alert"
    );
    assert!(
        body.contains("OBS_ROUTER"),
        "must reference OBS_ROUTER_* env placeholders (no live secrets)"
    );
    assert!(
        !body.contains("skills/obs.triage-alert"),
        "must not claim the phantom skills/ tree"
    );
}
