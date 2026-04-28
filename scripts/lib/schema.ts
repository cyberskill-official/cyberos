/**
 * Schema definitions for feature_request@1 — single source of truth for the
 * generator's frontmatter emission and the validator's checks.
 */

import type { Finding } from "./types.ts";

export const TEMPLATE_ID = "feature_request@1";

/**
 * Canonical frontmatter for `feature_request@1`.
 * Mirrors `@cyberskill/templates` v1.0.0 `feature-request.schema.json` —
 * which has `additionalProperties: false`. The schema is the source of truth;
 * keep this list in sync with `schemas/feature-request.schema.json`.
 */
export const FRONTMATTER_KEY_ORDER = [
  "title",
  "author",
  "department",
  "status",
  "priority",
  "created_at",
  "ai_authorship",
  "feature_type",
  "eu_ai_act_risk_class",
  "target_release",
  "client_visible",
  "template",
] as const;

export const ALLOWED_KEYS: ReadonlySet<string> = new Set(FRONTMATTER_KEY_ORDER);

export const REQUIRED_KEYS = [
  "title",
  "author",
  "department",
  "status",
  "priority",
  "created_at",
  "ai_authorship",
  "feature_type",
  "eu_ai_act_risk_class",
  "client_visible",
  "template",
] as const;

export const ENUMS = {
  department: [
    "engineering",
    "design",
    "product",
    "sales",
    "operations",
    "hr",
    "client_success",
  ],
  // Matches common.schema.json $defs.status — do not extend.
  status: [
    "draft",
    "ready_for_review",
    "in_review",
    "approved",
    "merged",
    "closed",
  ],
  priority: ["p0", "p1", "p2", "p3"],
  ai_authorship: [
    "none",
    "assisted",
    "co_authored",
    "generated_then_reviewed",
  ],
  feature_type: [
    "user_facing",
    "internal_tooling",
    "integration",
    "infrastructure",
  ],
  // `unacceptable` is intentionally excluded — schema reject.
  eu_ai_act_risk_class: ["not_ai", "minimal", "limited", "high"],
  moscow: ["MUST", "SHOULD", "COULD", "WONT", "MUST_NOT"],
  phase: ["P0", "P1", "P2", "P3", "P4"],
} as const satisfies Record<string, readonly string[]>;

/** Required H2 sections in every feature_request@1 file. */
export const REQUIRED_H2 = [
  "Summary",
  "Problem",
  "Proposed Solution",
  "Alternatives Considered",
  "Success Metrics",
  "Scope",
  "Dependencies",
];

/** Conditional H2 sections that must appear when their flag is set. */
export const CONDITIONAL_H2 = {
  ai_risk: {
    when: (fm: Record<string, unknown>) =>
      fm.eu_ai_act_risk_class === "limited" || fm.eu_ai_act_risk_class === "high",
    h2: "AI Risk Assessment",
    requiredH3: ["Data Sources", "Human Oversight", "Failure Modes"],
  },
  customer_quotes: {
    when: (fm: Record<string, unknown>) => fm.client_visible === true,
    h2: "Customer Quotes",
    requiredH3: [],
  },
  sales_cs: {
    when: (fm: Record<string, unknown>) => fm.client_visible === true,
    h2: "Sales/CS Summary",
    requiredH3: [],
  },
  ai_authorship: {
    when: (fm: Record<string, unknown>) =>
      typeof fm.ai_authorship === "string" && fm.ai_authorship !== "none",
    h2: "AI Authorship Disclosure",
    requiredH3: [],
  },
} as const;

