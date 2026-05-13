# CyberOS BRAIN — Memory Protocol

A portable, local-first, append-only memory layer for AI-assisted work.
Drop `AGENTS.md` into any project; your agent (Claude / Cursor / Codex /
Copilot / Cowork) loads it and starts building a project-local **BRAIN**
you can copy, audit, and merge with teammates.

This README is a **step-by-step guide for newcomers**. The protocol
itself is in [`AGENTS.md`](AGENTS.md) (~373 lines, ~3.6k tokens).

---

## TL;DR — installing in a brand-new project

```bash
# From inside your new project (one-liner; runs the installer)
bash <(curl -fsSL https://raw.githubusercontent.com/CyberSkill/cyberos/main/scripts/install.sh) \
    --with-automation --with-pre-commit

# That's it. The agent now reads AGENTS.md and writes to .cyberos-memory/.
```

If you cloned the cyberos repo locally:

```bash
~/Projects/CyberSkill/cyberos/scripts/install.sh ~/Projects/my-new-project \
    --with-automation --with-pre-commit
```

The full eight steps below document what `install.sh` does, so you can
verify each piece and customise where needed.

---

## The four workflows this protocol supports

| # | Workflow | Status |
|---|---|---|
| 1 | **Solo, single machine** — one person, one laptop, agent auto-builds BRAIN | ✅ production-ready |
| 2 | **Solo, multi-machine** — copy `.cyberos-memory/` between your own machines | ✅ production-ready |
| 3 | **Multi-person, independent BRAINs** — each teammate has their own | ✅ production-ready |
| 4 | **Multi-person, merged** — pull selected memories from a teammate's BRAIN | ✅ production-ready (v2.1 — `cyberos import`) |

---

## Step-by-step: starting from zero

These eight steps take a fresh project from nothing to *"my agent
remembers everything I'm doing, automatically"*.

### Step 1 — Install the dependencies (one-time, per machine)

```bash
# Python runtime deps (required)
pip install msgspec cryptography crc32c rfc8785 pyyaml jsonschema zstandard \
    --break-system-packages

# pandoc (optional, only if you'll convert PRD/SRS docx ↔ md)
brew install pandoc        # macOS
# apt-get install pandoc   # Linux
```

Verify Python is ≥ 3.11:

```bash
python --version
```

### Step 2 — Copy the protocol files into your project

```bash
cd ~/Projects/my-new-project

mkdir -p docs/memory
cp ~/Projects/CyberSkill/cyberos/docs/memory/AGENTS.md             docs/memory/
cp ~/Projects/CyberSkill/cyberos/docs/memory/INTEROP.md            docs/memory/
cp ~/Projects/CyberSkill/cyberos/docs/memory/memory.schema.json    docs/memory/
cp ~/Projects/CyberSkill/cyberos/docs/memory/memory.invariants.yaml docs/memory/
```

That's all four normative files. Total ~30 KB. The rest of `docs/memory/`
in the cyberos repo (this README, EVOLUTION.md, CHANGELOG.md, etc.) is
*informative* — do not copy unless you want them.

You also need the `cyberos` Python package so `python -m cyberos`
works:

```bash
cp -r ~/Projects/CyberSkill/cyberos/cyberos ./cyberos
```

### Step 3 — Initialise the BRAIN

```bash
mkdir -p .cyberos-memory/{audit,memories/{decisions,facts,people,projects,preferences,drift,refinements},meta,company,module,member,client,project,persona,conflicts,exports,index}

cat > .cyberos-memory/manifest.json <<EOF
{
  "schema_version": 2,
  "project": {
    "root_path": "$(pwd)"
  }
}
EOF
```

### Step 4 — Wire `AGENTS.md` so your agent loads it

The agent needs to see `AGENTS.md` as its system prompt. The convention
differs by tool — symlink so any tool finds it:

```bash
# At the project root:
ln -s docs/memory/AGENTS.md AGENTS.md   # Codex CLI, agents.md ecosystem
ln -s docs/memory/AGENTS.md CLAUDE.md   # Claude Code

# For Cursor:
mkdir -p .cursor/rules
ln -s ../../docs/memory/AGENTS.md .cursor/rules/cyberos-memory.mdc

# For Cowork: set docs/memory/AGENTS.md as a system prompt in the UI.
```

### Step 5 — Verify

```bash
python -m cyberos --store .cyberos-memory state    # should print: READY
python -m cyberos --store .cyberos-memory doctor   # should print: 15 pass, 0 error
```

If `doctor` reports any error, fix it before continuing. The most common
issue is a non-canonical directory at the BRAIN root — `doctor --repair`
auto-resharrds them.

### Step 6 — Install host-side automation (macOS)

```bash
~/Projects/CyberSkill/cyberos/scripts/automation-install.sh \
    --target ~/Projects/my-new-project
```

