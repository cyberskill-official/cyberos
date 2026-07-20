# architectural-spike-audit - audit loop

1. Read the artefact once; refuse unknown versions (needs_human).
2. Structural pass (SPK-STRUCT) - cheap failures first.
3. Evidence resolution pass (SPK-EVID) - actually resolve each citation: stat the file path, re-read the recorded command output, or fetch-check the URL shape.
4. Box arithmetic (SPK-BOX) and discard honesty (SPK-DISC).
5. Verdict: 10/10 -> pass. Otherwise emit findings (rule id + location + fix) back to the author; max 3 author-audit iterations, then needs_human with the open findings. The audit never edits the artefact; it only scores and reports.