/** Validate one parsed doc — returns 0+ findings. */
export function validateDoc(
  filePath: string,
  fm: Record<string, unknown>,
  body: string,
): Finding[] {
  const findings: Finding[] = [];
  const err = (m: string): Finding => ({ level: "error", file: filePath, message: m });
  const warn = (m: string): Finding => ({ level: "warning", file: filePath, message: m });

  // Required keys
  for (const k of REQUIRED_KEYS) {
    if (!(k in fm) || fm[k] === "" || fm[k] === null) {
      findings.push(err(`missing required frontmatter key \`${k}\``));
    }
  }

  // snake_case keys + additionalProperties:false
  for (const key of Object.keys(fm)) {
    if (!/^[a-z][a-z0-9_]*$/.test(key)) {
      findings.push(err(`frontmatter key \`${key}\` is not snake_case`));
    }
    if (!ALLOWED_KEYS.has(key)) {
      findings.push(
        err(`frontmatter key \`${key}\` is not allowed by the canonical schema (additionalProperties: false)`),
      );
    }
  }

  // Enums
  for (const [key, allowed] of Object.entries(ENUMS) as [string, readonly string[]][]) {
    if (key in fm) {
      const v = fm[key];
      if (typeof v !== "string" || !allowed.includes(v)) {
        findings.push(
          err(`\`${key}\` must be one of [${allowed.join(", ")}], got ${JSON.stringify(v)}`),
        );
      }
    }
  }

  // Hard rejection
  if (fm.eu_ai_act_risk_class === "unacceptable") {
    findings.push(err("`eu_ai_act_risk_class: unacceptable` is rejected by schema"));
  }

  // Template pin
  if (fm.template !== TEMPLATE_ID) {
    findings.push(err(`\`template\` must be exactly \`${TEMPLATE_ID}\``));
  }

  // client_visible must be boolean
  if ("client_visible" in fm && typeof fm.client_visible !== "boolean") {
    findings.push(err("`client_visible` must be a boolean"));
  }

  // title length
  if (typeof fm.title === "string" && fm.title.length > 72) {
    findings.push(err(`title exceeds 72 chars (got ${fm.title.length})`));
  }

  // ISO date
  if (typeof fm.created_at === "string" && !/^\d{4}-\d{2}-\d{2}/.test(fm.created_at)) {
    findings.push(err(`created_at must be ISO 8601 (YYYY-MM-DD), got ${fm.created_at}`));
  }

  // Required H2 sections
  for (const h of REQUIRED_H2) {
    const re = new RegExp(`^##\\s+${escapeRe(h)}\\s*$`, "m");
    if (!re.test(body)) findings.push(err(`missing required section \`## ${h}\``));
  }

  // Conditional H2 sections
  for (const [name, rule] of Object.entries(CONDITIONAL_H2)) {
    if (rule.when(fm)) {
      const re = new RegExp(`^##\\s+${escapeRe(rule.h2)}\\s*$`, "m");
      if (!re.test(body)) {
        findings.push(err(`required section \`## ${rule.h2}\` missing (rule: ${name})`));
      }
      for (const h3 of rule.requiredH3) {
        const re3 = new RegExp(`^###\\s+${escapeRe(h3)}\\s*$`, "m");
        if (!re3.test(body)) {
          findings.push(err(`section \`### ${h3}\` missing inside \`## ${rule.h2}\``));
        }
      }
    }
  }

  // Untrusted-content nesting / injection guard
  const utOpen = (body.match(/<untrusted_content\b/g) ?? []).length;
  const utClose = (body.match(/<\/untrusted_content>/g) ?? []).length;
  if (utOpen !== utClose) {
    findings.push(err("unbalanced <untrusted_content> tags"));
  }
  if (/<untrusted_content[^>]*>[\s\S]*?<untrusted_content/.test(body)) {
    findings.push(err("nested <untrusted_content> blocks are not allowed"));
  }
  if (/(?:ignore|disregard).{0,20}previous.{0,20}instructions/i.test(body)) {
    findings.push(warn("possible prompt-injection marker detected in body"));
  }

  // Canonical body is English-only — flag any inline _VN: glosses as a warning,
  // since Vietnamese content belongs in the per-template README_VI.md, not in
  // the artifact the validator parses.
  if (/_VN:/.test(body)) {
    findings.push(
      warn("inline `_VN:` gloss found — canonical bodies are English-only; move VN copy to README_VI.md"),
    );
  }

  return findings;
}

function escapeRe(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
