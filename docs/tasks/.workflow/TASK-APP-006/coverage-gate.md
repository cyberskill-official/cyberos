# TASK-APP-006 — coverage gate (testing phase, 2026-07-13)

- T17: cask ruby -c. PASS
- T18: parses ×4 (3 winget files + workflow). PASS
- T19: zero submission commands repo-wide (check a; check b clean earlier same session). PASS
- T20/T21: render dry-run both ecosystems (fake artifact → version/sha substituted, placeholders gone, outputs stay valid). PASS
- Floor: run-gates.sh GREEN. Coverage N/A (declared).

TRACE closure: §1 #1✅ #2✅(always-on desktop-job artifacts only) #3✅(T17 fields) #4✅(T18, hedges in-file) #5✅(nothing submits) #6✅(T19 standing guard) #7✅(T20/T21 re-derivation) #8✅(answer sheet) #9✅(zap real-test requirement recorded). Deferred: AC #1 brew audit / AC #2 winget validate (tools absent here — pre-PR requirements vs RENDERED drafts); AC #8 NSIS switch real test (human).

**Machine gates green → HALTED at HITL gate 2. Final acceptance verdict is yours.**
