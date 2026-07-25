# TASK-IMP-146 implementation evidence

Implemented on `batch/11-wave2-residuals`.

- Added `--dry-run` to `memory-append.mjs append`.
- Factored lease inspection from lease minting, so dry-run and real appends share held,
  expired, orphan and boot-epoch-skew semantics without the rehearsal writing `.lock`.
- Shared the record/hash constructor between projection and append.
- Dry-run reports projected seq/hash/path and bootstrap or one-behind-HEAD actions while
  skipping every store write: root creation, lock, scaffold, tmp sweep, segment, HEAD and
  MMR peaks.
- Added t06–t09 for byte immutability, exact projection under a pinned clock, refusal-code
  parity, nonexistent-store bootstrap reporting, one-behind-HEAD reporting and docs.

Targeted evidence:

```text
$ bash tools/install/tests/test_memory_append.sh
  ok   t01 ... t09
test_memory_append: pass=9 fail=0

$ bash tools/install/tests/test_hitl_lock.sh
pass=7 fail=0
```

