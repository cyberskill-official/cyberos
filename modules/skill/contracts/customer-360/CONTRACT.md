# customer-360@1 — Customer-360/CDP audit and architecture

A `customer-360@1` artefact audits the customer-360 (CDP — Customer Data Platform) state and proposes the next-quarter architecture refinement. Per CDP Institute reference architecture + Segment/RudderStack/mParticle CDP patterns + IAB GVL/TCF for consent + ISO/IEC 19944 cloud data classification.

## Required sections (template.md H2 order)
1. Inventory of customer-data sources (CRM / product / billing / support / marketing)
2. Identity resolution status (deterministic + probabilistic match-rate)
3. Master entity model (customer / account / opportunity / interaction)
4. Activation surfaces (downstream consumers: email, ads, in-product, sales)
5. Consent + governance posture (per GDPR / PDPD / CCPA)
6. Data-quality scorecards (completeness / freshness / accuracy per entity)
7. Gap analysis + next-quarter roadmap
8. Risks & open issues

## Citations
- CDP Institute reference architecture
- Segment / RudderStack / mParticle CDP patterns
- IAB GVL/TCF for consent
- ISO/IEC 19944 cloud data classification
- DAMA-DMBOK master-data-management chapter

## KPI
- Identity-resolution match rate %
- Master-entity data-quality score
- Activation latency (source → activation surface)
