# REF-NNN Regression eval

## What existing memories does this REF affect?
[describe — usually "should not affect any existing memory"]

## Test fixture
`regression.test.py` — pass criteria: all 134+ existing memories still validate after the REF lands.

## Pass / fail criteria
- PASS: cyberos_validate.py returns 0 CRITICAL on full memory walk
- FAIL: any pre-existing memory rejected by new rule
