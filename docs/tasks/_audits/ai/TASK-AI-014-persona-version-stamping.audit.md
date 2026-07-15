---
task_id: TASK-AI-014
audited: 2026-05-16
auditor: manual (engineering-spec template)
verdict: PASS (after revision)
score_pre_revision: 7.5/10        # the first-pass compressed version (287 lines)
score_post_expansion: 9.0/10      # after expanding to TASK-AI-012 depth (~860 lines)
score_post_revision: 10/10        # after 6 mechanical fixes
issues_open: 0
issues_resolved: 6
issues_critical: 0
template: engineering-spec@1
revised_at: 2026-05-16
---

## §1 — Verdict summary

TASK-AI-014 was expanded from 287 lines to ~860 lines matching TASK-AI-001 / TASK-AI-012 depth.

The expansion added 9 §1 normative clauses (#2 handle-keyed registry with `<id>@<version>` format and filename-match check, #6 explicit canonical-builder reference, #8 source-hash canonicalisation rules, #9 expanded downstream-artefact list including response body badge for EU AI Act Art. 50, #11 explicit non-duplication of policy check, #12 250ms file-watch debounce, #13 LLM-hint merge order, #14 strict semver parse, #16 reload INFO log), 7 substantive §2 rationale paragraphs (handle-vs-id keying, hash-on-every-load justification, ArcSwap-vs-RwLock contention math, system-message-vs-concatenation alignment argument, canonicalisation cross-platform false-positive rationale, override-order Bayesian frame, semver-strict scope discipline), full Rust type definitions in §3 (PersonaId, PersonaHandle, Persona, PersonaError variants with thiserror, PersonaInitError variants, PersonaParseError variants), full parse_persona_md skeleton with canonicalisation function, full handler injection skeleton, expanded §4 from 10 to 20 acceptance criteria, full Rust test bodies in §5 (happy + cache + Arc::ptr_eq, unknown-handle sorted-available, semver strictness, filename-mismatch, forbidden-frontmatter, double-init, tamper with metric assertion, hot-reload polling, parse-error cache-hold, canonicalisation CRLF + BOM + NFC tests, 100-concurrent + 1000-cache-hit budget, hint-merge), full registry + watch + hash + memory_writer-canonical-builder skeletons in §6, expanded §7 with code/concept/operational dependency split, 7 example payloads in §8 (request, injection, headers, body badge, audit row, unknown-handle, tamper response), 21 failure modes in §10 (vs. 7 in first pass), 9 implementation notes in §11.

Six residual issues prevented 10/10 at the post-expansion checkpoint; all six are mechanical and all six are resolved in this revision.

## §2 — Findings

### ISS-001 — Registry key was `PersonaVersion(String)` wrapping `"0.4.1"` but example used full handle `"cuo-cpo@0.4.1"`

- **severity:** error
- **rule_id:** consistency / type-vs-example mismatch
- **location:** §3 type sig, §4 example payloads
- **status:** resolved

#### Description

The first-pass §3 declared `pub struct PersonaVersion(pub String); // wraps "0.4.1"` — a bare semver wrapper. But §8 example payloads use `agent_persona: "cuo-cpo@0.4.1"` — a full handle. There is no documented mapping from the request field to the registry key; a code-gen agent reads the type signature and writes `registry.get(&PersonaVersion("0.4.1".into()))`, which loses the persona-id and silently picks the wrong persona if two ids share a version.

This is the same identity-collision class as TASK-AI-006 ISS-001 (province-code list duplicated across recognizers): the type doesn't carry enough information to unambiguously identify the entity.

#### Suggested fix

Introduce `PersonaHandle { id: PersonaId, version: Version }` as the registry key. Add `PersonaHandle::parse("cuo-cpo@0.4.1")` and `display() → "cuo-cpo@0.4.1"`. Update §1 #2, §3 types, §5 tests, §8 examples to use the handle uniformly. The bare `PersonaVersion(String)` type is removed.

### ISS-002 — `system_prompt` field allowed in BOTH frontmatter AND body; precedence undefined

- **severity:** error
- **rule_id:** spec-completeness / single-source-of-truth
- **location:** §1 #1, §3 file format
- **status:** resolved

#### Description

The first-pass §1 #1 said: *"The body of the file is the canonical system prompt (alternative to frontmatter `system_prompt`)."* The "alternative to" phrasing leaves precedence ambiguous: if both are present, which wins? Common patterns (newest-wins, frontmatter-wins, body-wins) all have advocates; the spec doesn't pick.

A code-gen agent implements one precedence rule; an operator who edits the frontmatter expecting it to override the body discovers the body wins (or vice versa) only after deployment. Worse: the frontmatter field gives the appearance of a knob ("oh, I can override the body via frontmatter") when in practice it's redundant or actively confusing.

#### Suggested fix

Make the body the **only** valid source. Reject frontmatter that contains `system_prompt:` at parse time with `PersonaInitError::ForbiddenFrontmatterField`. This eliminates the precedence question by deletion. Document the rule in §1 #1 and §11 (rationale: single source of truth; precedence ambiguity is solved by removing the duplicate).

Add `test_forbidden_frontmatter_system_prompt_rejected` in §5.

### ISS-003 — Source-hash canonicalisation undefined → CRLF↔LF flip false-positives as tampering

- **severity:** error
- **rule_id:** correctness / cross-platform false-positive
- **location:** §1 #7 (hash verify), §3 (no canonicalisation function)
- **status:** resolved

#### Description

The first-pass §1 #7 said "verify `source_hash` matches the cached body before injection." The `verify_hash` skeleton in §6 hashed `p.system_prompt.as_bytes()` directly. No canonicalisation rule.

Failure mode: a developer on Windows checks out the repo with `git config core.autocrlf true`. The persona file is checked out as CRLF; their save preserves CRLF. The Linux gateway, when it last hot-reloaded, hashed the LF version. On the next reload it hashes the CRLF version — different hash → `Tampered` error → sev-1 page → 503 to every persona-using request. The "incident" is a benign cross-platform git checkout.

This is not a hypothetical; it's the standard cross-platform bug class for any hash-of-text-file primitive.

#### Suggested fix

Add §1 #8 specifying the 5-step canonicalisation:
1. Strip leading BOM if present.
2. Normalise CRLF → LF.
3. Apply Unicode NFC normalisation (combining-form ↔ precomposed).
4. Right-trim trailing whitespace on each line.
5. Ensure exactly one terminating LF.

Add `canonicalise_body` in §3 + §6. Hash the canonicalised string, not the raw bytes. Add `test_canonicalisation_is_lf_normalised` and `test_canonicalisation_strips_bom_and_nfc_normalises` in §5. Add §11 note explaining this is a false-positive prevention measure, not a security weakening.

### ISS-004 — `ai.persona_loaded` audit row claimed but no canonical builder shown

- **severity:** error
- **rule_id:** spec-completeness / promise-vs-implementation
- **location:** §1 #6 (claim), §3 (no builder), §6 (no builder)
- **status:** resolved

#### Description

The first-pass §1 #6 said: *"emit one `ai.persona_loaded` memory audit row per request (via TASK-AI-003's `canonical::persona_loaded` builder)."* But (a) TASK-AI-003 declares the row *kind* (`ai.persona_loaded`) without specifying the payload schema, and (b) this task doesn't implement the builder.

A code-gen agent reading TASK-AI-014 cannot tell what fields go in the row's payload. The available signal is "TASK-AI-003 declares the kind" — but reading TASK-AI-003 reveals only the kind name, not the payload. The builder has to live somewhere; in the absence of explicit ownership, it lives nowhere and the row never gets emitted.

#### Suggested fix

This task owns the builder (since the row's data lives here — persona handle, source path, source hash). Add to §3 + §6:

```rust
pub mod canonical {
    pub fn persona_loaded(persona: &Persona, request_id: &str) -> AuditRow {
        AuditRow {
            kind: "ai.persona_loaded".into(),
            payload: serde_json::json!({
                "persona_id": persona.handle.id.0,
                "persona_version": persona.handle.version.to_string(),
                "persona_handle": persona.handle.display(),
                "source_path": persona.source_path,
                "source_hash": hex::encode(persona.source_hash),
                "request_id": request_id,
            }),
            ..Default::default()
        }
    }
}
```

Add the builder file path (`src/memory_writer.rs`) to `modified_files`. Add §8 example payload showing the row's full structure.

### ISS-005 — Hot-reload + `OnceCell::set` race; double-init not handled

- **severity:** warning
- **rule_id:** robustness
- **location:** §3 (`init_persona_registry`), §6 (skeleton)
- **status:** resolved

#### Description

The first-pass §6 used `REGISTRY.set(ArcSwap::from_pointee(map)).map_err(|_| PersonaInitError::AlreadyInitialised)`. Reasonable for production (init runs once at boot) but fragile for tests — every `#[tokio::test]` in the same process inherits the static state, so the second test's `init_persona_registry().await` returns `Err(AlreadyInitialised)` and pre-empts test setup.

Plus, the watcher's hot-reload path uses `REGISTRY.get().unwrap().store(Arc::new(new_map))` — which assumes init has run. If a hot-reload event fires *between* OnceCell construction and OnceCell::set (a microsecond window during boot), the unwrap panics.

This is the same pattern as TASK-AI-009 ISS-004 and TASK-AI-012 ISS-003 — `init` functions need either guard-against-double-call OR an explicit reset for tests.

#### Suggested fix

1. Add `PersonaInitError::AlreadyInitialised` (named clearly) and document the boot-once invariant.
2. Add `pub fn reset_for_tests()` that clears the OnceCell (test-only).
3. AC #20: explicit "double-init returns Err" assertion.
4. Add §10 row: "Registry init called twice (test re-entry, sidecar reload) → `AlreadyInitialised` returned; tests use `reset_for_tests()`."
5. Watcher must defensively check `REGISTRY.get()` and skip the reload (with WARN log) if init hasn't completed yet.

### ISS-006 — `PersonaVersion(pub String)` lacks parse validation; `0.4` and `0.4.1-alpha` slip through

- **severity:** warning
- **rule_id:** correctness / input validation
- **location:** §3 (type sig)
- **status:** resolved

#### Description

The first-pass §3 declared `pub struct PersonaVersion(pub String); // wraps "0.4.1" with parse validation` — but no parse function, no semver crate, no test. A `PersonaVersion("not-a-version".into())` is structurally valid; the only check would happen at filename match time (which doesn't exist either).

A request with `agent_persona = "cuo-cpo@latest"` would be looked up in the registry and miss (returning UnknownPersona). Functionally fine — but the error message says "unknown persona" when the real issue is "invalid version syntax." Operators see the wrong signal.

#### Suggested fix

1. Use the `semver` crate's `Version` type as the version field.
2. `PersonaHandle::parse("cuo-cpo@0.4")` returns `PersonaParseError::InvalidSemver` (missing patch).
3. `PersonaHandle::parse("cuo-cpo@0.4.1-alpha")` returns `PersonaParseError::PreReleaseUnsupported` (slice 3 scope discipline).
4. Add `test_semver_parse_rejects_pre_release_and_short_version` in §5.
5. Add §1 #14 normative requirement for strict semver.

## §3 — Strengths preserved through expansion

- §3 introduces `PersonaHandle`, `PersonaId`, `Version`-from-`semver` as distinct, parseable types — preventing the identity-collision class of bug from ISS-001 once and for all. The newtype pattern is consistent with TASK-AI-005's `TenantId` / `Region` types.
- §1 #3 commits to `ArcSwap` (not `RwLock`, not `DashMap`) with a paragraph in §2 explaining the contention math. Future engineers won't second-guess the choice.
- §1 #5 + §1 #7 explicitly preserve a caller's system message at index 1 instead of overwriting it; the AC test asserts the [persona, caller-system, user] sequence. Catches the "silent overwrite" regression class.
- §1 #11 explicitly defers persona-allow enforcement to TASK-AI-001 §1 #13; the spec doesn't duplicate the check, eliminating the dual-source-of-truth problem.
- §1 #13 documents the LLM-hints merge order (`request > persona > default`); the AC asserts it; future call-site overrides don't surprise anyone.
- §10 inventory grew from 7 rows to 21 — including the cross-platform line-ending row, the BOM row, the hot-reload-during-boot panic row, the watcher-thread-panic recovery row, and the editor-save-burst debounce row. Each row has an unambiguous detection mechanism.
- §11 documents the canonicalisation rationale ("not a security weakening — an attacker landing on the same hash means the content is unchanged") so future engineers don't strip the canonicalisation under the mistaken belief that it loosens the security boundary.

## §4 — Resolution

All 6 mechanical revisions applied (2026-05-16) within the task itself:

- **ISS-001 RESOLVED**: `PersonaHandle` introduced as `{ id: PersonaId, version: Version }`; bare `PersonaVersion(String)` removed; all examples, AC tests, and skeletons use the handle uniformly. Filename-match check added to enforce the `<handle>.md` invariant at parse time.

- **ISS-002 RESOLVED**: §1 #1 now says the body is the ONLY valid source; frontmatter `system_prompt:` is rejected with `PersonaInitError::ForbiddenFrontmatterField`; `test_forbidden_frontmatter_system_prompt_rejected` in §5 enforces the rule. AC #16 covers it.

- **ISS-003 RESOLVED**: §1 #8 added with the 5-step canonicalisation (BOM strip, CRLF→LF, NFC, line-trim, terminating LF); `canonicalise_body` shown in §3 + §6; tests for CRLF tolerance and BOM+NFC equivalence added in §5; §11 note added explaining "not a security weakening."

- **ISS-004 RESOLVED**: `canonical::persona_loaded` builder shown in §3 + §6 with full payload schema; `src/memory_writer.rs` listed in `modified_files`; §8 example payload shows the audit row's full JSON structure.

- **ISS-005 RESOLVED**: `PersonaInitError::AlreadyInitialised` documented; AC #20 asserts double-init returns Err; §10 row added; watcher defensively checks `REGISTRY.get()` before storing (otherwise WARN-and-skip).

- **ISS-006 RESOLVED**: `semver` crate integrated; `PersonaHandle::parse` rejects short versions with `InvalidSemver` and pre-releases with `PreReleaseUnsupported`; AC #17 enforces both; §1 #14 normative.

**Score = 10/10.** Ship as-is. Ready to transition `draft → accepted`.

---

*End of TASK-AI-014 audit (final). Status: PASS at 10/10.*