That installs two LaunchAgents:

* **Nightly** (01:09 local) — `cyberos doctor` + consolidate dry-run.
  Logs to `~/Library/Logs/cyberos/nightly.log`. Notifies on failure.
* **Weekly** (02:07 Sunday) — `cyberos backup` + `cyberos consolidate`
  + determinism guard. Backups land in `~/cyberos-backups/<project>/`.

To test the nightly immediately:

```bash
launchctl start world.cyberskill.cyberos.nightly
tail ~/Library/Logs/cyberos/nightly.log
```

To uninstall:

```bash
~/Projects/CyberSkill/cyberos/scripts/automation-install.sh --uninstall
```

### Step 7 — Install the git pre-commit hook

```bash
~/Projects/CyberSkill/cyberos/scripts/install-pre-commit.sh \
    ~/Projects/my-new-project
```

Now `git commit` refuses to commit changes that would corrupt the
BRAIN (doctor failure, schema-invalid memory, schema-drift).

### Step 8 — Use it

There is nothing else to set up. Open your project in the agent of your
choice. Work normally. The agent reads `AGENTS.md`, understands the
protocol, and uses `python -m cyberos put / move / delete / view` to
write to `.cyberos-memory/` as your work progresses.

You can verify the BRAIN's state at any time:

```bash
python -m cyberos --store .cyberos-memory state
python -m cyberos --store .cyberos-memory audit head     # number of audit rows
python -m cyberos --store .cyberos-memory search "design decision"
```

---

## Daily operation

Once installed, you have nothing to do. The agent writes; the
automation guards. Day-to-day commands you might run manually:

```bash
# How many audit rows? What's the chain head?
python -m cyberos --store .cyberos-memory audit head

# Full status
python -m cyberos --store .cyberos-memory state

# Search
python -m cyberos --store .cyberos-memory search "decision about X"

# View a memory file
python -m cyberos --store .cyberos-memory view memories/decisions/X.md

# Add a memory manually (rare; agent usually does this)
python -m cyberos --store .cyberos-memory --actor stephen put \
    memories/preferences/dark-mode.md body.md
```

---

## Workflow 2 — Copy `.cyberos-memory` between your own machines

```bash
# On machine A — produce a deterministic, audit-verifiable bundle
python -m cyberos --store .cyberos-memory export ~/Drive/cyberos-A.zip

# Optional: copy the signing key so STH lineage stays single-identity
rsync -a ~/.config/cyberos/ user@machine-B:~/.config/cyberos/

# On machine B — extract + verify
mkdir -p ~/Projects/<same-project>/.cyberos-memory
unzip ~/Drive/cyberos-A.zip -d ~/Projects/<same-project>/.cyberos-memory
python -m cyberos --store .cyberos-memory doctor
```

Or, point both machines at a synced folder (iCloud, Dropbox, Syncthing)
and you don't even need the export step — the BRAIN itself is
sync-safe. **Just keep the index out of the synced area**: the SQLite
cache lives at `~/Library/Caches/cyberos/` and stays per-host
automatically.

---

## Workflow 4 — Merging your teammate's BRAIN into yours

Alice exports her shareable memories; Bob imports them with full
control over filters and conflict resolution:

```bash
# Alice
python -m cyberos --store .cyberos-memory export alice-shared.zip

# Bob — see what would import without writing
python -m cyberos --store .cyberos-memory import alice-shared.zip \
    --filter sync_class=shareable \
    --dry-run

# Bob — actually import
python -m cyberos --store .cyberos-memory import alice-shared.zip \
    --filter sync_class=shareable \
    --map-actor alice:alice@cyberskill.world \
    --on-conflict branch
```

Filters available: `kind=`, `sync_class=`, `actor=`, `classification=`.
Conflict policies: `skip` (default), `overwrite`, `branch`
(creates `<path>.from-<short-fp>.md`).

**Idempotent**: Bob can re-run the same `cyberos import` next week and
only Alice's NEW memories will pull (tracked via
`manifest.imports.<fingerprint>.last_imported_seq`).

**Audit-bracketed**: the import is recorded on Bob's chain as
`session.start` → N × `op="put"` (carrying `extra.imported_from` and
`extra.foreign_chain`) → `session.end`.

---

## The CLI in one screen

