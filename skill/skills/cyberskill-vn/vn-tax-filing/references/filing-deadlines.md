# VAT filing deadlines — monthly vs quarterly

## Authority

- **Thông tư 80/2021/TT-BTC**, Article 8 — eligibility for monthly vs quarterly filing.
- **Luật Quản lý thuế 38/2019/QH14**, Article 44 — statutory deadline dates.
- **Luật Quản lý thuế 38/2019/QH14**, Article 86 — weekend / public-holiday roll-forward.

## Monthly filing (kê khai tháng)

| Field | Value |
|---|---|
| Form | Mẫu 01/GTGT |
| Period | One calendar month (`YYYY-MM`) |
| Statutory deadline | 20th day of the *following* month |
| Eligibility | Annual revenue ≥ 50 billion VND, or taxpayer opt-in |

**Example**: For the May 2026 period (`2026-05`), the statutory deadline is `2026-06-20`. June 20, 2026 is a Saturday, so the deadline rolls forward to Monday `2026-06-22`.

## Quarterly filing (kê khai quý)

| Field | Value |
|---|---|
| Form | Mẫu 01/GTGT (quarterly variant) |
| Period | One calendar quarter (`YYYY-QN`) |
| Statutory deadline | Last day of the *first month of the following quarter* |
| Eligibility | Annual revenue < 50 billion VND, household businesses, newly-established entities (first 12 months) |

Quarter-to-month mapping:

| Quarter | Months | Following month | Statutory deadline (last day of) |
|---|---|---|---|
| Q1 | Jan–Mar | April | April 30 |
| Q2 | Apr–Jun | July | July 31 |
| Q3 | Jul–Sep | October | October 31 |
| Q4 | Oct–Dec | January (next year) | January 31 (next year) |

**Example**: For the Q2 2026 period (`2026-Q2`), the statutory deadline is `2026-07-31`. July 31, 2026 is a Friday and not a public holiday — deadline stands.

**Example with rollover**: Q1 2025 (`2025-Q1`) → `2025-04-30` is a Wednesday but it's a public holiday (Reunification Day), and May 1 is Labour Day, May 2 a Friday observance, May 3 weekend… the deadline rolls forward to the next working day. The bundled `compute_deadline.py` carries a curated 2025–2027 holiday set.

## Frequency switching

A taxpayer chooses monthly or quarterly at the start of each calendar year, based on the *previous year's revenue*. Mid-year switches are not permitted (Article 9, TT 80/2021). Newly-established entities default to quarterly for the first 12 months, then re-elect based on actual revenue.

## Weekend / public holiday rule (Article 86)

> Nếu ngày cuối cùng của thời hạn… trùng vào ngày nghỉ cuối tuần hoặc ngày nghỉ lễ thì ngày cuối cùng của thời hạn được tính là ngày làm việc liền kề sau ngày nghỉ đó.

If the statutory deadline falls on a Saturday, Sunday, or a Vietnamese public holiday, it rolls forward to the next working day.

### Vietnamese public holidays (calendar years 2025–2027)

Bundled in `assets/holidays.json`. Notable dates:

- **Tết Dương Lịch** (New Year) — January 1
- **Tết Nguyên Đán** (Lunar New Year) — typically 5–7 days late January or mid-February
- **Giỗ Tổ Hùng Vương** (Hung Kings' Festival) — 10th day of the 3rd lunar month
- **Ngày Giải phóng / Quốc tế Lao động** — April 30 + May 1
- **Quốc khánh** (National Day) — September 2 (+1 day observance)

The exact dates shift year to year for the lunar holidays; the bundled JSON snapshots the official PM-announced calendar at the time of skill packaging. **Refresh annually** — the Ministry of Labour publishes the next year's calendar each October.

## Newly-established entities

A newly-established taxpayer files quarterly for the first 12 months regardless of revenue. After 12 months, eligibility is re-evaluated against the trailing-revenue threshold.

The first quarterly return covers the period from the business-licence issuance date through the end of the calendar quarter in which the entity was established. If the licence was issued in the final month of a quarter, the first return covers fewer than 30 days — this is permitted, and the standard deadline (last day of the first month of the *next* quarter) still applies.

## Other returns affecting VAT taxpayers

Beyond Mẫu 01/GTGT (covered here), VAT taxpayers may also need:

- **Mẫu 02/GTGT** — direct-method VAT (for taxpayers without sufficient invoice substantiation; not common for typical companies).
- **Mẫu 03/GTGT** — supplementary / corrective return.
- **Annual settlement** — not separately required for VAT (unlike CIT / PIT), but the year-end balance is reconciled with the closing period's `C2 / C5`.

This skill currently models only Mẫu 01/GTGT. Supplementary and direct-method returns are out of scope.
