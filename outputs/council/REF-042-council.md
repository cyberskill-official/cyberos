# Council session — REF-042

**Subject REF:** `.cyberos-memory/memories/refinements/REF-042-brain-writer-recompute-memory-count.md`
**Generated:** 2026-05-12T12:15:58+07:00
**Voices:** architect, skeptic, pragmatist, critic

## Heuristic context (deterministic, no LLM)

**Tags on this REF:** refinement, brain-writer, manifest-count, bug-fix, tier-2

**GLOSSARY terms used:**
- **brain** — Whole system: Layer 1 + Layer 2 + Layer 3. Case-sensitive uppercase only per Section 0.3.
- **bundle** — Named cohort of related protocol amendments shipped together. Single letter (A-Q so far).
- **drift candidate** — Section 8.6 signal that a source file's SHA changed since digest was written. Three responses: re-ingest, accept drift, update source.
- **manifest** — `.cyberos-memory/manifest.json`. Per-project root pointer.
- **refinement** — Protocol amendment proposed per Section 0.4. Recorded in `memories/refinements/REF-NNN-<slug>.md`.

_(no obvious locked-decisions.md conflicts)_

---

## Voice prompts (feed each to a fresh Claude conversation)

### Voice: Architect

**Prompt to feed Claude (or another LLM):**

```
You are reviewing a CyberOS BRAIN refinement (REF-042). Your role is
the **Architect**. Evaluate this REF strictly through the lens of
structural fit with the existing AGENTS.md protocol.

Read the REF body below. Answer:
1. Which AGENTS.md section does this modify (§-number)? Is the change
   confined to one section or does it ripple through multiple?
2. Are the field/schema additions backward-compatible? Would a pre-REF
   manifest fail to validate post-REF?
3. Does this introduce new invariants? List them. Are existing
   invariants still satisfied?
4. What is the smallest set of code changes that delivers the
   capability described? (Architect-minimal, not pragmatist-minimal.)
5. Are there protocol-amendment bundles already in flight that would
   conflict? (Check AGENTS.CHANGELOG.md.)

REF body:
---
# REF-042 brain_writer.session-end recomputes memory_count

## Trigger
cyberos status dashboard surfaced manifest.memory_count drift of 77 between manifest field and on-disk file count. Investigation: brain_writer.cmd_session_end never recomputes the field. Bug accumulates with every successful op:create.

## Tier
TIER 2 (standard — affects manifest field semantics, not protocol mechanism)

## AGENTS.md section
§4.7 reconciliation (session-end touches manifest); §6 (manifest schema field semantics)

## Exact prose to insert in AGENTS.md §6 (manifest fields table)
`memory_count` row clarification:
"Recomputed at every session.end by walking .cyberos-memory/*.md (excluding cache + hidden files). MAY be stale between session.end calls; cyberos status surfaces drift if real ≠ recorded."

## Capability eval
- **What new behavior:** session-end MUST recompute memory_count; cyberos status no longer surfaces drift bottleneck post-session-end
- **Test fixture:** runtime/tests/refinements/REF-042/capability.test.py
- **Pass criteria:** start fresh BRAIN, write 3 memories, session-end; manifest.memory_count == 3 (currently fails: stays at 0 or pre-write value)

## Regression eval
- **What to verify:** all existing 152 memories still validate; chain LINK invariant preserved; no perf regression > 100ms on session-end
- **Test fixture:** runtime/tests/refinements/REF-042/regression.test.py
- **Pass criteria:** cyberos verify returns 0 CRITICAL after fix

## Implements decision
DEC-110

## Implementation steps
1. In brain_writer.cmd_session_end, after Step 2 (manifest construction), add:
```python
real_count = sum(1 for _ in brain_root.rglob("*.md") if _.is_file() and not _.name.startswith("."))
new_manifest["memory_count"] = real_count
```
2. Move BEFORE the json.dumps so it's serialised in the same str_replace
3. Add test fixture exercising the case (write N memories, session-end, assert count == N)
4. Document in CHANGELOG as Bundle R or similar
5. Capability + regression eval both must pass before §0.5 protocol-upgrade approval

## Related
- Bundle: TBD (proposed for next bundle)
- DEC: 110
- Drift candidate that surfaced this: ~/.cyberos-memory/memories/drift/2026-05-12-refinement-candidate-repeated-revert-any.md (different pattern but same dashboard)
---

Respond in ≤300 words. No em dashes. No AI-vocab. Plain English.
```

