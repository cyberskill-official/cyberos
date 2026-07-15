        ---
        skill_id: emissions-inventory-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for emissions-inventory-author

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a emissions inventory"
- "Create the emissions inventory"
- "Author a new emissions inventory"
- "Generate the emissions inventory"

        ## Negative triggers (MUST NOT route here)

        - "Audit this emissions inventory" → emissions-inventory-audit
- "Check the emissions inventory for completeness" → emissions-inventory-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
