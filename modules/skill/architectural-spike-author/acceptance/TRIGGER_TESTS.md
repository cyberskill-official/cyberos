        ---
        skill_id: architectural-spike-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for architectural-spike-author

        > Pair verification (TASK-SKILL-117 §5, executable line by line):
        >   grep -q "SPIKE-<FR-ID>-<n>" ../SKILL.md                  # artefact id grammar declared
        >   grep -q "1.5x" ../SKILL.md && grep -q "1.5x" ../PIPELINE.md   # timebox HALT normative (AC 3)
        >   grep -q "checkable" ../SKILL.md                          # evidence rule present (AC 4)
        >   test -f ../PIPELINE.md -a -f ../INVARIANTS.md            # layout parity (AC 7)
        >   test -f ../envelopes/input.json -a -f ../envelopes/output.json
        >   test -f ../references/FAILURE_MODES.md

        ## Positive triggers (MUST route here)

        - "Draft an architectural spike"
- "Create the architectural spike"
- "Run a spike on MMR vs plain hash chain before the ADR"
- "Time-box an investigation of the two storage options"

        ## Negative triggers (MUST NOT route here)

        - "Audit this architectural spike" → architectural-spike-audit
- "Check the spike for evidence gaps" → architectural-spike-audit
- "Draft an ADR" → architecture-decision-record-author
- "Deep-map the repo before implementation" → repo-context-map-author
- "What is our company holiday schedule" → none