**Council finding (live, claude-sonnet-4-6):**

_(anthropic SDK not installed; paste prompt into a fresh Claude conversation by hand)_

---

### Voice: Skeptic

**Prompt to feed Claude (or another LLM):**

```
You are reviewing a CyberOS BRAIN refinement (REF-042). Your role is
the **Skeptic**. Find what breaks.

Read the REF body below. Answer:
1. What is the worst case this REF enables? (Adversarial user, buggy
   tool, accidental data loss, malicious co-author.)
2. What perverse incentive does this create for future authors? (Does
   it reward gaming a metric, hiding mistakes, skipping reconciliation?)
3. What happens at scale? (N=1,000 memories. N=100,000. Multiple
   subjects writing concurrently.)
4. What edge case is the author NOT thinking about? (Power-loss mid
   write; clock skew; symlinks; UTF-8 normalisation; ACL changes.)
5. What is the rollback path if this REF turns out wrong 3 months
   later? Is the change reversible?

REF body:
---
# REF-042 brain_writer.session-end recomputes memory_count

## Trigger
cyberos status dashboard surfaced manifest.memory_count drift of 77 between manifest field and on-disk file count. Investigation: brain_writer.cmd_session_end never recomputes the field. Bug accumulates with every successful op:create.

## Tier
TIER 2 (standard — affects manifest field semantics, not protocol mechanism)

## AGENTS.md section
§4.7 reconciliation (session-end touches manifest); §6 (manifest schema field semantics)

## Exact prose to insert in AGENTS.md §6 (manifest fields table)
`memory_count` row clarification:
"Recomputed at every session.end by walking .cyberos-memory/*.md (excluding cache + hidden files). MAY be stale between session.end calls; cyberos status surfaces drift if real ≠ recorded."

## Capability eval
- **What new behavior:** session-end MUST recompute memory_count; cyberos status no longer surfaces drift bottleneck post-session-end
- **Test fixture:** runtime/tests/refinements/REF-042/capability.test.py
- **Pass criteria:** start fresh BRAIN, write 3 memories, session-end; manifest.memory_count == 3 (currently fails: stays at 0 or pre-write value)

## Regression eval
- **What to verify:** all existing 152 memories still validate; chain LINK invariant preserved; no perf regression > 100ms on session-end
- **Test fixture:** runtime/tests/refinements/REF-042/regression.test.py
- **Pass criteria:** cyberos verify returns 0 CRITICAL after fix

## Implements decision
DEC-110

## Implementation steps
1. In brain_writer.cmd_session_end, after Step 2 (manifest construction), add:
```python
real_count = sum(1 for _ in brain_root.rglob("*.md") if _.is_file() and not _.name.startswith("."))
new_manifest["memory_count"] = real_count
```
2. Move BEFORE the json.dumps so it's serialised in the same str_replace
3. Add test fixture exercising the case (write N memories, session-end, assert count == N)
4. Document in CHANGELOG as Bundle R or similar
5. Capability + regression eval both must pass before §0.5 protocol-upgrade approval

## Related
- Bundle: TBD (proposed for next bundle)
- DEC: 110
- Drift candidate that surfaced this: ~/.cyberos-memory/memories/drift/2026-05-12-refinement-candidate-repeated-revert-any.md (different pattern but same dashboard)
---

Respond in ≤300 words. Be specific. Name the failure mode. No em
dashes. No AI-vocab.
```

