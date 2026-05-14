//! Invocation pipeline. Phase 6 wires capability checks + first-use prompt.

use crate::capabilities::Capability;
use crate::grants::{default_grants_path, is_granted, record_grant};
use tracing::{info, warn};

pub struct InvokeContext {
    pub skill_name: String,
    pub skill_md_sha256: String,
    pub operator: String,
    pub interactive: bool,
}

pub fn ensure_granted(ctx: &InvokeContext, required: &Capability) -> anyhow::Result<()> {
    let path = default_grants_path();
    if is_granted(&path, &ctx.skill_name, &ctx.skill_md_sha256, required) {
        info!(skill = %ctx.skill_name, cap = %required, "capability already granted");
        return Ok(());
    }

    // Auto-deny dangerous tools unconditionally — operator must edit
    // grants.json by hand if they really mean it.
    if matches!(required.name.as_str(), "bash" | "shell" | "exec") {
        warn!(skill = %ctx.skill_name, cap = %required,
              "dangerous capability requested — auto-denied, edit grants.json manually to override");
        anyhow::bail!("capability '{}' is auto-denied for safety", required);
    }

    if !ctx.interactive {
        anyhow::bail!(
            "skill `{}` requires capability `{}` but no grant exists. Run `cyberos-skill cap grant --skill {} --caps {}` interactively first.",
            ctx.skill_name, required, ctx.skill_name, required
        );
    }

    // Interactive prompt — read y/n from stdin.
    use std::io::{self, Write};
    println!();
    println!("Skill `{}` requests capability `{}`", ctx.skill_name, required);
    println!("This will be recorded against the skill's content hash;");
    println!("modifying the SKILL.md will require re-approval.");
    print!("Grant? [y/N] ");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    if !answer.trim().eq_ignore_ascii_case("y") {
        anyhow::bail!("operator denied capability `{}`", required);
    }
    record_grant(&path, &ctx.skill_name, &ctx.skill_md_sha256,
                 &[required.clone()], &ctx.operator)?;
    info!(skill = %ctx.skill_name, cap = %required, "capability granted by operator");
    Ok(())
}
