---
title: "EMAIL — Vietnamese-aware composition (Anh/Chị/Bạn salutations, sign-offs, diacritics, vi-VN locale, business honorifics)"
author: "@stephen-cheng"
department: product
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "P1 / 2026-Q4"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Make EMAIL Vietnamese-first: correct **honorific selection** (Anh / Chị / Bạn / Em / Quý anh chị / Quý khách) with per-recipient memory; correct **business salutations and sign-offs** (`Chào anh [tên]`, `Trân trọng`, `Thân ái`); **diacritic-aware HTML rendering and search** so `Trịnh Thái Anh` always renders correctly across mail clients and `trinh thai anh` (no diacritics) still finds it; **vi-VN as the composition default** for VN-recipients with one-click switch to English; **bilingual mail templates** (`Chào anh / Hi`); **PGroonga-tokenised search** consistent with the rest of CyberOS; **default sign-off** wired to the founder's bilingual block from the global instructions; and **Vietnamese spelling assist** that runs on draft text without sending it to an external provider.

## Problem

Vietnamese business email is materially different from English business email in three ways a generic mail client cannot get right:

- **Honorifics are mandatory.** Addressing a customer as "Hi" instead of "Chào anh [tên]" is a reputational risk in Vietnamese B2B; addressing them with the wrong honorific (Bạn for someone older or higher-status; Em for someone you are not familiar with) is worse than no honorific at all.
- **Diacritics are content, not decoration.** "Trịnh" and "Trinh" are different names. A mail client that strips or auto-corrects diacritics produces a wrong recipient name in the From header. Search must accept both diacritic and non-diacritic forms because Vietnamese keyboards are not always available.
- **Mixed-language threads are normal.** A thread between the founder and a Vietnamese partner naturally mixes vi-VN and en-US; the editor must support both inline without auto-correct misfiring on either.

This FR makes EMAIL viable for the Vietnamese internal team and for any future Vietnamese-tenant customers (the wedge per Bet 7).

## Proposed Solution

The shape of the answer is a Vietnamese composition layer in the EMAIL UX (FR-EMAIL-002), per-recipient honorific memory, diacritic-aware indexes, vi-VN-aware draft assist, and the founder's default sign-off block.

**Per-recipient honorific memory.**

`email.recipient_profile{tenant_id, recipient_email, honorific, register, language_default, last_used_at, source_thread_id}` — populated by:

1. Manual selection in the composer (the user picks once; the system remembers).
2. Auto-detection from inbound emails: the sender's signature ("Trân trọng, Anh Khoa") yields `honorific: anh`; the sender's prior salutation to the team ("Chào anh Stephen") yields the recipient's preferred register.
3. CRM linkage (FR-EMAIL-006 + batch-05) — when a recipient is linked to a CRM contact with `vi_honorific` set, that overrides the auto-detection.

The composer renders the suggested salutation as a chip ("Chào anh Khoa") that's clickable to insert + dropdown to switch.

**Honorific options.**

- `Anh` — male, peer or older.
- `Chị` — female, peer or older.
- `Em` — younger, more familiar; rarely used for new business contacts.
- `Bạn` — neutral, peer; informal.
- `Quý anh chị` — formal plural ("dear sirs and madams").
- `Quý khách` — formal customer-facing ("dear customer").
- `Anh/Chị` — when gender is uncertain and a single honorific is needed.

The composer also exposes a "register" axis (formal / business / informal) that adjusts the salutation + sign-off pairings.

**Default salutations and sign-offs (vi-VN).**

| Register | Salutation pattern | Sign-off |
|---|---|---|
| Formal | `Kính gửi Quý anh chị,` | `Trân trọng,` |
| Business | `Chào anh [tên],` / `Chào chị [tên],` | `Trân trọng,` / `Cảm ơn anh/chị,` |
| Informal | `Chào [tên],` | `Thân,` / `Thân ái,` |

The founder's default English sign-off:
```
Best,
Stephen Cheng (Trịnh Thái Anh)
Founder & CEO · CyberSkill
"Turn Your Will Into Real."
info@cyberskill.world  ·  +84 906 878 091
```

