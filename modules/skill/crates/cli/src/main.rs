//! cyberos-skill — Rust CLI surface for the CyberOS skill module.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cyberos_skill_host::{Loader, SkillRegistry};
use cyberos_skill_manifest::{parse_frontmatter, validate_manifest};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser, Debug)]
#[command(name = "cyberos-skill", version, about = "CyberOS Skill module CLI")]
struct Cli {
    /// Skill roots to index. Repeat for multiple. Defaults to
    /// `./skills/` if run from the skill/ module folder.
    #[arg(long, global = true)]
    root: Vec<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// List installed skills (Level-1 view: name + description).
    List,
    /// Show full frontmatter + first 200 chars of body for one skill.
    Info { name: String },
    /// Validate one or more SKILL.md files against the spec.
    Validate { paths: Vec<PathBuf> },
    /// Bundle a skill directory into a deterministic .skill.tar.gz.
    Package {
        skill_dir: PathBuf,
        #[arg(long, default_value = ".")]
        out: PathBuf,
    },
    /// Install a skill from a local path into the user-global skill root.
    Install {
        /// Path to a skill directory containing SKILL.md.
        skill_dir: PathBuf,
        /// Target root. Defaults to ~/.cyberos/skills/ (uses $HOME).
        #[arg(long)]
        target: Option<PathBuf>,
        /// Overwrite an existing skill of the same name.
        #[arg(long)]
        force: bool,
    },
    /// Invoke a skill with JSON input on stdin, write JSON output to stdout.
    /// Default executor=auto: WASM if dist/skill.wasm exists, otherwise
    /// dispatches to the native-script tier (Phase 2 / audit §3).
    Run {
        skill: String,
        /// Executor selector: auto | script | wasm. auto picks wasm when
        /// dist/skill.wasm exists (and the `wasm` feature is compiled in),
        /// otherwise falls back to script.
        #[arg(long, default_value = "auto")]
        executor: String,
    },
    /// Manage skill capability grants.
    Cap {
        #[command(subcommand)]
        action: CapAction,
    },
}

#[derive(Subcommand, Debug)]
enum CapAction {
    /// List recorded grants.
    List,
    /// Audit dangerous grants (bash/shell/exec) and unsigned skills.
    Audit,
    /// Revoke all grants for a skill name.
    Revoke { skill: String },
}

