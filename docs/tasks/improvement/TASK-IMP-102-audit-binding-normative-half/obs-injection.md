# TASK-IMP-102 observability injection

Contract prose plus a hash-preference branch: nothing long-lived to instrument, so inventing
spans would be theatre. The honest surface:

- **The report line IS the signal.** R1 now emits one of three self-describing notes: binding
  intact via the normative half, SPEC DRIFT with both hashes quoted, or the legacy path naming
  itself and the audit commit it reconstructed from. A reader never has to guess which rule fired.
- **Error branches**: none added - the preference is a pure selection between two verified paths.
- **Fixity is the monitor**: two audits of the same normative half now produce the same key, so
  drift in the report itself is detectable - which is what re_entrancy claimed all along and can
  now support.

Branch coverage: 4 of 4 binding paths (body-match, body-mismatch, legacy-honest,
legacy-dishonest) asserted in t06; 100 % of the added branch.
