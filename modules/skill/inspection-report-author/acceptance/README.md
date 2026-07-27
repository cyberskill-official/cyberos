# inspection-report-author — acceptance

Author goldens live with the audit pair (`../inspection-report-audit/acceptance/*.golden.md`) because the machine floor and rubric both consume the same report shape. This directory holds trigger tests and any author-only fixtures added later.

| Flow | Location | Notes |
|---|---|---|
| Spec 1.0 goldens | `../inspection-report-audit/acceptance/*.golden.md` (non-`.r2`) | Version-gated; stay valid |
| Spec 1.2 goldens | `../inspection-report-audit/acceptance/*.r2.golden.md` | 75-discipline ledger |
| Trigger tests | `TRIGGER_TESTS.md` | Must / must-not fire |
