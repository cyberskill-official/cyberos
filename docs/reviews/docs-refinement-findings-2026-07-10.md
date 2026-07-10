# Docs refinement pass - findings for operator resolution (2026-07-10)

The FR-DOCS-002 dedicated refinement pass (30 pages, three parallel editors under a
strict no-invention rule) cleaned every migrated page to current-state prose. Real
content contradictions were deliberately LEFT AS WRITTEN and recorded here - each
needs an operator ruling, then a one-line doc fix.

## Cross-page contradictions

- ISO 27001 timing: compliance + milestones place "certified"/"Stage 1" at P2 exit in
  one spot and Stage 1 at P3 / certified at P4 in another. Pick one schedule.
- Module-count base flips between 22 and 23 across milestones and strategy.
- Apollo Router license: infrastructure says MIT/OSS; tech-stack says Elastic v1.2.
- tech-stack cost totals disagree: internal <= $535 vs <= $530 per month; 50-tenant
  ~$2,650 vs ~$2,750.
- tech-stack still lists Apache AGE in the data tier; AGE was dropped for the
  relational l2_edge - stale claim to update after confirmation.
- Overtime cap phrasing differs: time says 300 h/yr hard; the compliance table and
  res say 200 h, 300 h with consent/MoLISA.
- proj claims "221 workflows as of 2026-05-18" vs cuo module.md's 194 for the same date.
- mcp-gateway: header says "P0 slice 3", phase section says "slice 2"; "canonical six
  verbs" lists four.
- cuo module.md: "7 colliding acronyms" lists 8; §7.5 example path conflicts with the
  cro-revenue slug; "46 currently-shipped bundles" vs the 208-bundle total.
- ten pricing: per-seat ($49/$39) vs flat ($49/$249) for the same plans; Team AI
  tokens 5M vs 3M.

## Systematic migration placeholders (pending FR re-authoring)

- "(FR pending)" / "(NFR pending)" placeholders are pervasive in vision pages.
- "P0 -> P3/P4 horizon" phrases (esop, hr, rew, compliance Decree-53 trigger) are
  find-replace artifacts standing in for original durations - originals unrecoverable.
- Every vision page's "N KPIs" intro undercounts its KPI table (rows were appended
  after the intros were written).
- glossary + risk-register: the old pages rendered their data client-side from
  embedded script; the 199 glossary terms and RSK row details did not survive as
  markdown. Pages now describe the model and point at the interactive site pages;
  restoring the datasets is its own task.

## Numbering/reference nits kept verbatim

- cuo appendices body skips section 11; skill appendices had A-L (intro said A-H -
  corrected); several "traces back to ..." references had eaten targets (tidied,
  originals unrecoverable); AGENTS-CORE.md rename debris remains in skill appendices.