**Council finding (live, claude-sonnet-4-6):**

_(anthropic SDK not installed; paste prompt into a fresh Claude conversation by hand)_

---

### Voice: Pragmatist

**Prompt to feed Claude (or another LLM):**

```
You are reviewing a CyberOS BRAIN refinement (REF-042). Your role is
the **Pragmatist**. Find the smallest possible variant.

Read the REF body below. Answer:
1. What is the 80/20 cut? Could 80% of the value ship in 20% of the
   effort by trimming a specific scope?
2. Which steps in "Implementation steps" can be deferred to a follow-up
   REF without losing the core?
3. Could this be a runtime tool / hook instead of a protocol change?
   (Tools are cheaper to revert than protocol amendments.)
4. Estimate implementation effort: <1h, <4h, <1d, <1wk, >1wk. Show
   your reasoning.
5. Is there a 3-line fix that gets the user unblocked TODAY while the
   full REF lands later?

REF body:
---
# REF-042 brain_writer.session-end recomputes memory_count

## Trigger
cyberos status dashboard surfaced manifest.memory_count drift of 77 between manifest field and on-disk file count. Investigation: brain_writer.cmd_session_end never recomputes the field. Bug accumulates with every successful op:create.

## Tier
TIER 2 (standard — affects manifest field semantics, not protocol mechanism)

## AGENTS.md section
§4.7 reconciliation (session-end touches manifest); §6 (manifest schema field semantics)

## Exact prose to insert in AGENTS.md §6 (manifest fields table)
`memory_count` row clarification:
"Recomputed at every session.end by walking .cyberos-memory/*.md (excluding cache + hidden files). MAY be stale between session.end calls; cyberos status surfaces drift if real ≠ recorded."

## Capability eval
- **What new behavior:** session-end MUST recompute memory_count; cyberos status no longer surfaces drift bottleneck post-session-end
- **Test fixture:** runtime/tests/refinements/REF-042/capability.test.py
- **Pass criteria:** start fresh BRAIN, write 3 memories, session-end; manifest.memory_count == 3 (currently fails: stays at 0 or pre-write value)

## Regression eval
- **What to verify:** all existing 152 memories still validate; chain LINK invariant preserved; no perf regression > 100ms on session-end
- **Test fixture:** runtime/tests/refinements/REF-042/regression.test.py
- **Pass criteria:** cyberos verify returns 0 CRITICAL after fix

## Implements decision
DEC-110

## Implementation steps
1. In brain_writer.cmd_session_end, after Step 2 (manifest construction), add:
```python
real_count = sum(1 for _ in brain_root.rglob("*.md") if _.is_file() and not _.name.startswith("."))
new_manifest["memory_count"] = real_count
```
2. Move BEFORE the json.dumps so it's serialised in the same str_replace
3. Add test fixture exercising the case (write N memories, session-end, assert count == N)
4. Document in CHANGELOG as Bundle R or similar
5. Capability + regression eval both must pass before §0.5 protocol-upgrade approval

## Related
- Bundle: TBD (proposed for next bundle)
- DEC: 110
- Drift candidate that surfaced this: ~/.cyberos-memory/memories/drift/2026-05-12-refinement-candidate-repeated-revert-any.md (different pattern but same dashboard)
---

Respond in ≤300 words. Concrete. No em dashes. No AI-vocab.
```

**Council finding (live, claude-sonnet-4-6):**

_(anthropic SDK not installed; paste prompt into a fresh Claude conversation by hand)_

---

### Voice: Critic

**Prompt to feed Claude (or another LLM):**

