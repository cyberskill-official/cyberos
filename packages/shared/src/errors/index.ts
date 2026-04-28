/**
 * Canonical error taxonomy — every module throws one of these. The Apollo
 * formatter and the MCP error handler map them to wire-level codes the same
 * way, so a 403 at the GraphQL boundary equals an `unauthorized` MCP error.
 *
 * Add a code here, not in a module — the central catalogue is what makes
 * client-side handling uniform.
 */

export type CyberOSErrorCode =
  | "unauthorized"
  | "forbidden"
  | "not_found"
  | "validation_failed"
  | "conflict"
  | "rate_limited"
  | "tenant_residency_violation"
  | "ai_budget_exceeded"
  | "internal_error";

export class CyberOSError extends Error {
  readonly code: CyberOSErrorCode;
  readonly httpStatus: number;
  readonly details?: Readonly<Record<string, unknown>>;

  constructor(
    code: CyberOSErrorCode,
    message: string,
    details?: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "CyberOSError";
    this.code = code;
    this.httpStatus = HTTP_STATUS[code];
    if (details !== undefined) {
      this.details = details;
    }
  }
}

const HTTP_STATUS: Record<CyberOSErrorCode, number> = {
  unauthorized: 401,
  forbidden: 403,
  not_found: 404,
  validation_failed: 422,
  conflict: 409,
  rate_limited: 429,
  tenant_residency_violation: 403,
  ai_budget_exceeded: 429,
  internal_error: 500,
};

/** Convenience constructors — readable at the call site. */
export const Errors = {
  unauthorized: (m = "unauthorized", d?: Record<string, unknown>) =>
    new CyberOSError("unauthorized", m, d),
  forbidden: (m = "forbidden", d?: Record<string, unknown>) =>
    new CyberOSError("forbidden", m, d),
  notFound: (m = "not found", d?: Record<string, unknown>) =>
    new CyberOSError("not_found", m, d),
  validation: (m: string, d?: Record<string, unknown>) =>
    new CyberOSError("validation_failed", m, d),
  conflict: (m: string, d?: Record<string, unknown>) =>
    new CyberOSError("conflict", m, d),
  rateLimited: (m = "rate limited", d?: Record<string, unknown>) =>
    new CyberOSError("rate_limited", m, d),
  residencyViolation: (m: string, d?: Record<string, unknown>) =>
    new CyberOSError("tenant_residency_violation", m, d),
  aiBudgetExceeded: (m = "AI budget exceeded", d?: Record<string, unknown>) =>
    new CyberOSError("ai_budget_exceeded", m, d),
  internal: (m = "internal error", d?: Record<string, unknown>) =>
    new CyberOSError("internal_error", m, d),
} as const;
