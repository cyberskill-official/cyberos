//! `cyberos-skill-transpile-anthropic` — emit Anthropic-flat SKILL.md from CCSM.
//!
//! Per FR-SKILL-103 §3 + Anthropic guide Reference B. The output is the
//! authoritative deliverable for shipping a CyberOS skill to Claude.ai /
//! Claude Code / the Anthropic Agent Skills API.

use std::path::PathBuf;

use clap::Parser;
use cyberos_cli_exit::ExitCode;
use cyberos_skill_broker::transpile_anthropic;

#[derive(Parser)]
#[command(
    name = "cyberos-skill-transpile-anthropic",
    about = "Transpile a CyberOS SKILL.md (CCSM) to Anthropic-flat form"
)]
struct Args {
    /// Source bundle directory (containing SKILL.md) OR direct SKILL.md path
    bundle: PathBuf,

    /// Output path. If a directory, writes <dir>/SKILL.md. If a file, writes there.
    /// If omitted, prints to stdout.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let exit_code = run(args);
    std::process::exit(exit_code as i32);
}

fn run(args: Args) -> ExitCode {
    match transpile_anthropic(&args.bundle) {
        Ok(skill) => {
            let content = skill.to_skill_md();
            match args.output {
                Some(out_path) => {
                    let target = if out_path.is_dir() {
                        out_path.join("SKILL.md")
                    } else {
                        out_path
                    };
                    if let Some(parent) = target.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    match std::fs::write(&target, &content) {
                        Ok(()) => {
                            println!("✓ {} → {}", skill.name, target.display());
                            ExitCode::Ok
                        }
                        Err(e) => {
                            eprintln!("✗ write failed: {e}");
                            ExitCode::ConfigError
                        }
                    }
                }
                None => {
                    print!("{}", content);
                    ExitCode::Ok
                }
            }
        }
        Err(e) => {
            eprintln!("✗ transpile failed: {e}");
            ExitCode::PreconditionFailed
        }
    }
}