```
cyberos --store <path> <verb> [args]

  Six canonical file ops (any agent will use these):
    put <path> <body_file>                      create or replace
    move <src> <dst>                            rename (preserves content hash)
    delete <path> [--mode tombstone|purge]      tombstone (default) or GDPR purge
    view <path>                                 read (implicit; no audit row)
    create / str-replace / insert / rename      v1 aliases — still accepted

  Audit + integrity:
    state                                       READY / FROZEN_RECOVERABLE / FROZEN_HUMAN
    doctor [--repair]                           15 invariants; --repair auto-fixes safe ones
    verify                                      walk Merkle chain
    audit dump|head                             inspect ledger
    consolidate [--dry-run]                     Walk → Compact → Sign → Publish
    prove <seq> --out p.json                    Merkle inclusion proof
    verify-proof p.json                         re-verify a proof
    sth-wrap --passphrase-file <f>              passphrase-wrap signing key
    validate <path>                             check a memory file against the schema

  Sharing + storage:
    export <out.zip>                            deterministic portable bundle
    import <source>                             cross-BRAIN merge (P6)
    backup --target <dir>                       incremental hard-link snapshot
    prune --soak-days 30                        sweep .zst-archived originals
    search <query>                              FTS5 over memory bodies
    checkpoint                                  force F_FULLFSYNC barrier
```

Cold `cyberos --help` is ~14 ms (lazy imports). 22 subcommands total.

---

## Folder contents (this directory)

| File | Status | What it is |
|---|---|---|
| `AGENTS.md` | normative, v2.0.0 | Protocol spec. **Load as system prompt.** ~3.6k tokens. |
| `memory.schema.json` | machine | JSON Schema for frontmatter / manifest / audit row. |
| `memory.invariants.yaml` | walker input | 15 invariants the doctor / state machine check. |
| `INTEROP.md` | normative subset | ≤6 KB. Minimum profile for non-ledger agents. |
| `EVOLUTION.md` | informative | Stage / Bundle history; **do not load per session**. |
| `PROPOSAL.md` | informative | Outstanding proposals. |
| `P2_RESOLUTION.md` | informative | Resolved P2 design questions. |
| `LEGACY_SCRIPTS.md` | informative | Status of the 59 legacy `runtime/tools/cyberos_*.py` scripts. |
| `CHANGELOG.md` | append-mostly | Dated history. |
| `README.md` | this file | Step-by-step newcomer guide. |

The **four normative + two machine** files are the protocol — those
are what you copy into other projects. Everything else is informative
or historical.

---

## Why this protocol is safe to apply broadly

* **Self-contained.** `AGENTS.md` + `.cyberos-memory/` per project. No
  external services, no daemons, no cloud dependencies.
* **Local-first.** Everything on your filesystem. Nothing leaves your
  machine unless you export it.
* **Auditable.** Every mutation is a Merkle-chained audit row. Signed
  Tree Heads (Ed25519) anchor each consolidation; passphrase-wrapped
  signing key on disk.
* **Portable.** `cyberos export` produces byte-identical zips across
  runs. Drop the protocol in any project; remove cleanly any time.
* **Agent-agnostic.** Same `AGENTS.md` works for Claude / Cursor /
  Codex / Copilot / Cowork.
* **Privacy-respecting.** `delete(mode="purge")` honours GDPR Article
  17. `sync_class=private` (default) never leaves the local store.
* **Team-friendly.** `cyberos import` selectively merges teammates'
  BRAINs with filter + conflict-resolution + idempotent re-import.

---

## Troubleshooting

| Symptom | Fix |
|---|---|
| `doctor` reports `layout-root-canonical` ERROR | Run `cyberos doctor --repair`; if v1 debris is at root, run `~/Projects/CyberSkill/cyberos/scripts/cleanup-v1.sh --apply`. |
| `doctor` reports `crypto-crc-implementation` WARN | `pip install crc32c` to enable the hardware-accelerated path. |
| `consolidate` says `STH signing requires 'cryptography'` | `pip install cryptography`. |
| `consolidate` says `zstandard not installed` | `pip install zstandard`. |
| Nightly soak ran but didn't update anything | If it ran in a Cowork sandbox, disable the Cowork scheduled task and use the macOS LaunchAgent instead (Step 6). |
| `import` says "no matching memories" | Check filter syntax (`key=value`); confirm the source actually has memories matching. Try `--dry-run` without filters first. |
| Pre-commit hook blocks a legitimate commit | Bypass once with `git commit --no-verify`. Then fix the underlying issue (usually a schema-drift; regen with `python -m runtime.tools.cyberos_generate_schema --out docs/memory/memory.schema.json`). |

---

## Where to go next

* **For protocol details:** [`AGENTS.md`](AGENTS.md) — v2 normative spec.
* **For history + audit reports:** [`EVOLUTION.md`](EVOLUTION.md).
* **For implementation reference:** [`../../cyberos/README.md`](../../cyberos/README.md).
* **For pending proposals:** [`PROPOSAL.md`](PROPOSAL.md) — P2 Stage 3 is the only one.

---

*This file is an explainer; the contract itself is `AGENTS.md`. If
anything here contradicts `AGENTS.md`, the protocol wins.*
