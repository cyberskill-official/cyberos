#!/usr/bin/env node
// apply-resolutions.mjs
//
// Companion to apply-resolutions.sh. Idempotent — safe to re-run.
//
// Subcommands:
//   modules     — overwrite modules.yaml with the resolved 25-module manifest
//   frontmatter — re-add `template: feature_request@1` to every FR if missing
//   sections    — add stub sections (Alternatives Considered, AI Authorship
//                 Disclosure, Customer Quotes, Sales/CS Summary) to every FR
//                 where missing; uses frontmatter.client_visible to decide
//                 whether Customer Quotes + Sales/CS Summary apply
//   readme      — update docs/tasks/README.md §3 to document canonical
//                 section list
//   audit       — scan every FR; report any required sections still missing

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const TASKS_DIR = path.join(REPO_ROOT, 'docs', 'tasks');
const MODULES_YAML = path.join(REPO_ROOT, 'modules.yaml');
const README_MD = path.join(TASKS_DIR, 'README.md');

const cmd = process.argv[2];
if (!cmd) {
  console.error('Usage: apply-resolutions.mjs <modules|frontmatter|sections|readme|audit>');
  process.exit(2);
}

// ─── Helpers ──────────────────────────────────────────────────────────────

function listFRFiles() {
  const files = [];
  for (const entry of fs.readdirSync(TASKS_DIR)) {
    const full = path.join(TASKS_DIR, entry);
    if (!fs.statSync(full).isDirectory()) continue;
    if (!entry.startsWith('batch-')) continue;
    for (const f of fs.readdirSync(full)) {
      if (f.startsWith('FR-') && f.endsWith('.md')) files.push(path.join(full, f));
    }
  }
  return files.sort();
}

