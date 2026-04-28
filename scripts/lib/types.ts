/**
 * Shared types for the FR generator + validator.
 * The schema mirrors `docs/templates/feature_request.md` template@1.
 */

export type Phase = "P0" | "P1" | "P2" | "P3" | "P4";

export type Department =
  | "engineering"
  | "design"
  | "product"
  | "sales"
  | "operations"
  | "hr"
  | "client_success";

export type Status =
  | "draft"
  | "ready_for_review"
  | "in_review"
  | "approved"
  | "in_progress"
  | "blocked"
  | "shipped"
  | "closed";

export type Priority = "p0" | "p1" | "p2" | "p3";

export type AiAuthorship =
  | "none"
  | "assisted"
  | "co_authored"
  | "generated_then_reviewed";

export type FeatureType =
  | "user_facing"
  | "internal_tooling"
  | "integration"
  | "infrastructure";

export type EuAiActRiskClass = "not_ai" | "minimal" | "limited" | "high";

export type Moscow = "MUST" | "SHOULD" | "COULD" | "WONT" | "MUST_NOT";

/** YAML task entry — one per FR. */
export interface TaskEntry {
  id: string;
  module: string;
  phase: Phase;
  moscow: Moscow;
  priority: Priority;
  department: Department;
  feature_type: FeatureType;
  eu_ai_act_risk_class: EuAiActRiskClass;
  client_visible: boolean;
  title: string;
  summary: string;
  /** @default [] */
  tags?: string[];
  /** Other FR ids this depends on. @default [] */
  depends_on?: string[];
  /** Optional override for ai_authorship. @default "none" */
  ai_authorship?: AiAuthorship;
  /** Optional explicit author handle override. @default "@cyberos-bot" */
  author?: string;
  /** Optional target release (SemVer or quarter). */
  target_release?: string;
  /** Optional initial status. @default "draft" */
  status?: Status;
}

/** Top-level shape of `tasks.yaml`. */
export interface TasksYaml {
  version: number;
  template: string;
  source_doc: string;
  source_version: string;
  generated_from?: string;
  tasks: TaskEntry[];
}

/** A validation finding — error blocks merge; warning does not. */
export interface Finding {
  level: "error" | "warning";
  file: string;
  message: string;
}

/** Generator CLI options. */
export interface GenOptions {
  module?: string;
  phase?: Phase;
  dryRun: boolean;
  validateOnly: boolean;
  force: boolean;
}
