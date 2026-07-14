import { decodeJwt } from "./api";

// TASK-CHAT-269 §1 #2 / #18 — who may see the moderation surface.
//
// These MUST stay in step with MODERATOR_ROLES in services/chat/src/auth.rs. The client copy exists only to
// decide whether to RENDER the entry point; it is not a security boundary and must never be treated as one.
// Every one of the three admin routes re-checks the role server-side and fails closed, so a user who edits
// this array in devtools gets a 403, not a queue.
const MODERATOR_ROLES = ["tenant-admin", "root-admin"];

/// Read the workspace roles out of the access token's `roles` claim.
///
/// Fails closed, deliberately and in the same way the server does: a token with no `roles` claim at all
/// yields `[]`, which grants nothing. "Unknown" is not "allow".
export function rolesOf(token: string | null): string[] {
  if (!token) return [];
  const claims = decodeJwt(token);
  const roles = claims?.["roles"];
  return Array.isArray(roles) ? roles.filter((r): r is string => typeof r === "string") : [];
}

/// §1 #18 — the Moderation entry is rendered ONLY for an administrator. Absent, not disabled: a
/// visible-but-403 route teaches everyone in the workspace that a moderation surface exists and that they are
/// not trusted with it, which is a worse outcome than not knowing.
export function isModerator(token: string | null): boolean {
  return rolesOf(token).some((r) => MODERATOR_ROLES.includes(r));
}