fn default_roots() -> Vec<PathBuf> {
    // 2026-05-17 flat-layout rebuild: skill bundles live directly under skill/
    // (no more skill/skills/ subfolder). The loader's EXCLUDED_DIR_NAMES list
    // filters out infra dirs like crates/, contracts/, docs/, etc.
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut roots = Vec::new();

    // Case 1: invoked from inside skill/ — cwd IS the skill module root.
    // Heuristic: skill/ contains both Cargo.toml AND MODULE.md.
    if cwd.join("Cargo.toml").is_file() && cwd.join("MODULE.md").is_file() {
        roots.push(cwd.clone());
    }
    // Case 2: invoked from the project root — look for skill/ subdir.
    let try_module = cwd.join("skill");
    if try_module.is_dir() && try_module.join("MODULE.md").is_file() {
        roots.push(try_module);
    }
    // Case 3: user-global install root.
    if let Ok(home) = std::env::var("HOME") {
        let user_global = PathBuf::from(home).join(".cyberos").join("skills");
        if user_global.is_dir() {
            roots.push(user_global);
        }
    }
    // Case 4 (legacy soak compat — remove after 30-day soak per Phase 7):
    // honour the old skill/skills/ path if it still exists somewhere.
    let try_legacy_local  = cwd.join("skills");
    let try_legacy_module = cwd.join("skill/skills");
    if try_legacy_local.is_dir()  { roots.push(try_legacy_local); }
    if try_legacy_module.is_dir() { roots.push(try_legacy_module); }

    if roots.is_empty() {
        // Last-resort fallback so help/error messages name something concrete.
        roots.push(PathBuf::from("./skill"));
    }
    roots
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let roots = if cli.root.is_empty() { default_roots() } else { cli.root.clone() };

    match cli.command {
        Command::List => {
            let registry = Arc::new(SkillRegistry::new());
            let loader = Loader::new(Arc::clone(&registry));
            let n = loader.index_roots(&roots).await?;
            println!("Indexed {} skill(s) under: {:?}", n, roots);
            println!();
            println!("  {:<32}  {}", "NAME", "DESCRIPTION (first 80 chars)");
            println!("  {:-<32}  {:-<80}", "", "");
            for (name, desc) in registry.header_summaries() {
                let short = desc.replace('\n', " ");
                let short: String = short.chars().take(80).collect();
                println!("  {:<32}  {}", name, short);
            }
        }
        Command::Info { name } => {
            let registry = Arc::new(SkillRegistry::new());
            let loader = Loader::new(Arc::clone(&registry));
            loader.index_roots(&roots).await?;
            let h = registry.get_header(&name)
                .ok_or_else(|| anyhow::anyhow!("skill not found: {}", name))?;
            println!("name        : {}", h.manifest.name);
            println!("description : {}", h.manifest.description);
            println!("dir         : {}", h.skill_dir.display());
            if let Some(lic) = &h.manifest.license {
                println!("license     : {}", lic);
            }
            if let Some(compat) = &h.manifest.compatibility {
                println!("compatibility: {}", compat);
            }
            if !h.manifest.metadata.is_empty() {
                println!("metadata    :");
                for (k, v) in &h.manifest.metadata {
                    println!("  {} = {}", k, v);
                }
            }
            if let Some(tools) = &h.manifest.allowed_tools {
                println!("allowed-tools: {:?}", tools.as_vec());
            }
        }
        Command::Validate { paths } => {
            let mut errors = 0usize;
            for p in &paths {
                match std::fs::read(p) {
                    Ok(bytes) => match parse_frontmatter(&bytes) {
                        Ok((m, _)) => match validate_manifest(&m) {
                            Ok(()) => println!("  OK    {}", p.display()),
                            Err(e) => { println!("  FAIL  {} — {}", p.display(), e); errors += 1; }
                        },
                        Err(e) => { println!("  PARSE {} — {}", p.display(), e); errors += 1; }
                    },
                    Err(e) => { println!("  READ  {} — {}", p.display(), e); errors += 1; }
                }
            }
            if errors > 0 {
                anyhow::bail!("{}/{} file(s) failed validation", errors, paths.len());
            }
        }
        Command::Package { skill_dir, out } => {
            // Phase 1: shell out to skill/tools/package.sh for deterministic tarball
            // build (sorted entries, fixed mtime, fixed uid/gid). Rewriting in pure
            // Rust is a Phase 5+ task once the on-disk format is locked.
            let script = locate_package_script()?;
            let status = std::process::Command::new("bash")
                .arg(&script)
                .arg(&skill_dir)
                .arg("--out")
                .arg(&out)
                .status()?;
            if !status.success() {
                anyhow::bail!("package.sh exited with status {}", status);
            }
        }
        Command::Install { skill_dir, target, force } => {
            install_skill(&skill_dir, target.as_deref(), force)?;
        }
        Command::Run { skill, executor } => {
            run_skill(&roots, &skill, &executor).await?;
        }
        Command::Cap { action } => {
            handle_cap(action)?;
        }
    }
    Ok(())
}

/// Map skill name -> primary entry script (Phase 2 native-script tier).
/// Mirrors `tools/run_fixtures.py::PRIMARY_SCRIPT` to keep parity tests
/// honest. Phase 5 generalises this via a manifest field.
///
/// 2026-05-17 flat-layout rebuild: the SDP-driven catalog (sow/srs/task/prd/etc.
/// author+audit pairs) executes through LLM prompts described in each SKILL.md
/// body — none of them ship a primary executable script in this generation.
/// The VN bundles (vietnam-mst-validate, vietnam-vat-invoice, vietnam-bank-transfer,
/// vietnam-vneid-integration, vn-tax-filing) have moved to `cyberos/public-skills/`
/// for open-registry publication; their mappings are kept here only for the
/// 30-day Phase 7 soak compat window. After Phase 7 finalisation, this
/// function reduces to `_ => None`.
fn primary_script(name: &str) -> Option<&'static str> {
    match name {
        // Legacy VN bundles (soak-window compat only — present at
        // cyberos/public-skills/ for open-registry publication).
        "vietnam-mst-validate"   => Some("scripts/validate_mst.py"),
        "vietnam-vat-invoice"    => Some("scripts/generate_invoice.py"),
        "vietnam-bank-transfer"  => Some("scripts/generate_qr.py"),
        "vietnam-vneid-integration" => Some("scripts/validate_cccd.py"),
        "vn-tax-filing"     => Some("scripts/generate_return.py"),
        // SDP-driven catalog bundles: no primary script — body is prompt-only.
        _ => None,
    }
}

