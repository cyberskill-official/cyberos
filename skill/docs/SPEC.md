# CyberOS Skill — protocol specification (v0.1, 2026-05-14)

> This document describes the protocol contract CyberOS skills MUST satisfy.
> The architectural reasoning lives in `AUDIT.md`. The day-by-day shipping
> record lives in `CHANGELOG.md`.

## §1 — The contract

CyberOS Skills are **valid [Anthropic Agent Skills](https://agentskills.io/specification)** verbatim. A skill authored for CyberOS works in Claude Code, OpenAI Codex CLI, Cursor, VS Code with GitHub Copilot, Goose, Amp, Gemini CLI, Mistral, Databricks, Letta, and 20+ other compliant clients without modification.

The protocol is **the Anthropic open standard, plus opt-in CyberOS extensions inside the spec-permitted `metadata` map.** CyberOS never adds top-level fields, never invents an alternative manifest format, and never accepts a non-SKILL.md descriptor.

## §2 — Skill structure (required)

A skill is a directory containing a `SKILL.md` file:

```
<skill-name>/
├── SKILL.md         ← required: frontmatter (YAML) + body (Markdown)
├── scripts/         ← optional: helper scripts (Level 3 disclosure)
├── references/      ← optional: detailed reference docs (Level 3)
├── assets/          ← optional: templates, fixtures (Level 3)
└── dist/skill.wasm  ← optional: compiled wasm32-wasi component (Phase 5+)
```

The directory name MUST equal `name` in the SKILL.md frontmatter.

## §3 — Frontmatter

YAML 1.2.2 between two `---` delimiters at the top of `SKILL.md`. Required fields per Anthropic spec:

| Field | Type | Required | Constraint |
|---|---|---|---|
| `name` | string | yes | 1–64 chars, `[a-z0-9-]+`, must equal directory name. Reserved: `anthropic`, `claude`. |
| `description` | string | yes | 1–1024 chars. MUST say both *what* the skill does and *when* to invoke it. Drives model-routing. |
| `license` | string | no | SPDX or free-form license reference |
| `compatibility` | string | no | ≤500 chars; free-form environment requirements |
| `metadata` | map<string,string> | no | Open spec — agent-specific extensions live here |
| `allowed-tools` | string \| list<string> | no | Capability declaration (experimental in spec; CyberOS treats as canonical) |

### §3.1 — CyberOS extensions (under `metadata`)

| Key | Type | Purpose |
|---|---|---|
| `version` | SemVer string | Required for registry-resolved skills (not parse-time) |
| `author` | string | Free-form attribution |
| `region` | ISO 3166-1 alpha-2 | Locale targeting (e.g. `VN` for the CyberSkill Vietnamese-market bundle) |
| `cyberos-caps` | string | Finer-grained capability syntax until the open spec's `allowed-tools` matures |

## §4 — Three-level progressive disclosure (mandatory)

Per the Anthropic spec:

| Level | Trigger | Cost | Content loaded |
|---|---|---|---|
| 1 — Startup | Host boot | ~100 tokens per skill, sub-ms parse | Frontmatter only |
| 2 — Activation | Controller decides description matches OR `activationEvents` fire | ≤5,000 tokens per activated skill | SKILL.md body |
| 3 — Execution | Body explicitly names a script/reference/asset | Per-resource | Referenced files only |

**Eager loading is forbidden.** The CyberOS host (Rust binary at `../crates/host/`) enforces Level-1-only at startup.

## §5 — Capability model

- Default grant set per skill is **empty**.
- Skills declare needs via `allowed-tools`.
- Operator approves on first use (recorded in `~/.cyberos/grants.json` by skill content hash).
- Modified skills (different hash) require re-approval.
- WASI capability bundle is computed at activation from the declared `allowed-tools` set.
- `bash`, `shell`, `exec` capabilities are auto-denied and require explicit operator opt-in.

## §6 — Distribution

Three channels, all resolving through the `Resolver` trait at `../crates/resolver/`:

1. **Local filesystem** — `~/.cyberos/skills/` (user-global) or `<project>/.cyberos/skills/` (project-scoped)
2. **OCI registry** — `oci://ghcr.io/org/skill-name:1.2.3` (Phase 5+)
3. **HTTPS URL** — direct download (Phase 5+; for hot-fix / signed bundles)

`.skill` bundles are zip-packed directories with a content hash and (Phase 6+) cosign signature. The resolver refuses unsigned bundles unless `--allow-unsigned` is passed.

## §7 — Versioning

The protocol is unversioned at the spec level (it tracks the Anthropic open standard). Individual skills carry SemVer in `metadata.version`. Breaking changes to the protocol require updating the Anthropic open spec upstream — CyberOS does not unilaterally evolve it.

The CyberOS host versions independently (see `../Cargo.toml` workspace version).

## §8 — Strategic posture

CyberSkill is a citizen of the open Agent Skills ecosystem. The differentiation play is **publishing high-quality Vietnamese-market skills** (VAT/e-invoice, VNeID integration, local bank APIs, Vietnamese legal/compliance, regional tax formatting) to the open registry. Inventing a competing format is strictly value-destroying — see `AUDIT.md` §2 and §7.

---

See `AUDIT.md` for the full architectural rationale, kill list, MVA, and 7-phase migration plan. See `CHANGELOG.md` for the day-by-day shipping record.
