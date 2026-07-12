        ---
        skill_id: architectural-spike-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for architectural-spike-audit

        > Pair verification (FR-SKILL-117 §5, executable line by line):
        >   grep -q "SPK-STRUCT" ../RUBRIC.md && grep -q "SPK-EVID" ../RUBRIC.md   # families (AC 5)
        >   grep -q "SPK-BOX" ../RUBRIC.md && grep -q "SPK-DISC" ../RUBRIC.md
        >   grep -q "10/10" ../RUBRIC.md                                            # pass bar (AC 5)
        >   test -f ../AUDIT_LOOP.md -a -f ../REPORT_FORMAT.md                      # layout parity (AC 7)
        >
        > Fixture case table (AC 4 - evidence rule enforceable):
        >   CASE-01 clean spike (2 options, cited evidence, box respected)  -> pass 10/10
        >   CASE-02 option carrying only "X is faster" (no citation)        -> fail SPK-EVID-002
        >   CASE-03 actual 10h vs timebox 6h, halted absent                 -> fail SPK-BOX-003
        >   CASE-04 recommendation names an unprobed option                 -> fail SPK-STRUCT-003
        >   CASE-05 confidence high with 1 evidence entry per option        -> fail SPK-EVID-004
        >   CASE-06 rejected option missing from discard log                -> fail SPK-DISC-001

        ## Positive triggers (MUST route here)

        - "Audit this architectural spike"
- "Check the architectural spike"
- "Score the spike against the rubric"
- "Does this spike pass?"

        ## Negative triggers (MUST NOT route here)

        - "Draft an architectural spike" → architectural-spike-author
- "Run a spike on X vs Y" → architectural-spike-author
- "Audit this ADR" → architecture-decision-record-audit
- "What is our company holiday schedule" → none
