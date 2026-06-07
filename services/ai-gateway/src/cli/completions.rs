//! FR-AI-021 §1 #15 — Shell completion generation.

use clap::CommandFactory;
use clap_complete::{
    generate,
    shells::{Bash, Fish, Zsh},
};
use std::io;

use super::{Cli, Shell};

pub fn run(shell: Shell) {
    let mut cmd = Cli::command();
    let bin_name = "cyberos-ai";

    match shell {
        Shell::Bash => generate(Bash, &mut cmd, bin_name, &mut io::stdout()),
        Shell::Zsh => generate(Zsh, &mut cmd, bin_name, &mut io::stdout()),
        Shell::Fish => generate(Fish, &mut cmd, bin_name, &mut io::stdout()),
    }
}
