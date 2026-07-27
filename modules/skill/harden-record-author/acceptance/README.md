# harden-record-author — acceptance

| Flow | Location | Notes |
|---|---|---|
| Planner regression | `../tools/harden-plan-check.sh` via author `tools/` | Includes shopass INS-F-0002 operator_prerequisites |
| Trigger tests | `TRIGGER_TESTS.md` | Must / must-not fire |
| Lint precondition | `tools/inspect-lint.mjs` | Must exit 0 before any finding is worked |

Do not treat package-root `harden-run/` patches as runtime fixtures.
