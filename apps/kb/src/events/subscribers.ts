/**
 * Events @cyberos/kb subscribes to from other modules.
 *
 * Each subscription is registered at module boot in `index.ts` (or a small
 * `bootstrapSubscribers()` called from there). Handlers must:
 *   - be idempotent (events are at-least-once; SRS §5.4)
 *   - dedupe via `event.idempotency_key`
 *   - never throw without rolling back the side effect
 */

import { subscribe, type CyberOSEvent } from "@cyberos/events";

export async function bootstrapSubscribers(): Promise<void> {
  // Example pattern — replace with the events this module actually consumes.
  await subscribe<{ example: string }>("placeholder.example.created", async (event) => {
    // TODO: handle event using event.payload, event.tenant_id, event.idempotency_key
    void event;
  });
}

// Re-export for clarity at the call site.
export type { CyberOSEvent };
