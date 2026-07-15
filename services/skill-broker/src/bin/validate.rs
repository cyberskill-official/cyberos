//! `cyberos-skill-validate` CLI — validate one or more SKILL.md bundles.
//!
//! Per TASK-SKILL-103 §3 + TASK-SKILL-111 §3 + TASK-SKILL-113 §3.

use std::path::PathBuf;

use clap::Parser;
use cyberos_cli_exit::ExitCode;
use cyberos_skill_broker::{frontmatter, FrontmatterError};

#[derive(Parser)]
#[command(
    name = "cyberos-skill-validate",
    about = "Validate SKILL.md frontmatter against TASK-SKILL-103/111/113 rules"
)]
struct Args {
    /// Bundle directory or SKILL.md path
    bundle: PathBuf,

    /// Output JSON instead of human-readable
    #[arg(long)]
    json: bool,
}

fn main() {
    let args = Args::parse();
    let exit_code = run(args);
    std::process::exit(exit_code as i32);
}

fn run(args: Args) -> ExitCode {
    match frontmatter::load_and_validate(&args.bundle) {
        Ok((fm, _body)) => {
            if args.json {
                println!(
                    "{}",
                    serde_json::json!({
                        "status": "ok",
                        "name": fm.name,
                        "description_length": fm.description.replace('\n', " ").trim().chars().count(),
                        "wrap_in_marker": fm.untrusted_inputs.as_ref().map(|ui| ui.wrap_in_marker.as_str()),
                    })
                );
            } else {
                println!("✓ {} — valid", fm.name);
            }
            ExitCode::Ok
        }
        Err(e) => {
            eprintln!("✗ validation failed: {e}");
            match e {
                FrontmatterError::Io(_) => ExitCode::ConfigError,
                _ => ExitCode::PreconditionFailed,
            }
        }
    }
}
