        ---
        skill_id: delivery-review-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for delivery-review-author

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

- "Draft a delivery review"
- "Create the delivery review"
- "Author a new delivery review"
- "Generate the delivery review"

        ## Negative triggers (MUST NOT route here)

- "Audit this delivery review" → delivery-review-audit
- "Check the delivery review for completeness" → delivery-review-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

- Triggers derived from skill name + role (author/audit) via the heuristic backfill script. They are conservative — refine with OBS-observed real user phrasings during the next natural fine-tune cycle.
- Re-author when classifier_version MAJOR-bumps.
