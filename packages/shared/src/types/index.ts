/**
 * Shared types — the small set of identifiers that cross module boundaries.
 *
 * Module-internal types stay inside the module. Anything in this file is a
 * cross-module contract; changes here trigger a federation composition diff.
 */

/** Stable tenant identifier. ULID, prefixed `tnt_`. */
export type TenantId = `tnt_${string}`;

/** Stable Member (human user) identifier. ULID, prefixed `mbr_`. */
export type MemberId = `mbr_${string}`;

/** Per-tenant data residency code. Drives DB cluster routing (DEC-011). */
export type Residency = "vn-hcm" | "vn-han" | "sg" | "us-east-1" | "eu-fra";

/** Module identifier — must match the canonical 21 in `modules.yaml`. */
export type ModuleCode =
  | "AUTH" | "AI" | "MCP" | "OBS" | "CHAT" | "BRAIN" | "GENIE"
  | "PROJ" | "TIME" | "CRM" | "KB" | "HR" | "EMAIL" | "REW" | "LEARN"
  | "INV" | "ESOP" | "RES" | "OKR" | "DOC" | "CP";

/** Phase identifier. */
export type Phase = "P0" | "P1" | "P2" | "P3" | "P4";

/**
 * Standard request context propagated to every subgraph + MCP call.
 * Built by AUTH, mounted on `Express.Request.context`, and re-emitted to
 * Apollo `contextValue`. Never mutate after request entry.
 */
export interface RequestContext {
  readonly traceparent: string;
  readonly tenantId: TenantId;
  readonly memberId: MemberId | null;
  readonly residency: Residency;
  readonly roles: readonly string[];
  readonly scopes: readonly string[];
  /** When this request started, for SLO measurement. */
  readonly startedAt: Date;
}

/** Result envelope for every cross-module RPC. Forces explicit error handling. */
export type Result<T, E = Error> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly error: E };
