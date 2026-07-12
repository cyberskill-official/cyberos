# architectural-spike-audit - report format

Header: spike_id | fr_id | verdict (pass|fail|needs_human) | score N/10 | iteration k/3.

Findings table (fail/needs_human only):
| rule | location | finding | resolves by |
|---|---|---|---|
| SPK-EVID-002 | options[1].evidence | "X is faster" carries no citation | add command+output or drop the claim |

Closing line, verbatim shape: **Score = N/10.** A pass report ends with the single
line `spike <spike_id> PASS 10/10` so ship-chain greps stay stable.
