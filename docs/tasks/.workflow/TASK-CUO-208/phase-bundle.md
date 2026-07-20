# TASK-CUO-208 phase bundle

## repo-context-map (step 1)
Two template ecosystems coexist: 470 engineering-spec files (repo-native, §-grammar, TRACE per RUBRIC §9) vs 6 task@1 files (template: key, FM/SEC/COND families). Normative homes found: author §12 (engineering-spec authoring), RUBRIC.md §2-§4 + §9 (task fields/sections + TRACE scope), command doc step 1 (the wording to fix), author input envelope (full pair - field added there).

## edge-case matrix (step 5) -> covering case
NULL: no config + no override -> default engineering-spec@1 + PLAN echo (profile doc chain). MALFORMED/AMBIGUOUS: both markers -> needs_human template_ambiguous (fixture TC-03); neither -> same. BOUNDS: none numeric. RACE: none. SECURITY: detection from file bytes only - a prose body containing '## Summary' cannot flip families (detection requires the template: KEY; § grammar is the other side's tiebreak - audit ISS/§10 #4). DEGRADATION: mixed batch -> per-file judgment (TC-04); rule drift -> families cited by NAME so intra-family additions inherit (§10 #3).

## implementation (steps 6-14)
TEMPLATE_PROFILES.md (verification preamble, resolution chain, detection, both profiles side by side, family lists by name); author envelope template field (enum, resolution documented); author SKILL.md template-profiles section (additive); audit SKILL.md detection+family-switch section (additive); RUBRIC.md §10 detection preamble (FM-004 untouched per §11); command doc names the chain, no longer asserts one format. Acceptance: TEMPLATE_CASES.md TC-01..05 with inline fixtures; both TRIGGER_TESTS extended in native list format (P5/N5 each).

## code review vs §1 (steps 16-18)
#1 profiles complete PASS (both frontmatter sets, grammars, end markers, family lists); #2 resolution chain + PLAN echo PASS (command doc + profile doc); #3 author input field + faithful emission PASS (envelope + SKILL.md + TC-05); #4 family switch on detection + identical bar PASS (audit SKILL.md + RUBRIC §10 + TC-01/02); #5 ambiguity needs_human PASS (TC-03); #6 command doc updated PASS (old wording gone - grep); #7 mixed repos per-file PASS (TC-04). Additive-only on all four modified skill files.

## coverage gate (steps 21-29)
Acceptance-driven task (contracts, no executable code): TC-01..05 case table + trigger P5/N5 pairs = the §5 verification set. 7/7 cyberos-install suites green post-change (payload rebuilds carry the edited command doc + skill files; chain-coverage + pair-parity + t04-additive all pass).