/// Pick the actual executor to use given the user's request and the skill's
/// on-disk artefacts. Returns "script" or "wasm".
///
/// - `script` → always returns "script" (legacy native-script tier).
/// - `wasm`   → returns "wasm" iff the `wasm` Cargo feature is compiled in,
///              otherwise errors with a clear install hint.
/// - `auto`   → prefers wasm when both (a) `dist/skill.wasm` exists under the
///              skill dir and (b) the `wasm` feature is compiled in.
///              Falls back to "script" otherwise.
fn pick_executor(requested: &str, skill_dir: &Path) -> anyhow::Result<&'static str> {
    match requested {
        "script" => Ok("script"),
        "wasm" => {
            #[cfg(not(feature = "wasm"))]
            {
                let _ = skill_dir;
                anyhow::bail!(
                    "wasm executor not compiled in. Rebuild with `cargo build --features wasm` after running `rustup target add wasm32-wasi`."
                );
            }
            #[cfg(feature = "wasm")]
            {
                let _ = skill_dir;
                Ok("wasm")
            }
        }
        "auto" => {
            let wasm_path = skill_dir.join("dist/skill.wasm");
            if wasm_path.is_file() {
                #[cfg(feature = "wasm")]
                { Ok("wasm") }
                #[cfg(not(feature = "wasm"))]
                { Ok("script") }
            } else {
                Ok("script")
            }
        }
        other => anyhow::bail!(
            "unknown executor: {} (use script|wasm|auto)",
            other
        ),
    }
}

/// Phase 3 `cyberos-skill run <skill>` — resolve via registry, dispatch to
/// either the native-script tier or (when compiled in + a `dist/skill.wasm`
/// is present) the Wasmtime component-model tier. Pipes stdin/stdout
/// through transparently and forwards exit code.
async fn run_skill(
    roots: &[PathBuf],
    skill_name: &str,
    executor_req: &str,
) -> Result<()> {
    let registry = Arc::new(SkillRegistry::new());
    let loader = Loader::new(Arc::clone(&registry));
    loader.index_roots(roots).await?;
    let header = registry
        .get_header(skill_name)
        .ok_or_else(|| anyhow::anyhow!("skill not found: {}", skill_name))?;

    let exec_kind = pick_executor(executor_req, &header.skill_dir)?;

    tracing::info!(
        skill = %skill_name,
        requested = executor_req,
        chosen = exec_kind,
        "dispatching skill"
    );

    if exec_kind == "wasm" {
        #[cfg(feature = "wasm")]
        {
            return run_wasm(skill_name, &header.skill_dir).await;
        }
        #[cfg(not(feature = "wasm"))]
        {
            // pick_executor() already filters this branch out when the
            // feature isn't compiled, but keep a defensive arm.
            anyhow::bail!(
                "wasm executor selected but not compiled in. Rebuild with `cargo build --features wasm` after running `rustup target add wasm32-wasi`."
            );
        }
    }

    // script tier
    let script_rel = primary_script(skill_name).ok_or_else(|| {
        anyhow::anyhow!(
            "no primary script mapping registered for skill '{}'",
            skill_name
        )
    })?;
    let script_path = header.skill_dir.join(script_rel);
    if !script_path.is_file() {
        anyhow::bail!(
            "primary script not found at {}",
            script_path.display()
        );
    }

    // Read stdin fully (small fixture payloads only at this point) and
    // hand to python3 via tokio::process::Command for async execution.
    use tokio::io::AsyncReadExt;
    let mut stdin_buf = Vec::new();
    tokio::io::stdin().read_to_end(&mut stdin_buf).await?;

    let python = std::env::var("PYTHON").unwrap_or_else(|_| "python3".to_string());
    let mut cmd = tokio::process::Command::new(&python);
    cmd.arg(&script_path);
    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::inherit());

    let mut child = cmd.spawn().with_context(|| {
        format!("spawning {} {}", python, script_path.display())
    })?;
    if !stdin_buf.is_empty() {
        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            stdin.write_all(&stdin_buf).await?;
            stdin.shutdown().await?;
        }
    } else {
        // Close stdin so the child sees EOF immediately.
        drop(child.stdin.take());
    }

    let output = child.wait_with_output().await?;
    // Forward stdout verbatim — byte-for-byte parity is required.
    use std::io::Write;
    std::io::stdout().write_all(&output.stdout)?;
    std::io::stdout().flush()?;

    let code = output.status.code().unwrap_or(1);
    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}

