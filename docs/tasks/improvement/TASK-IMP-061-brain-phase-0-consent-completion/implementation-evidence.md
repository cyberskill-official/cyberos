# TASK-IMP-061 implementation evidence

- AGENTS.md §19 Phase 0 consent added (Layer-1 only; EVAL activation out of scope).
- `CONSENT.md` template + starter `meta/consent/README.md`.
- `install.sh` scaffolds `meta/consent/` on fresh BRAIN.
- Invariant `personnel-requires-consent` + `check_personnel_requires_consent`.
- Fixture 21 flipped to expect that code.
- `build.sh` materialises `modules/memory/cyberos/data/AGENTS.md` into payload memory/.
- Tests: 8/8 pass (`test_personnel_consent.py`).
