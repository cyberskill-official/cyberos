/**
 * Tenancy primitives — every module reads tenant context through these helpers
 * so the 3-layer isolation rule (JWT → Postgres session var → RLS) is uniform.
 *
 * Detail in SRS §5.1 (residency engineering) and §11 (auth + multi-tenancy).
 */

import { CyberOSError, Errors } from "../errors/index.ts";
import type { RequestContext, TenantId } from "../types/index.ts";

/**
 * Pull the tenant id off a context, throwing if absent.
 * Modules that need a tenant id should never reach into the JWT claims;
 * always go through this helper so the error is uniform.
 */
export function requireTenant(ctx: RequestContext): TenantId {
  if (!ctx.tenantId) throw Errors.unauthorized("missing tenant context");
  return ctx.tenantId;
}

/** Throw if the caller's roles do not include any of `accepted`. */
export function requireAnyRole(ctx: RequestContext, accepted: readonly string[]): void {
  if (!ctx.roles.some((r) => accepted.includes(r))) {
    throw Errors.forbidden(`role required: ${accepted.join(" | ")}`, {
      ownedRoles: ctx.roles,
    });
  }
}

/** Throw if the caller's scopes do not include `required`. */
export function requireScope(ctx: RequestContext, required: string): void {
  if (!ctx.scopes.includes(required)) {
    throw Errors.forbidden(`scope required: ${required}`, { ownedScopes: ctx.scopes });
  }
}

/**
 * Assert that a residency-tagged resource matches the caller's residency.
 * Cross-residency reads are a hard error — emit a 403, not a 404, so the
 * audit log can see the attempt (DEC-011).
 */
export function assertResidencyMatch(
  ctx: RequestContext,
  resourceResidency: RequestContext["residency"],
): void {
  if (ctx.residency !== resourceResidency) {
    throw Errors.residencyViolation("residency mismatch", {
      caller: ctx.residency,
      resource: resourceResidency,
    });
  }
}

export { CyberOSError };
