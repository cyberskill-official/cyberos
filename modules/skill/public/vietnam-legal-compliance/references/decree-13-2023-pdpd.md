# Nghị định 13/2023/NĐ-CP — Personal Data Protection Decree (PDPD)

**In force:** 1 July 2023. **Issuing authority:** Government of Vietnam (proposed by Bộ Công An). **Supervisory authority:** Ministry of Public Security (MoPS) — specifically the Department of Cybersecurity and High-Tech Crime Prevention (A05).

This is Vietnam's principal data-protection regulation. It applies to any organisation that processes the personal data of natural persons located in Vietnam, regardless of the operator's own location — i.e. it has extraterritorial reach similar in spirit to GDPR Article 3.

## Scope (Articles 1–2)

- **Personal data ("dữ liệu cá nhân")** — any information identifying or capable of identifying a natural person, including indirect identifiers when combined.
- **Sensitive personal data ("dữ liệu cá nhân nhạy cảm")** — explicitly enumerated: race/ethnicity, political opinion, religion, health, genetic data, biometric identifiers, financial data, location, online activity, communication records, sexual life, criminal records.
- **Processing** — any operation performed on personal data, electronic or otherwise.

## Lawful bases (Article 8)

A controller MAY process personal data only with at least one of:

1. **Consent** of the data subject — the default and most-litigated basis.
2. **Contract performance** to which the subject is a party.
3. **Legal obligation** of the controller.
4. **Vital interest** of the subject or another natural person.
5. **Public interest** or exercise of official authority.
6. **Legitimate interest** of the controller, only where it does not override the subject's rights.

The PDPD is markedly more restrictive than GDPR in practice because the consent bar is high (see below) and "legitimate interest" is interpreted narrowly by MoPS guidance.

## Consent (Article 11)

Consent MUST be:

- **Written** — express, recorded, retrievable. Click-to-accept is acceptable if the act is logged with a timestamp and the data subject can later retrieve evidence.
- **Specific** — purpose-bound; bundled consent for multiple unrelated purposes is invalid.
- **Unambiguous** — pre-ticked checkboxes are explicitly invalid.
- **Freely given** — consent obtained under duress or as a precondition for a service that does not require the data is invalid.
- **Revocable** — the revocation mechanism must be as easy as the granting mechanism. Revocation does not unwind past lawful processing.

Sensitive data requires **separate, explicit consent** beyond any general processing consent.

## Data subject rights (Articles 14–16)

- **Right of access** — within 72 hours of a verified request.
- **Right to rectification** — controller must correct or annotate within 72 hours.
- **Right to erasure** — except where retention is mandated by law (e.g. tax records).
- **Right to restriction** of processing pending a dispute.
- **Right to portability** — machine-readable export.
- **Right to object** to automated decision-making with significant effect.
- **Right to lodge a complaint** with MoPS.

## Data Protection Officer (Article 28)

A **DPO is mandatory** for:

- Public authorities and state-owned enterprises.
- Operators whose core activities involve **systematic monitoring at scale** (e.g. behavioural advertising, fleet tracking).
- Operators that process **sensitive data at scale** as a core activity.

The DPO MUST have legal-data-protection competence, report directly to the highest management level, and be reachable by data subjects. The DPO contact MUST be published in the operator's privacy notice and filed with MoPS upon designation.

## Cross-Border Data Transfer Impact Assessment (CBDTIA) — Articles 24–25

Any transfer of personal data **outside Vietnam** (including transfer to a foreign-located cloud region operated by a Vietnamese entity) requires:

1. A written **Transfer Impact Assessment** (TIA) per a standard template prescribed by MoPS.
2. **Filing the TIA with MoPS** within 60 days of starting the transfer (Article 25).
3. Annual update of the TIA filing.
4. The assessment must cover: data categories, purposes, recipients, recipient-jurisdiction legal regime, security measures, retention, and the impact on data subjects.

The TIA is **filed, not approved** — MoPS does not pre-clear transfers. However, MoPS may inspect the TIA at any time and order suspension if the assessment is found inadequate.

## Breach notification (Article 23)

- **To MoPS:** within **72 hours** of becoming aware of a breach with potential impact on data subject rights. The notification must include scope, categories, number of affected subjects, likely consequences, mitigations, and a contact.
- **To affected data subjects:** within **48 hours** if the breach poses high risk to the subject's rights (e.g. financial data, biometric data, large-scale leakage). Notice must be in clear non-technical language with concrete mitigation steps the subject can take.
- A breach register MUST be maintained even for non-notifiable incidents.

## Penalties (Article 32 + Decree 15/2020 cross-refs)

- Up to **5% of preceding-year revenue** for repeat or egregious violations.
- Up to **100 million VND** administrative fines for routine violations.
- Personal liability for senior officers in certain cases.
- Possible **suspension of operations** in Vietnam.

## Exemptions (Article 6)

The PDPD does not apply to:

- Purely personal/household activities by the data subject themselves.
- Processing strictly for national defence, state security, or crime prevention (these are governed by the Cybersecurity Law / Decree 53 instead).
- Strictly anonymised data — but the bar for "anonymised" is high; pseudonymisation is NOT anonymisation under MoPS guidance.

## Practical compliance checklist

The following are operator-side actions that are immediate and high-leverage:

1. **Inventory** all personal-data processing activities (categories, purposes, recipients, retention).
2. **Map lawful basis** for each — most ops will land on Consent or Contract.
3. **Audit consent flows** — pre-ticked? Bundled? Revocable? Logged?
4. **Designate DPO** if any of the mandatory triggers fire.
5. **Draft + file TIAs** for all cross-border flows (including AWS-Tokyo, GCP-Singapore, etc.).
6. **Wire the 72h / 48h breach-notification clock** into your incident-response runbook.
7. **Update privacy notice** to enumerate data subject rights and DPO contact.
8. **Train staff** — consent collection, breach detection, data-subject-rights handling.

## Article cross-reference index

| Topic | Article(s) |
|---|---|
| Scope + definitions | 1, 2, 3 |
| Sensitive data list | 2.4 |
| Lawful bases | 8 |
| Consent | 11, 12, 13 |
| Subject rights | 14–16 |
| Controller obligations | 17–22 |
| Breach notification | 23 |
| Cross-border transfer + TIA | 24, 25 |
| DPO | 28 |
| MoPS supervision powers | 30, 31 |
| Penalties | 32 |

## Disclaimer

This summary is informational; consult Vietnamese legal counsel before production filings. CyberSkill provides this skill as procedural knowledge for AI agents; not as legal advice. The text of the decree is published in the Official Gazette (Công báo) and on the MoPS portal at `mps.gov.vn`.