The founder's default Vietnamese sign-off:
```
Trân trọng,
Trịnh Thái Anh (Stephen Cheng)
Nhà sáng lập & CEO · CyberSkill
"Turn Your Will Into Real."
info@cyberskill.world  ·  +84 906 878 091
```

Sign-off blocks are stored in `email.member_signature{member_id, locale, body_html, body_md, is_default}` and are auto-inserted on compose; the user can swap with a single key (`Cmd-Shift-L` toggles vi/en signature).

**Diacritic-aware rendering.**

- The Stalwart MIME pipeline preserves UTF-8 bodies + headers natively.
- The webmail renders with `font-feature-settings: "liga", "calt"` and the Be Vietnam Pro typeface (FR-DESIGN-001) for stacked-diacritic correctness.
- Search uses PGroonga's `Mecab` tokeniser with the Vietnamese dictionary configuration (the same one BRAIN Layer 2 + CHAT FTS use) and **diacritic-folding query mode**: typing `trinh thai anh` matches `Trịnh Thái Anh`. The original message text remains diacriticised; only the search query is folded.
- HTML email composition is RFC 5321/5322-compliant + UTF-8 + flowed-content; subject lines are RFC 2047 MIME-encoded for legacy-MTA compatibility.

**Mixed-language threads.**

- The composer detects the input language paragraph-by-paragraph (lightweight `cld3` library, self-hosted).
- Spelling assist (next bullet) is per-paragraph: vi-VN paragraphs get vi-VN suggestions, en-US paragraphs get en-US.
- The CUO/COO suggested-reply path (FR-EMAIL-004) follows the language of the most recent inbound message unless the user overrides.

**Vietnamese spelling assist.**

- A locally-running Hunspell dictionary with the Vietnamese word list (community-maintained `vi_VN.aff` + `vi_VN.dic`).
- Suggestions surface as red-underline + right-click correction (standard browser pattern); never round-trips to an external provider; never logs the draft text outside the user's browser.
- For tone-mark errors common when typing without Telex/VNI ("anh" should be "ánh" in context), the Vietnamese-tone helper offers contextual suggestions; this uses a small self-hosted model (the same `bge-m3`-derived classifier line we already run for BRAIN; FR-AI-001 §"Self-hosted models").

**Bilingual templates.**

- A small library of templates (greeting + sign-off pairings, follow-up prompts, "thanks for the meeting" variants, "we'll get back to you" variants) is shipped as `email.template{id, tenant_id, name, locale, subject_template, body_template, member_id?}`.
- The composer "Templates" menu lists them; selecting one inserts into the composer with `{{recipient.first_name}}` and `{{recipient.honorific}}` placeholders auto-filled.
- The Founder + HR/Ops Lead can author tenant-shared templates; Members can author personal templates.

**Locale at the message level.**

- `email.message_index.locale` is populated by `cld3` on receipt + on send.
- The mailbox view shows a small "vi" / "en" / "mixed" chip on each thread row.
- Per-Member preference (FR-EMAIL-004) `locale_default` selects the composition default; the user can switch per-thread.

**Search UX.**

- The search bar accepts both forms; queries are diacritic-folded and tokenised consistently.
- Query language: `from:nguyen subject:hợp đồng` works; `from:nguyen subject:hop dong` also works (folding); explicit case-sensitive search via `"...":exact` syntax.
- Saved searches per Member (`email.saved_search`).

**MCP tool surface (extends FR-EMAIL-001/002/004).**

- `cyberos.email.compose_in_locale(to, locale, register, body)` — `destructive: false`; produces a draft with appropriate salutation/sign-off.
- `cyberos.email.list_templates(locale?, scope?)` — read.
- `cyberos.email.suggest_honorific(recipient_email)` — read; returns the suggested honorific + confidence + sources.

## Alternatives Considered

