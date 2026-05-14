---
name: vn-tax-filing
description: >-
  Generate Vietnamese VAT return XML (Mẫu 01/GTGT for monthly and quarterly
  filings per Thông tư 80/2021/TT-BTC), compute filing deadlines, estimate
  late-filing penalties per Decree 125/2020. Use when the user needs to
  file a Vietnamese monthly or quarterly VAT return, asks "when is the
  next VAT filing due", or has missed a filing and wants to know the
  penalty. Do NOT use for invoice generation — use vn-vat-invoice for that.
license: Apache-2.0
compatibility: >-
  Fully offline. Python 3.11+ for the bundled scripts. Output XML follows
  TT 80/2021 schema sketch — final submission must validate against
  the GDT-published .xsd in force at filing time. Depends on vn-mst-validate.
metadata:
  author: cyberskill
  version: "0.1.0"
  region: VN
  collection: cyberskill-vn
  depends_on: vn-mst-validate,vn-vat-invoice
allowed-tools: read_file write_file
---

# Vietnamese VAT Return (Mẫu 01/GTGT — Tờ khai thuế GTGT)

## When to use

- User has output VAT + input VAT figures for a period and needs the XML return for monthly or quarterly filing.
- User asks "when is the next VAT filing due", "hạn nộp tờ khai thuế GTGT", or "deadline for May 2026 VAT".
- User missed a filing and wants to know the late-filing penalty band.
- Migration from spreadsheet-based VAT calculations to GDT-submittable XML.

Do NOT use this skill for issuing customer-facing invoices — that is `vn-vat-invoice`. This skill aggregates already-issued invoices into the periodic return form.

## Procedure

1. **Validate MST.** Taxpayer MST must structurally pass `vn-mst-validate` (10 digits, or 10-digit entity + `-NNN` branch).
2. **Categorize output VAT by rate.** Group every taxable sale in the period by VAT rate: `0% / 5% / 8% / 10%` (Mẫu 01/GTGT rows A1–A4). The 8% rate is the temporary reduction valid through 2026 — see `references/tt80-2021-schema.md`.
3. **Compute output VAT per rate.** For each rate, `thue = taxable_value * rate`, rounded half-up to integer VND. Document totals are the sum of per-rate totals.
4. **Compute input VAT deduction.** `B1` = total purchases, `B2` = total deductible input VAT (from supplier VAT invoices), `B3` = allowable deduction this period after any prior-period carry-over.
5. **Compute net payable / refundable.** `C1 = sum(A.thue) - B3`. `C4 = max(0, C1 + C2 - C3)` payable; `C5` refund request if `C1 + C2 < C3`. Exactly one of `C4`/`C5` is non-zero.
6. **Emit the XML** per the TT80/2021 sketch via `scripts/generate_return.py`. Validate with `scripts/validate_return.py` before forwarding to a TVAN.

## Quick start

```bash
cat > /tmp/return.json <<'EOF'
{
  "period": "2026-05",
  "filing_frequency": "monthly",
  "taxpayer": {
    "name": "Công ty TNHH ABC",
    "mst": "0312345678",
    "address": "123 Lê Lợi, P. Bến Nghé, Q.1, TP.HCM"
  },
  "filing_date": "2026-06-15",
  "output_vat": [
    {"rate": 10, "taxable": 90000000}
  ],
  "input_vat": {"total_purchases": 50000000, "deductible": 5000000},
  "previous_carry_over": 0,
  "paid_this_period": 0
}
EOF

python scripts/generate_return.py < /tmp/return.json > /tmp/return.xml
python scripts/validate_return.py < /tmp/return.xml
# → {"ok": true, ...}

echo '{"period": "2026-05", "filing_frequency": "monthly"}' \
  | python scripts/compute_deadline.py
# → {"deadline": "2026-06-22", "days_remaining": ..., "is_overdue": ...}

echo '{"period": "2026-05", "filing_date": "2026-07-15"}' \
  | python scripts/compute_penalty.py
# → {"days_late": 23, "penalty_band": "2-5M", "recommended_action": ...}
```

## Filing deadlines

| Filing frequency | Eligibility | Deadline |
|---|---|---|
| **Monthly** (Mẫu 01/GTGT) | Annual revenue ≥ 50 billion VND, or opted in | 20th day of the following month |
| **Quarterly** (Mẫu 01/GTGT, quarterly) | Annual revenue < 50 billion VND, household businesses, newly-established (first 12 months) | Last day of the first month of the following quarter |

If the statutory deadline falls on a weekend or Vietnamese public holiday, it rolls forward to the next working day (Article 86, Law on Tax Administration 38/2019/QH14). The bundled `compute_deadline.py` accounts for this with a curated 2025–2027 holiday calendar.

Quarter mapping: `Q1 = Jan–Mar`, `Q2 = Apr–Jun`, `Q3 = Jul–Sep`, `Q4 = Oct–Dec`. So `2026-Q2` is due `2026-07-31` (rolled forward if needed).

## Late filing penalty quick reference

Per Nghị định 125/2020/NĐ-CP, Article 13:

| Days late | Penalty band | Recommended action |
|---|---|---|
| 1–5 days | Warning (cảnh cáo) | File immediately. Document mitigating circumstances. |
| 6–30 days | 2–5 million VND | File now. Consider voluntary disclosure to reduce. |
| 31–60 days | 5–8 million VND | File now. Engage tax advisor for response strategy. |
| 61–90 days | 8–15 million VND | File now. Prepare formal explanation. |
| 90+ days | 15–25 million VND | File now. Criminal-evasion investigation possible if intent is shown. Engage counsel. |

In addition to the administrative penalty above, late-paid tax accrues a daily 0.03% interest charge (Article 59, Law on Tax Administration 38/2019/QH14). The penalty bands compound with the interest; both are owed.

See `references/penalty-table.md` for citations + edge cases (corrections, force majeure, voluntary disclosure).

## Structure

- `scripts/generate_return.py` — JSON → Mẫu 01/GTGT XML (stdin → stdout, or `--out <path>`).
- `scripts/validate_return.py` — XML structural validator + arithmetic check.
- `scripts/compute_deadline.py` — Given `{period, filing_frequency}` → deadline + `days_remaining` + `is_overdue`.
- `scripts/compute_penalty.py` — Given `{period, filing_date}` → `days_late` + `penalty_band` + recommended action.
- `references/tt80-2021-schema.md` — Element-by-element form structure.
- `references/filing-deadlines.md` — Monthly vs quarterly rules, edge cases.
- `references/penalty-table.md` — Late filing penalties per Decree 125/2020.
- `references/examples-monthly.xml` — Worked example.
- `assets/template-monthly.xml` + `assets/template-quarterly.xml` — Human-readable Mẫu 01/GTGT skeletons.
- `assets/holidays.json` — Vietnamese public holidays 2025–2027 used by `compute_deadline.py`.

## Production caveat

The bundled XML schema is an **approximation of the TT 80/2021 Mẫu 01/GTGT form**, suitable for development, prototyping, and integration with downstream systems that accept the rough shape. **Before submitting to a GDT-authorised TVAN (transmission service provider), validate against the official `.xsd` from `gdt.gov.vn` in force at filing time** — element names, namespace URIs, and required/optional cardinalities are version-pinned to the current GDT release. This skill is structurally correct but not production-certified.

The penalty computations are estimates only — the final amount is set by the issuing tax authority and may include additional factors (recidivism, magnitude of underpayment, etc.) not modelled here.
