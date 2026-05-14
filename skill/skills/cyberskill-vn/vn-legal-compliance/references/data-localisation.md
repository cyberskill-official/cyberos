# Data localisation in Vietnam — practical reference

Vietnam has two overlapping localisation regimes, governed by different decrees and supervised by different ministries. Operators should map their footprint against both.

## Regime 1 — Decree 53/2022 (Cybersecurity)

Triggered when **MoPS issues a localisation order** to a foreign service provider with Vietnamese users.

### Data categories that must reside in Vietnam

1. **Personal data of users in Vietnam** — name, contact info, ID numbers, payment methods, etc.
2. **User-generated data** — account history, social-graph relationships, content created or uploaded by the user, message metadata.
3. **Service usage data** — login records, IP/device fingerprints, location data when collected, time-of-use logs.
4. **Any other categories specified in the order** — MoPS retains discretion to extend the list per provider.

### Retention period

At least the period of service operation in Vietnam plus **24 months** after operations cease.

### Local-presence requirement

If localisation is ordered, the foreign provider MUST also establish a **branch or representative office** in Vietnam. The office is the recipient of legal process and cooperation requests.

### What is NOT required

- Continuous real-time mirroring to Vietnam (a snapshot model is acceptable per MoPS guidance).
- Storing technical-infrastructure data (load-balancer logs, internal-service metrics that don't reference user-level data).
- Storing data of non-Vietnamese users.

## Regime 2 — Decree 13/2023 (PDPD) — Cross-Border Transfer Impact Assessment

Triggered when **any personal data flows outside Vietnam**, including:

- Data sent to a foreign cloud region (AWS us-east, GCP europe-west, Azure ap-southeast-1 in Singapore, etc.).
- Data shared with foreign-domiciled processors (analytics SaaS, error tracking, payment processors).
- Data accessed remotely by foreign-located staff (engineering teams logging in from outside VN).

### What's required

A **Transfer Impact Assessment (TIA)** must be:

1. Prepared in advance of the transfer per the MoPS template.
2. Filed with MoPS within 60 days of starting the transfer.
3. Updated annually and re-filed.
4. Made available for MoPS inspection.

### What's NOT required

- **Pre-approval.** Filing is sufficient — MoPS does not pre-clear transfers.
- Storing the data in Vietnam in addition to the foreign location.
- Stopping the transfer for any standing reason — only if MoPS specifically suspends it.

### Common transfer scenarios

| Scenario | TIA required? | Notes |
|---|---|---|
| App stores personal data in AWS Singapore | Yes | Cross-border transfer. |
| App stores personal data in VN-hosted DB; remote engineer in US queries it for debugging | Yes | "Remote access" is a transfer per MoPS guidance. |
| App uses Stripe for payment processing (US-headquartered) | Yes | Card data goes to Stripe US. |
| App uses Sentry / Datadog | Yes | Logs may contain personal data. |
| App uses Google Workspace email | Yes if user-side mailboxes contain personal data of VN users. |
| App stores only company employee data in VN | No | No cross-border flow. |
| Strictly anonymised analytics export to a foreign warehouse | Technically no, but bar for anonymisation is high — pseudonymisation does not qualify. |

## Practical posture

For most SaaS operators with Vietnamese users:

1. **Default to filing TIAs.** They are cheap to produce relative to the cost of being out of compliance.
2. **Do not pre-localise** under Decree 53 — wait for a formal order. Localising preemptively does not reduce the TIA filing burden under Decree 13.
3. **Architect for optional localisation.** If you operate in payments, OTT messaging, or social — categories that have historically attracted MoPS attention — keep a Vietnam-region database deployment template ready to spin up.
4. **Document everything.** MoPS supervision is reactive — they inspect when triggered. The strength of your documentation is the difference between a routine review and a suspension order.

## Bank-specific localisation

The State Bank of Vietnam (SBV) imposes its own data-localisation requirements on credit institutions via Circular 09/2020/TT-NHNN and successors. Banking-core systems, payment switches, and core ledgers must be hosted in-country. This is a separate regime from Decree 53 and applies even where MoPS has not issued an order. Banking operators should consult SBV regulations directly.

## Cloud provider posture

- **AWS, Azure, GCP, OVH** — none have a VN-region as of 2026. Operators wanting on-soil hosting use VN domestic providers (Viettel IDC, VNPT, FPT, CMC) or co-location facilities.
- **Vendor SLAs** — VN-domestic providers typically offer 99.9% uptime against the hyperscaler 99.99%. Disaster-recovery planning matters more.
- **Migration cost** — production migration from a hyperscaler region to a VN-domestic provider commonly runs 6–12 months for a multi-tenant SaaS of meaningful scale. Factor it into your contingency planning if you operate in a category MoPS targets.

## Disclaimer

This summary is informational. Cross-border data flow regulation is one of the most rapidly evolving areas of Vietnamese law; consult counsel before relying on this guidance. CyberSkill maintains the skill against the regulatory state at the version pin shown in the frontmatter.
