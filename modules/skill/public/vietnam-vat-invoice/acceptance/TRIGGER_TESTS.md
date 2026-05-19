        ---
        skill_id: vietnam-vat-invoice
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for vietnam-vat-invoice

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Reference vietnam vat invoice"
- "Look up vietnam vat invoice"
- "Consult the vietnam vat invoice reference"

        ## Negative triggers (MUST NOT route here)

        - "Run unrelated task" → none
- "What time is it" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
