        ---
        skill_id: observability-injection-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for observability-injection-audit

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this observability injection"
- "Check the observability injection for completeness"
- "Verify the observability injection meets the rubric"
- "Re-audit the observability injection"

        ## Negative triggers (MUST NOT route here)

        - "Draft a observability injection" → observability-injection-author
- "Create the observability injection" → observability-injection-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
