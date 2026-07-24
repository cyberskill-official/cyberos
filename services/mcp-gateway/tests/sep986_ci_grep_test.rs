//! TASK-MCP-003 — CI grep gate covers the DEC-2362 source-side tripwire.
//!
//! Runs `scripts/check_sep986_naming.sh` against the live tree (must exit 0) and against a
//! temporary fixture that plants a non-conforming registry-module skill ID (must exit non-zero).
//! Both phases share one test so they cannot race on a `services/` fixture directory.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("services/")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn script_path() -> PathBuf {
    repo_root().join("scripts/check_sep986_naming.sh")
}

fn run_gate() -> std::process::Output {
    Command::new("bash")
        .arg(script_path())
        .current_dir(repo_root())
        .output()
        .expect("run check_sep986_naming.sh")
}

#[test]
fn sep986_ci_grep_live_clean_and_planted_violation_fails() {
    let fixture_dir = repo_root().join("services/_sep986_ci_fixture_tmp");
    let _ = fs::remove_dir_all(&fixture_dir);

    let live = run_gate();
    assert!(
        live.status.success(),
        "live tree must be SEP-986 clean:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&live.stdout),
        String::from_utf8_lossy(&live.stderr)
    );

    fs::create_dir_all(&fixture_dir).expect("mkdir fixture");
    let fixture = fixture_dir.join("planted.rs");
    fs::write(
        &fixture,
        r#"// temporary SEP-986 CI fixture — deleted by this test
pub const BAD: &str = "cyberos.obs.triage";
"#,
    )
    .expect("write fixture");

    let planted = run_gate();
    let _ = fs::remove_dir_all(&fixture_dir);

    assert!(
        !planted.status.success(),
        "planted cyberos.obs.triage must fail the gate:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&planted.stdout),
        String::from_utf8_lossy(&planted.stderr)
    );
    let stdout = String::from_utf8_lossy(&planted.stdout);
    assert!(
        stdout.contains("cyberos.obs.triage") || stdout.contains("SEP-986 violation"),
        "failure output should name the violation:\n{stdout}"
    );

    // Ensure cleanup left the tree clean for subsequent suite runs.
    let after = run_gate();
    assert!(
        after.status.success(),
        "fixture cleanup must restore a clean tree:\n{}",
        String::from_utf8_lossy(&after.stdout)
    );
}