```
You are reviewing a CyberOS BRAIN refinement (REF-042). Your role is
the **Critic**. Audit the writing itself against the gstack /codex
voice standard and AGENTS.md §0.4 (propose-adopt-record cycle).

Read the REF body below. Answer:
1. Does the REF cite a TRIGGER concretely (a real incident, not a
   hypothetical)?
2. Does it state TIER (1/2/3) and justify it?
3. Does the implementation-steps section give exact code or exact
   file/section edits? (Vague language → bounce.)
4. Does the language follow the gstack /codex voice? (No em dashes;
   no AI-vocab like "leverage", "robust", "ensure", "comprehensive",
   "seamless", "delve", "navigate", "tapestry"; no marketing.)
5. List every voice-standard violation with line number.

REF body:
---
# REF-042 brain_writer.session-end recomputes memory_count

## Trigger
cyberos status dashboard surfaced manifest.memory_count drift of 77 between manifest field and on-disk file count. Investigation: brain_writer.cmd_session_end never recomputes the field. Bug accumulates with every successful op:create.

## Tier
TIER 2 (standard — affects manifest field semantics, not protocol mechanism)

## AGENTS.md section
§4.7 reconciliation (session-end touches manifest); §6 (manifest schema field semantics)

## Exact prose to insert in AGENTS.md §6 (manifest fields table)
`memory_count` row clarification:
"Recomputed at every session.end by walking .cyberos-memory/*.md (excluding cache + hidden files). MAY be stale between session.end calls; cyberos status surfaces drift if real ≠ recorded."

## Capability eval
- **What new behavior:** session-end MUST recompute memory_count; cyberos status no longer surfaces drift bottleneck post-session-end
- **Test fixture:** runtime/tests/refinements/REF-042/capability.test.py
- **Pass criteria:** start fresh BRAIN, write 3 memories, session-end; manifest.memory_count == 3 (currently fails: stays at 0 or pre-write value)

## Regression eval
- **What to verify:** all existing 152 memories still validate; chain LINK invariant preserved; no perf regression > 100ms on session-end
- **Test fixture:** runtime/tests/refinements/REF-042/regression.test.py
- **Pass criteria:** cyberos verify returns 0 CRITICAL after fix

## Implements decision
DEC-110

## Implementation steps
1. In brain_writer.cmd_session_end, after Step 2 (manifest construction), add:
```python
real_count = sum(1 for _ in brain_root.rglob("*.md") if _.is_file() and not _.name.startswith("."))
new_manifest["memory_count"] = real_count
```
2. Move BEFORE the json.dumps so it's serialised in the same str_replace
3. Add test fixture exercising the case (write N memories, session-end, assert count == N)
4. Document in CHANGELOG as Bundle R or similar
5. Capability + regression eval both must pass before §0.5 protocol-upgrade approval

## Related
- Bundle: TBD (proposed for next bundle)
- DEC: 110
- Drift candidate that surfaced this: ~/.cyberos-memory/memories/drift/2026-05-12-refinement-candidate-repeated-revert-any.md (different pattern but same dashboard)
---

Respond in ≤300 words. Be picky.
```

**Council finding (live, claude-sonnet-4-6):**

_(anthropic SDK not installed; paste prompt into a fresh Claude conversation by hand)_

---

## Synthesis (author fills after collecting voices)

After all 4 voices have weighed in, the author writes the synthesis:

**Verdict:** [ACCEPT as written / ACCEPT with modifications / DEFER / REJECT]

**Modifications:** _(if ACCEPT-with-mods, list every change to the REF
body in diff form)_

**Why this verdict:** _(1-2 paragraphs)_

**Cross-references to council voices:**
- Architect raised: ...
- Skeptic raised: ...
- Pragmatist raised: ...
- Critic raised: ...

**Next action:** _(specific action: e.g. "amend REF body, re-run
`cyberos verify`, then `cyberos eval REF-042`")_

---

_This file is a working artefact at outputs/council/REF-042-council.md.
It is NOT a memory in the BRAIN. Once synthesis is complete and the
REF body is updated, this file can be archived or deleted._