- **Default to en-US always; let the Member translate.** Rejected: the Vietnamese-first wedge collapses; this is the moat for the local-market expansion at P4.
- **Pull honorifics from a third-party provider's API.** Rejected: residency + cost; the per-recipient memory + auto-detection from prior threads is sufficient.
- **Auto-correct diacritics on outgoing messages.** Rejected: dangerous; a name like `Trinh` vs `Trịnh` is the user's choice.
- **Skip Vietnamese spelling assist; rely on browser default.** Rejected: browser defaults vary by OS + browser; the behaviour is unpredictable in production.
- **Use a fine-tuned model for honorific selection.** Rejected for P1: rules + memory + auto-detection are sufficient; fine-tune is a P3 initiative if precision plateaus.

## Success Metrics

- **Primary metric.** P1 → P2 gate progress: ≥ 95% of Vietnamese outbound emails from the team carry the correct honorific (sampled audit against known recipients) over a 14-day window.
- **Adoption metric.** ≥ 80% of vi-VN composition uses one of the bilingual templates by P1 → P2 exit.
- **Search precision.** Diacritic-folded search returns expected hits with ≥ 95% recall on a curated 50-query test set.
- **Performance.** Composer language detection latency ≤ 80 ms on a 500-character paragraph.

## Scope

**In-scope.**
- Per-recipient `email.recipient_profile` honorific memory + auto-detection.
- Composer salutation + sign-off picker with the seven honorifics + three registers.
- Founder + Member signature blocks (vi + en) with default selection logic.
- Diacritic-aware Be Vietnam Pro rendering across the EMAIL UX.
- PGroonga search with diacritic folding + advanced query syntax.
- `cld3` per-paragraph language detection.
- Hunspell + tone-helper Vietnamese spelling assist (in-browser).
- Bilingual template library with `{{recipient.*}}` placeholders.
- Per-Member `locale_default` preference.
- The three new MCP tools.

**Out-of-scope (deferred).**
- Auto-translation between vi-VN and en-US for displayed messages (P2; the CHAT translation chip is the prior art).
- Honorific selection from a third-party national-database (out of scope forever for residency reasons).
- Khmer / Thai / Lao support (P4 if regional expansion happens).
- Voice-input vi-VN composition (P3 mobile).

## Dependencies

- FR-EMAIL-001 / FR-EMAIL-002 / FR-EMAIL-004.
- FR-DESIGN-001 (Be Vietnam Pro typography).
- FR-INFRA-001 (PGroonga).
- FR-BRAIN-001 / FR-BRAIN-002 (per-recipient memory persistence).
- FR-AI-001 (self-hosted tone-helper model on the same GPU node).
- FR-MCP-001.
- A community Vietnamese Hunspell dictionary mirror in our private artefact store.
- Compliance: PDPL Decree 13 (recipient name / contact data is personal data; the per-recipient profile inherits BRAIN's controls).
- Locked decisions referenced: DEC-086 (vi-VN as composition default for VN-recipients), DEC-087 (Vietnamese spelling assist runs locally; never round-trips).

## AI Risk Assessment

The auto-honorific suggestion + suggested-replies localisation are AI-adjacent surfaces. EU AI Act risk class: `limited`.

### Data Sources

The honorific memory is per-tenant data populated from inbound emails to the same tenant. The CUO localisation pulls per-tenant context. No third-party data; no cross-tenant data. The Hunspell dictionary is community-maintained vocabulary; it does not learn from user input.

### Human Oversight

- Honorific selection is suggested, not enforced; the user picks before sending.
- Templates are user-authored; the AI composes within them, not over them.
- Spelling-assist surfaces underlines, not auto-corrections.
- Sign-off blocks are user-authored.

### Failure Modes

- **Wrong honorific suggested.** The user corrects before sending; the correction feeds the per-recipient memory.
- **Diacritic regression in a custom font.** Caught by FR-DESIGN-001's Vietnamese-typography do/don't gallery + a Storybook regression.
- **Hunspell over-flags.** Per-Member dictionary additions persist (`email.member_dict_addition`); shared additions can be promoted to tenant scope by HR/Ops Lead.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted honorific catalogue, register table, template surface, failure modes.
- **Human review:** `@stephen-cheng` reviewed (vi-native); a second vi-native Member confirms the honorific table at PR-review.