/// WASM dispatcher — Phase 5. Reads stdin, calls the Wasmtime component
/// host, writes the response JSON to stdout. Only compiled when the `wasm`
/// feature is on; the script path stays available for parity / fallback.
#[cfg(feature = "wasm")]
async fn run_wasm(skill_name: &str, skill_dir: &Path) -> Result<()> {
    use tokio::io::AsyncReadExt;
    let mut stdin_buf = Vec::new();
    tokio::io::stdin().read_to_end(&mut stdin_buf).await?;
    let input_json = String::from_utf8(stdin_buf)
        .context("wasm executor requires UTF-8 JSON input on stdin")?;

    // For Phase 5: granted = empty (operator approves separately via
    // `cyberos-skill cap`); cap-name list = an empty header-cap vector here,
    // populated from the manifest in a future cycle.
    let engine = cyberos_skill_host::make_engine()?;
    let header_caps: Vec<String> = Vec::new();
    let granted: Vec<cyberos_skill_host::Capability> = Vec::new();
    let out = cyberos_skill_host::run_component(
        &engine, skill_dir, skill_name, &granted, &input_json, header_caps,
    )?;
    print!("{}", out);
    use std::io::Write;
    std::io::stdout().flush()?;
    Ok(())
}

fn handle_cap(action: CapAction) -> Result<()> {
    use cyberos_skill_host::grants::{default_grants_path, load, save};
    let path = default_grants_path();
    match action {
        CapAction::List => {
            let g = load(&path).unwrap_or_default();
            if g.grants.is_empty() {
                println!("(no grants recorded; grants file: {})", path.display());
                return Ok(());
            }
            println!("Grants file: {}", path.display());
            let mut names: Vec<_> = g.grants.keys().collect();
            names.sort();
            for n in names {
                println!("\nSKILL: {}", n);
                let by_hash = &g.grants[n];
                let mut hashes: Vec<_> = by_hash.keys().collect();
                hashes.sort();
                for h in hashes {
                    let e = &by_hash[h];
                    println!(
                        "  sha256={} operator={} granted_at_unix_ms={}",
                        h, e.operator, e.granted_at_unix_ms
                    );
                    println!("    caps: {}", e.granted_caps.join(", "));
                }
            }
        }
        CapAction::Audit => {
            let g = load(&path).unwrap_or_default();
            let dangerous = ["bash", "shell", "exec"];
            let mut flagged = 0usize;
            for (skill, by_hash) in &g.grants {
                for (h, entry) in by_hash {
                    for cap in &entry.granted_caps {
                        if dangerous.iter().any(|d| cap == d || cap.starts_with(&format!("{}(", d))) {
                            println!(
                                "DANGEROUS  skill={} sha256={} cap={} operator={}",
                                skill, h, cap, entry.operator
                            );
                            flagged += 1;
                        }
                    }
                }
            }
            if flagged == 0 {
                println!("OK — no dangerous capabilities granted.");
            } else {
                println!("\n{} dangerous grant(s) flagged.", flagged);
            }
        }
        CapAction::Revoke { skill } => {
            let mut g = load(&path).unwrap_or_default();
            if g.grants.remove(&skill).is_some() {
                save(&path, &g)?;
                println!("Revoked all grants for '{}'.", skill);
            } else {
                println!("No grants found for '{}'.", skill);
            }
        }
    }
    Ok(())
}