function parseFR(content) {
  const m = content.match(/^---\n([\s\S]*?)\n---\n([\s\S]*)$/);
  if (!m) throw new Error('No frontmatter');
  const fm = {};
  for (const line of m[1].split('\n')) {
    const lm = line.match(/^([a-z_]+):\s*"?([^"#]*?)"?\s*(?:#.*)?$/);
    if (lm) fm[lm[1]] = lm[2].trim();
  }
  return { frontmatter: fm, body: m[2], rawFrontmatter: m[1] };
}

function listSectionHeaders(body) {
  const out = [];
  for (const line of body.split('\n')) {
    const m = line.match(/^##\s+(.+)$/);
    if (m) out.push(m[1].trim());
  }
  return out;
}

function hasSection(body, name) {
  // Match "## Section Name" — exact or with annotations like "(...)".
  const re = new RegExp(`^##\\s+${name.replace(/[-/\\^$*+?.()|[\]{}]/g, '\\$&')}\\b`, 'm');
  return re.test(body);
}

// ─── Subcommand: modules ──────────────────────────────────────────────────

function cmdModules() {
  const content = `# CyberOS — module manifest.
#
# This file is the single source of truth for the 25 runtime modules. Editing
# it + running \`pnpm gen:module\` re-stamps the apps/{code}/ folder so every
# module has the same canonical shape.
#
# Order is the canonical order for sprint pickup; do not reorder. Ports are
# deterministic and MUST NOT be reused — once a module owns a port, it owns it
# forever.
#
# Adding a new module:
#   1. Append a new entry below (next free port).
#   2. Run \`pnpm gen:module --module {CODE}\`.
#   3. Open the generated apps/{code}/README.md and start filling in.
#
# Non-runtime concerns (CORP — Singapore HoldCo flip; GTM — marketing site +
# Trust Center + launch playbook) are tracked in docs/tasks/ only and are not
# listed here because they have no Apollo subgraph + no Prisma schema +
# no MCP namespace.

version: 2
generated_apps_root: apps

modules:
  # ─── P0 · Foundation ──────────────────────────────────────────────────────

  - code: AUTH
    name: Authentication & Tenancy
    phase: P0
    package: "@cyberos/auth"
    port: 4001
    graphql_namespace: auth
    prisma_schema: auth
    mcp_namespace: auth
    department: engineering
    depends_on: []

  - code: AI
    name: AI Gateway
    phase: P0
    package: "@cyberos/ai"
    port: 4002
    graphql_namespace: ai
    prisma_schema: ai
    mcp_namespace: ai
    department: engineering
    depends_on: [AUTH]

  - code: MCP
    name: MCP Server
    phase: P0
    package: "@cyberos/mcp"
    port: 4003
    graphql_namespace: mcp
    prisma_schema: mcp
    mcp_namespace: mcp
    department: engineering
    depends_on: [AUTH]

  - code: OBS
    name: Observability
    phase: P0
    package: "@cyberos/obs"
    port: 4004
    graphql_namespace: obs
    prisma_schema: obs
    mcp_namespace: obs
    department: engineering
    depends_on: [AUTH]

  - code: CHAT
    name: Internal Chat
    phase: P0
    package: "@cyberos/chat"
    port: 4005
    graphql_namespace: chat
    prisma_schema: chat
    mcp_namespace: chat
    department: product
    depends_on: [AUTH, BRAIN, GENIE]

  - code: BRAIN
    name: Universal Knowledge Layer
    phase: P0
    package: "@cyberos/brain"
    port: 4006
    graphql_namespace: brain
    prisma_schema: brain
    mcp_namespace: brain
    department: engineering
    depends_on: [AUTH, AI]

  - code: GENIE
    name: Genie / CUO Persona Surface
    phase: P0
    package: "@cyberos/genie"
    port: 4007
    graphql_namespace: genie
    prisma_schema: genie
    mcp_namespace: genie
    department: product
    depends_on: [AUTH, AI, BRAIN, MCP]

  # ─── P1 · Run the company ─────────────────────────────────────────────────

  - code: PROJ
    name: Projects & Tasks
    phase: P1
    package: "@cyberos/proj"
    port: 4008
    graphql_namespace: proj
    prisma_schema: proj
    mcp_namespace: proj
    department: product
    depends_on: [AUTH, CHAT]

  - code: TIME
    name: Time Tracking
    phase: P1
    package: "@cyberos/time"
    port: 4009
    graphql_namespace: time
    prisma_schema: time
    mcp_namespace: time
    department: operations
    depends_on: [AUTH, PROJ]

  - code: CRM
    name: Customer Relationships
    phase: P1
    package: "@cyberos/crm"
    port: 4010
    graphql_namespace: crm
    prisma_schema: crm
    mcp_namespace: crm
    department: sales
    depends_on: [AUTH, EMAIL]

  - code: KB
    name: Knowledge Base
    phase: P1
    package: "@cyberos/kb"
    port: 4011
    graphql_namespace: kb
    prisma_schema: kb
    mcp_namespace: kb
    department: product
    depends_on: [AUTH, BRAIN]

  - code: EMAIL
    name: Email Client
    phase: P1
    package: "@cyberos/email"
    port: 4013
    graphql_namespace: email
    prisma_schema: email
    mcp_namespace: email
    department: product
    depends_on: [AUTH]

  # ─── P2 · Operationalize ──────────────────────────────────────────────────

  - code: HR
    name: Human Resources
    phase: P2
    package: "@cyberos/hr"
    port: 4012
    graphql_namespace: hr
    prisma_schema: hr
    mcp_namespace: hr
    department: hr
    depends_on: [AUTH]

  - code: REW
    name: Total Rewards
    phase: P2
    package: "@cyberos/rew"
    port: 4014
    graphql_namespace: rew
    prisma_schema: rew
    mcp_namespace: rew
    department: hr
    eu_ai_act_risk_class: high
    depends_on: [AUTH, HR, TIME]

  - code: LEARN
    name: Career Path & Learning
    phase: P2
    package: "@cyberos/learn"
    port: 4015
    graphql_namespace: learn
    prisma_schema: learn
    mcp_namespace: learn
    department: hr
    eu_ai_act_risk_class: high
    depends_on: [AUTH, HR, REW]

  - code: INV
    name: Invoicing
    phase: P2
    package: "@cyberos/inv"
    port: 4016
    graphql_namespace: inv
    prisma_schema: inv
    mcp_namespace: inv
    department: operations
    depends_on: [AUTH, CRM, PROJ]

  - code: ESOP
    name: Phantom Stock
    phase: P2
    package: "@cyberos/esop"
    port: 4017
    graphql_namespace: esop
    prisma_schema: esop
    mcp_namespace: esop
    department: hr
    eu_ai_act_risk_class: high
    depends_on: [AUTH, HR, REW]

  - code: RES
    name: Resource Allocation
    phase: P2
    package: "@cyberos/res"
    port: 4018
    graphql_namespace: res
    prisma_schema: res
    mcp_namespace: res
    department: operations
    depends_on: [AUTH, PROJ, TIME, HR]

  - code: OKR
    name: Objectives & Key Results
    phase: P2
    package: "@cyberos/okr"
    port: 4019
    graphql_namespace: okr
    prisma_schema: okr
    mcp_namespace: okr
    department: product
    depends_on: [AUTH]

  # ─── P3 · SaaS readiness ──────────────────────────────────────────────────

  - code: DOC
    name: Document Signing
    phase: P3
    package: "@cyberos/doc"
    port: 4020
    graphql_namespace: doc
    prisma_schema: doc
    mcp_namespace: doc
    department: operations
    depends_on: [AUTH, CRM]

  - code: TEN
    name: Tenancy & Lifecycle
    phase: P3
    package: "@cyberos/ten"
    port: 4023
    graphql_namespace: ten
    prisma_schema: ten
    mcp_namespace: ten
    department: engineering
    depends_on: [AUTH]

  - code: BILL
    name: Subscription Billing
    phase: P3
    package: "@cyberos/bill"
    port: 4024
    graphql_namespace: bill
    prisma_schema: bill
    mcp_namespace: bill
    department: operations
    depends_on: [AUTH, TEN, INV]

  - code: CP
    name: Compliance Plane
    phase: P0  # cross-cutting; built up across P0 → P3
    package: "@cyberos/cp"
    port: 4022
    graphql_namespace: cp
    prisma_schema: cp
    mcp_namespace: cp
    department: legal
    depends_on: [AUTH]

  # ─── P4 · Externalise ─────────────────────────────────────────────────────

  - code: PORTAL
    name: Client Portal
    phase: P4
    package: "@cyberos/portal"
    port: 4021  # was prior CP — same port, new code
    graphql_namespace: portal
    prisma_schema: portal
    mcp_namespace: portal
    department: client_success
    depends_on: [AUTH, PROJ, INV, DOC]

  - code: API
    name: Public API Gateway
    phase: P4
    package: "@cyberos/api"
    port: 4025
    graphql_namespace: api
    prisma_schema: api
    mcp_namespace: api
    department: engineering
    depends_on: [AUTH, TEN, BILL]
`;
  fs.writeFileSync(MODULES_YAML, content);
  console.log(`    ✓ ${MODULES_YAML} written (25 modules)`);
}

// ─── Subcommand: frontmatter ──────────────────────────────────────────────

function cmdFrontmatter() {
  const files = listFRFiles();
  let added = 0;
  for (const fp of files) {
    let c = fs.readFileSync(fp, 'utf8');
    if (/^template: feature_request@1$/m.test(c)) continue;
    // Insert template line right before the closing --- of the frontmatter
    c = c.replace(
      /(^---\n[\s\S]*?\n)(---\n)/,
      (_, fm, end) => fm + 'template: feature_request@1\n' + end
    );
    fs.writeFileSync(fp, c);
    added++;
  }
  console.log(`    ✓ template field added to ${added} / ${files.length} FRs`);
}

// ─── Subcommand: sections ─────────────────────────────────────────────────

const STUB_ALTERNATIVES = `## Alternatives Considered

The shape of the answer has been deliberately constrained by the architectural rules in §2 of \`README.md\` and the locked decisions cited in *Dependencies*. Notable rejected approaches:

- Approaches that would have allowed AI to make compensation, equity, or document-signing decisions — rejected per the "AI describes, humans decide" rule.
- Approaches that would have created cross-tenant read or write paths — rejected per the cross-tenant invariant (FR-TEN-001 invariant test harness).
- Where there are FR-specific alternatives, they're discussed inline in *Proposed Solution* and *Constraints*.

<!-- TODO during implementation PR: replace with FR-specific rejected alternatives. -->
`;

const STUB_AI_AUTHORSHIP = `## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from \`ready_for_review\`.
- **Human review:** founder (\`@stephen-cheng\`) — final wording is the founder's responsibility.
`;

const STUB_CUSTOMER_QUOTES = `## Customer Quotes

<!-- Required when client_visible: true. Verbatim, attributed where possible. Paraphrasing here costs you the signal. -->

<untrusted_content source="other">
…paste verbatim customer quote here…
</untrusted_content>

<!-- TODO during implementation PR: capture real customer quotes from sales calls / NPS / support tickets. -->
`;

const STUB_SALES_CS_SUMMARY = `## Sales/CS Summary

<!-- Required when client_visible: true. One paragraph written so a non-engineer can pitch the feature. Plain English. No internal jargon, no module codes, no speculation about future scope. -->

<!-- TODO during implementation PR: write the customer-facing pitch. -->
`;

function insertAfter(body, anchorRegex, insertion) {
  // Insert `insertion` right after the section ending with the next `## ` header
  // (or at the end of the file if no next header).
  const m = body.match(anchorRegex);
  if (!m) return body; // anchor not found
  const start = m.index + m[0].length;
  // Find the next `## ` header after `start`
  const rest = body.slice(start);
  const next = rest.match(/^## /m);
  if (next) {
    const insertPos = start + next.index;
    return body.slice(0, insertPos) + insertion + '\n' + body.slice(insertPos);
  }
  return body.slice(0, start) + '\n' + insertion;
}

function insertBefore(body, anchorRegex, insertion) {
  const m = body.match(anchorRegex);
  if (!m) return body;
  return body.slice(0, m.index) + insertion + '\n' + body.slice(m.index);
}

function cmdSections() {
  const files = listFRFiles();
  let altCount = 0, aiCount = 0, cqCount = 0, scsCount = 0;
  for (const fp of files) {
    let c = fs.readFileSync(fp, 'utf8');
    const fr = parseFR(c);
    const isClientVisible = /^(true|yes)$/i.test(fr.frontmatter.client_visible || '');

    let body = fr.body;
    let changed = false;

    // 1. Add Alternatives Considered after Proposed Solution
    if (!hasSection(body, 'Alternatives Considered')) {
      // Insert before "## Out of Scope" (which always follows Proposed Solution
      // in the canonical order). Fallback: before "## Dependencies".
      const before = body.match(/^## Out of Scope/m) ? /^## Out of Scope/m
                   : body.match(/^## Dependencies/m) ? /^## Dependencies/m
                   : null;
      if (before) {
        body = insertBefore(body, before, STUB_ALTERNATIVES);
        altCount++;
        changed = true;
      }
    }

    // 2. Add AI Authorship Disclosure before References (or before final --- divider)
    if (!hasSection(body, 'AI Authorship Disclosure')) {
      const before = body.match(/^## References/m) ? /^## References/m
                   : body.match(/^---\n\*ai_authorship/m) ? /^---\n\*ai_authorship/m
                   : null;
      if (before) {
        body = insertBefore(body, before, STUB_AI_AUTHORSHIP);
        aiCount++;
        changed = true;
      } else {
        // Append at end
        body += '\n' + STUB_AI_AUTHORSHIP;
        aiCount++;
        changed = true;
      }
    }

    // 3. Customer Quotes — only if client_visible: true. Insert before Proposed Solution.
    if (isClientVisible && !hasSection(body, 'Customer Quotes')) {
      const before = body.match(/^## Proposed Solution/m);
      if (before) {
        body = insertBefore(body, /^## Proposed Solution/m, STUB_CUSTOMER_QUOTES);
        cqCount++;
        changed = true;
      }
    }

    // 4. Sales/CS Summary — only if client_visible: true. Insert before Open Questions
    //    (or before References if no Open Questions).
    if (isClientVisible && !hasSection(body, 'Sales/CS Summary')) {
      const anchor = body.match(/^## Open Questions/m) ? /^## Open Questions/m
                   : body.match(/^## References/m) ? /^## References/m
                   : null;
      if (anchor) {
        body = insertBefore(body, anchor, STUB_SALES_CS_SUMMARY);
        scsCount++;
        changed = true;
      }
    }

    if (changed) {
      fs.writeFileSync(fp, '---\n' + fr.rawFrontmatter + '\n---\n' + body);
    }
  }
  console.log(`    ✓ Alternatives Considered: added to ${altCount} FRs`);
  console.log(`    ✓ AI Authorship Disclosure: added to ${aiCount} FRs`);
  console.log(`    ✓ Customer Quotes: added to ${cqCount} client-visible FRs`);
  console.log(`    ✓ Sales/CS Summary: added to ${scsCount} client-visible FRs`);
}

// ─── Subcommand: readme ───────────────────────────────────────────────────

const README_SECTIONS_BLOCK = `These rules are pinned so that every batch stays consistent without re-stating them:

1. **Canonical section list.** Every FR uses the canonical section list below. Sections marked **(required)** are always present (may be a stub). Sections marked **(conditional)** are present only when the trigger holds.

   1. \`Summary\` (required)
   2. \`Problem\` (required)
   3. \`Customer Quotes\` (conditional — required when \`client_visible: true\`)
   4. \`Proposed Solution\` (required)
   5. \`Alternatives Considered\` (required)
   6. \`Out of Scope\` (required)
   7. \`Dependencies\` (required)
   8. \`Constraints\` (required)
   9. \`Compliance / Privacy\` (required when the FR touches personal data, financial data, AI surfaces, or any compliance regime)
   10. \`Risk Assessment\` (conditional — required when \`eu_ai_act_risk_class\` is \`limited\` or \`high\`; with subsections *Data Sources*, *Human Oversight*, *Failure Modes*)
   11. \`Vietnamese-locale considerations\` (required when the FR has a user-visible surface)
   12. \`Scope\` (required, with at least one Gherkin acceptance block in \`\`\`gherkin\`\`\` fence)
   13. \`Success Metrics\` (required)
   14. \`Sales/CS Summary\` (conditional — required when \`client_visible: true\`)
   15. \`AI Authorship Disclosure\` (conditional — required when \`ai_authorship\` is not \`none\`; three bullets: *Tools used*, *Scope*, *Human review*)
   16. \`Open Questions\` (required, can be empty)
   17. \`References\` (required)

2. **Frontmatter.** Each FR has the canonical frontmatter fields (\`title\`, \`author\`, \`department\`, \`status\`, \`priority\`, \`created_at\`, \`ai_authorship\`, \`feature_type\`, \`eu_ai_act_risk_class\`, \`target_release\`, \`client_visible\`, \`template: feature_request@1\`).

3. **No invented facts.** Every numerical target, every module name, every locked-decision reference (\`DEC-XXX\`), every NFR ID, every FR cross-reference, and every compliance regime traces back to a citation in the PRD or SRS. Sources are listed in the FR's \`Dependencies\` section. If the spec is silent, the FR explicitly marks the gap as an *Open Question* (\`OQ-XXX\`) and routes resolution to the founder.

4. **Auditable acceptance criteria.** Every FR ends its \`Scope\` and \`Success Metrics\` with criteria that a CI job, a phase-gate review, or an external auditor can verify with no human interpretation. Where Gherkin is appropriate, the FR uses Gherkin verbatim.

5. **AI Risk Assessment.** When the feature emits AI-generated content visible to a natural person, \`eu_ai_act_risk_class: limited\` is the floor and the three required subsections are filled. When the feature decides on compensation, equity, hiring, or any HR-impacting axis, \`eu_ai_act_risk_class: high\` and Article 14 human-oversight controls are spelled out at the system-property level.

6. **Vietnamese-first.** Where a feature has a user-visible surface, the FR explicitly addresses Vietnamese-locale behaviour (PGroonga tokenisation, Be Vietnam Pro typography, Anh/Chị salutations, vi-VN as default locale).

7. **Compliance cross-references.** Every FR that touches personal data names the applicable regime: PDPL Law 91/2025 + Decree 356/2025 (Vietnam), Decree 13/2023, GDPR (EU), EU AI Act Articles 5–7 + 14 + 50, SOC 2 trust criteria, ISO/IEC 27001, and where relevant ISO/IEC 42001.

8. **Locked decisions.** When an FR depends on a locked decision, it cites the \`DEC-XXX\` ID and the PRD §11.1 or SRS Decisions Log section. FRs never silently override locked decisions; a change request must be filed against the decisions log first.

`;

function cmdReadme() {
  let content = fs.readFileSync(README_MD, 'utf8');
  // Replace the "## 3. Generation rules" section's body with the new block.
  // We match from the heading until the next "---" divider or "## " heading.
  const re = /(^## 3\. Generation rules \(binding for every batch\)\n)([\s\S]*?)(^---\n)/m;
  if (re.test(content)) {
    content = content.replace(re, '$1\n' + README_SECTIONS_BLOCK + '$3');
    fs.writeFileSync(README_MD, content);
    console.log(`    ✓ docs/tasks/README.md §3 updated`);
  } else {
    console.log(`    ⚠ §3 anchor not found in docs/tasks/README.md — manual edit required`);
  }
}

// ─── Subcommand: audit ────────────────────────────────────────────────────

function cmdAudit() {
  const files = listFRFiles();
  const REQUIRED_ALWAYS = [
    'Summary', 'Problem', 'Proposed Solution', 'Alternatives Considered',
    'Out of Scope', 'Dependencies', 'Constraints',
    'Scope', 'Success Metrics', 'AI Authorship Disclosure',
    'Open Questions', 'References',
  ];
  const REQUIRED_IF_CLIENT_VISIBLE = ['Customer Quotes', 'Sales/CS Summary'];

  let okCount = 0;
  const issues = [];
  for (const fp of files) {
    const c = fs.readFileSync(fp, 'utf8');
    const fr = parseFR(c);
    const isClientVisible = /^(true|yes)$/i.test(fr.frontmatter.client_visible || '');

    const missing = [];
    for (const sec of REQUIRED_ALWAYS) {
      if (!hasSection(fr.body, sec)) missing.push(sec);
    }
    if (isClientVisible) {
      for (const sec of REQUIRED_IF_CLIENT_VISIBLE) {
        if (!hasSection(fr.body, sec)) missing.push(sec);
      }
    }
    // Extra checks
    const issuesForThis = [];
    if (!/^template: feature_request@1$/m.test(fr.rawFrontmatter)) {
      issuesForThis.push('frontmatter missing `template: feature_request@1`');
    }
    if (!/```gherkin\n/.test(fr.body)) {
      issuesForThis.push('no Gherkin acceptance block in Scope section');
    }
    if (missing.length === 0 && issuesForThis.length === 0) {
      okCount++;
    } else {
      issues.push({
        path: path.relative(REPO_ROOT, fp),
        missing,
        issuesForThis,
        isClientVisible,
      });
    }
  }

  console.log(`    Total FRs:    ${files.length}`);
  console.log(`    Clean:        ${okCount}`);
  console.log(`    With issues:  ${issues.length}`);
  if (issues.length > 0) {
    console.log('');
    console.log('    Issues:');
    for (const it of issues.slice(0, 20)) {
      const cv = it.isClientVisible ? ' (client_visible)' : '';
      console.log(`      ${it.path}${cv}`);
      for (const m of it.missing) console.log(`        - missing: ${m}`);
      for (const m of it.issuesForThis) console.log(`        - ${m}`);
    }
    if (issues.length > 20) console.log(`      … (${issues.length - 20} more)`);
  }
}

// ─── Dispatch ─────────────────────────────────────────────────────────────

switch (cmd) {
  case 'modules':     cmdModules();     break;
  case 'frontmatter': cmdFrontmatter(); break;
  case 'sections':    cmdSections();    break;
  case 'readme':      cmdReadme();      break;
  case 'audit':       cmdAudit();       break;
  default:
    console.error(`Unknown subcommand: ${cmd}`);
    process.exit(2);
}
