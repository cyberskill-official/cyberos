---
title: ""
author: "@your-handle"
department: engineering
status: draft
priority: p2
created_at: "2026-04-28"
ai_authorship: none
pr_type: feat
breaking_change: false
linked_issues: []
soc2_change_class: standard
template: pull_request@1  # managed by @cyberskill/templates — do not edit
---

# Pull Request

> Turn Your Will Into Real.

## Summary

What changed and why, in two or three sentences. Reviewers should understand the intent without reading the diff.

## Context

Link the issue, the design doc, or the conversation that triggered this PR. If there is no upstream artifact, write the rationale here in plain prose.

## Changes

A short list of the meaningful changes — not a file inventory. Group by area when more than five files are touched.

## How to verify

Concrete steps a reviewer can run. Commands, URLs, expected output. If verification needs a fixture, link or attach it.

## Risk and rollback

What could break, who is affected, and how to revert. For database changes, name the rollback migration. For feature-flagged changes, name the flag.

## Migration

Required only when `breaking_change: true`. Describe what consumers must change, with code examples for before/after. Validator rejects empty Migration sections when the flag is set.

## Post-Incident Review Plan

Required only when `soc2_change_class: emergency`. Name the review owner, target date, and the SOC 2 control reference (CC8.1). Attach the incident ticket.

## AI Authorship Disclosure

Required only when `ai_authorship` is not `none`. Three bullets, no exceptions:

- **Tools used:** (e.g., Claude Sonnet 4.5, Cursor, GitHub Copilot)
- **Scope:** (which sections, files, or lines)
- **Human review:** (who reviewed, what they verified)