/// Install a skill bundle into the target root (default ~/.cyberos/skills/).
///
/// Validates the SKILL.md, copies the directory recursively, and records the
/// install in `<target>/.installed.json`.
fn install_skill(skill_dir: &Path, target: Option<&Path>, force: bool) -> Result<()> {
    if !skill_dir.is_dir() {
        anyhow::bail!("skill_dir is not a directory: {}", skill_dir.display());
    }
    let skill_md_path = skill_dir.join("SKILL.md");
    let skill_md_bytes = std::fs::read(&skill_md_path)
        .with_context(|| format!("reading {}", skill_md_path.display()))?;
    let (manifest, _) = parse_frontmatter(&skill_md_bytes)
        .with_context(|| format!("parsing SKILL.md at {}", skill_md_path.display()))?;
    validate_manifest(&manifest)
        .with_context(|| format!("validating SKILL.md at {}", skill_md_path.display()))?;
    let dir_name = skill_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
    if dir_name != manifest.name {
        anyhow::bail!(
            "directory name '{}' must match SKILL.md name '{}'",
            dir_name, manifest.name
        );
    }

    // Resolve target root.
    let target_root = match target {
        Some(p) => p.to_path_buf(),
        None => default_install_root()?,
    };
    std::fs::create_dir_all(&target_root)
        .with_context(|| format!("creating target root {}", target_root.display()))?;

    let dest = target_root.join(&manifest.name);
    if dest.exists() {
        if !force {
            anyhow::bail!(
                "skill '{}' already installed at {} (pass --force to overwrite)",
                manifest.name, dest.display()
            );
        }
        std::fs::remove_dir_all(&dest)
            .with_context(|| format!("removing existing install at {}", dest.display()))?;
    }

    // Recursive copy.
    copy_dir_recursive(skill_dir, &dest)
        .with_context(|| format!("copying {} -> {}", skill_dir.display(), dest.display()))?;

    // Compute SHA-256 of SKILL.md for the receipt.
    let mut hasher = Sha256::new();
    hasher.update(&skill_md_bytes);
    let sha256_hex = format!("{:x}", hasher.finalize());

    // Read version from frontmatter metadata if present, else fall back to "0.0.0".
    let version = manifest
        .metadata
        .get("version")
        .cloned()
        .unwrap_or_else(|| "0.0.0".to_string());

    let installed_at_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    // Load + update the receipts file.
    let receipts_path = target_root.join(".installed.json");
    let mut receipts: serde_json::Map<String, serde_json::Value> = if receipts_path.is_file() {
        let bytes = std::fs::read(&receipts_path)
            .with_context(|| format!("reading {}", receipts_path.display()))?;
        let parsed: serde_json::Value = serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| serde_json::json!({}));
        parsed.as_object().cloned().unwrap_or_default()
    } else {
        serde_json::Map::new()
    };

    let source_path = std::fs::canonicalize(skill_dir).unwrap_or_else(|_| skill_dir.to_path_buf());
    let entry = serde_json::json!({
        "sha256": sha256_hex,
        "version": version,
        "source_path": source_path.display().to_string(),
        "installed_at_unix_ms": installed_at_unix_ms,
    });
    receipts.insert(manifest.name.clone(), entry);

    let receipts_serialized = serde_json::to_string_pretty(&serde_json::Value::Object(receipts))
        .context("serialising receipts")?;
    std::fs::write(&receipts_path, receipts_serialized)
        .with_context(|| format!("writing {}", receipts_path.display()))?;

    println!("Installed '{}' v{} -> {}", manifest.name, version, dest.display());
    println!("Receipt   : {}", receipts_path.display());
    println!("SHA-256   : {}", sha256_hex);
    Ok(())
}

fn default_install_root() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME env var not set; pass --target explicitly")?;
    Ok(PathBuf::from(home).join(".cyberos").join("skills"))
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in walkdir::WalkDir::new(src).min_depth(1) {
        let entry = entry?;
        let rel = entry.path().strip_prefix(src)?;
        let target_path = dst.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target_path)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), &target_path)?;
        }
        // Skip symlinks / other entry types — skill bundles should be plain dirs.
    }
    Ok(())
}

/// Locate `skill/tools/package.sh` by walking up from the CLI's cwd. Phase 1
/// approximation; Phase 5 will inline the logic.
fn locate_package_script() -> anyhow::Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let candidates = [
        cwd.join("skill/tools/package.sh"),
        cwd.join("tools/package.sh"),
        cwd.join("../tools/package.sh"),
        cwd.join("../../tools/package.sh"),
    ];
    for c in &candidates {
        if c.is_file() {
            return Ok(c.clone());
        }
    }
    anyhow::bail!("could not locate package.sh (looked in: {:?})", candidates);
}
