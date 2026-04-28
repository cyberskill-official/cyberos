---
title: ""
author: "@your-handle"
department: engineering
status: draft
priority: p2
created_at: "2026-04-28"
ai_authorship: none
severity: sev3
affected_versions: []
pdpl_breach_suspected: false
discovered_at: ""
reproducible: intermittent
template: bug_report@1  # managed by @cyberskill/templates — do not edit
---

# Bug Report

> Turn Your Will Into Real.

## Summary

One or two sentences naming the user-visible symptom and the area of the product affected.

## Reporter Description

Verbatim words from whoever reported the bug. If the source is a customer email, paste it inside the untrusted block below — do not paraphrase. Paraphrasing in this section hides whether the customer is upset.

<untrusted_content source="customer_email">
…paste verbatim quoted content here…
</untrusted_content>

## Steps to Reproduce

Numbered steps. If the bug requires a specific account, dataset, or environment, name it. If the reporter's reproduction is the only one available, paste it inside the untrusted block.

<untrusted_content source="jira">
…paste verbatim quoted content here…
</untrusted_content>

## Expected Behaviour

What should have happened. Tie it to the spec, the docs, or the prior behaviour the user is comparing against.

## Actual Behaviour

What happened instead. Include error messages verbatim. Include screenshots, logs, and request IDs.

## Environment

Versions, OS, browser, region, tenant ID. Anything that narrows the search. Mirror the `affected_versions` field above.

## Impact

Who is affected, how many, and what they can or cannot do because of this. Used to set `priority` and `severity`.

## Breach Containment

Required only when `pdpl_breach_suspected: true`. Vietnam PDPL Article 23: notification within 72 hours of discovery. Describe the immediate containment actions taken, who took them, and the residual exposure.

## Notification Plan

Required only when `pdpl_breach_suspected: true`. Identify the data subjects, the regulators to notify, the deadline (compute from `discovered_at`), and the owner.

## AI Authorship Disclosure

Required only when `ai_authorship` is not `none`. Same three-bullet shape as the PR template:

- **Tools used:**
- **Scope:**
- **Human review:**
