---
memory_id: __MEM_ID__
scope: meta
classification: operational
authority: human-confirmed
version: 1
created_at: __ISO_TS__
created_by: subject:stephen-cheng
last_updated_at: __ISO_TS__
updated_by: subject:stephen-cheng
provenance:
  source: manual
  source_ref: bundle-q-protocol-upgrade-71a276c74fe5a1fb
  confidence: 1.0
tags: [refinement, bundle-q, section-0-6, section-4-7, section-13-1, section-15, implementation-files, close-pattern, gitignore-warn, relative-symlinks]
---

# REF-041 — Bundle Q: implementation files in source tree, §4.7 close-pattern alignment, BRAIN-not-versioned warn, relative symlinks

## Trigger

Real-world trigger 2026-05-11 during a cowork session that started with the user asking *"did you load protocol?"* and immediately surfaced the missing `brain_writer.py` — prescribed by 8 docs but never tracked in git, never on disk.

A Phase-1 BRAIN repair (rebuilt the writer from AGENTS.md §4 / §5.2 / §7 / §13 directly) and a Phase-2 repo audit produced four refinements in one bundle:

- **REF-1 / §0.6 line 175** — implementation files MUST live in project source tree, not in `.cyberos-memory/`. Three different prescribed locations existed (`outputs/`, `<cyberos-memory>/`, "PRD §5.10.11"); only one would have resolved on disk if the file existed; only one is git-trackable when the BRAIN is gitignored.
- **REF-2 / §4.7** — post-terminator close exemption added. The existing 357-row chain ended `session.end → str_replace manifest.json` (the canonical close pattern), which the pre-Q wording flagged as `crash-mid-manifest-update`. Without the exemption, every clean session triggers a freeze on the next reconciliation.
- **REF-3 / §13.1 step 11** — BRAIN-not-versioned warn. Pre-Q step 11 only handled the *opt-in* default (commented `.gitignore` line); the *opt-out* state (uncommented entry, full ignore) had no audit trail. This is exactly how the previous `brain_writer.py` vanished without anyone noticing. Bundle Q records the opt-out as a deduplicated `op:"warn" reason:"brain-not-versioned"` audit row and adds a `.gitignore` comment block documenting the deliberate intent.
- **REF-4 / §15** — relative-symlink rule. `<root>/AGENTS.md` was found to be an absolute-path symlink (`/Users/stephencheng/...`), broken under cowork's bind-mount and any other container/CI mount. Bundle Q mandates relative paths for all project-root convenience symlinks.

Direct §0.4 standing-rule trigger ("missing artefact OR missing audit trail surfaced from real-world failure"). All four refinements adopted in the same chat turn that surfaced them.

## Adoption

Adopted via §0.5 chat-turn approval (2026-05-11):

> approve protocol upgrade to sha256:71a276c74fe5a1fb65dbe24c6073f74d4cc7168b02aef1b577db9e01ccb13688

## Changes (textual diff against §617f5aef → §71a276c7)

- **§0.6 line 175** — appended a clause: *"Implementation files MUST live in the project source tree (versioned in git), NOT inside `.cyberos-memory/`."* Names `outputs/brain_writer.py` as canonical; allows alternates if §0.6 registry is updated.
- **§4.7 orphan-manifest-update bullet** — added "Post-terminator close exemption" defining the legitimate `session.end → str_replace manifest.json` close pattern (manifest update's `prev_chain` matches preceding terminator's `chain` AND new `audit_chain_head` value equals that terminator's `chain`).
- **§13.1 step 11** — replaced single-line `.gitignore` instruction with a two-branch decision tree (default versioning-opt-in available vs explicit opt-out). Opt-out branch appends one-time `op:"warn" reason:"brain-not-versioned"` deduplicated by `(reason, path)` and updates `.gitignore` with a comment block documenting deliberate intent.
- **§15 first paragraph** — added: *"All such symlinks MUST use relative paths"* with example showing relative vs absolute, plus rationale ("absolute-path symlinks break under container/CI/sandbox mounts where the host prefix differs and silently degrade portability").

## Verification

- Live AGENTS.md canonical SHA: `sha256:71a276c74fe5a1fb65dbe24c6073f74d4cc7168b02aef1b577db9e01ccb13688` (matches manifest pin)
- Pre-edit AGENTS.md (recoverable via `git show HEAD~1:docs/CyberOS-AGENTS.md` after this bundle's archive commit) hashes to `sha256:617f5aef1a49c394f6d17be072c8b29dbeb84c3265b80f3de3cb00a0f1c07759` (matches old pin)
- New `outputs/brain_writer.py` produces bit-perfect chain hashes for the last 5 rows of the 357-row pre-Q chain (post-Bundle-D writer compatibility)
- Chain LINK invariant: 0 breaks across full chain
- §4.7 reconciliation against the existing chain's `session.end → str_replace manifest.json` close pattern now passes (no false-positive crash flag)

## How to use this memory

When future protocol changes touch §0.6 (implementation files), §4.7 (reconciliation), §13.1 (bootstrap step 11), or §15 (multi-agent interop / symlinks), cross-link those changes back to REF-041 if the new change interacts with any of the four amendments here. Use the §4.7 post-terminator close exemption mentally when reading the audit ledger: a manifest update as the very last row is usually fine if the row before it is `session.end | consolidation_run | protocol_upgrade | protocol_rollback` and the chain values match.

## History

- 2026-05-11 — REF-041 created as part of Bundle Q (sha transition `617f5aef…07759` → `71a276c7…3688`). Real-world trigger: missing `brain_writer.py` discovered during cowork session.
