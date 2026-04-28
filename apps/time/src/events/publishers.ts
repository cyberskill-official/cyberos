/**
 * Events published by @cyberos/time.
 *
 * Naming: `time.{verb}.{noun}` — see SRS §5.4. Add an entry below
 * for every event you emit; `EventName` is a discriminated union that the
 * consumer side uses for type safety.
 */

import { publish } from "@cyberos/events";
import type { RequestContext } from "@cyberos/shared";

export type TimeEventName = never; // populate as you add events

// Example placeholder publisher — remove when you wire a real one.
export async function emitPlaceholder(_ctx: RequestContext): Promise<void> {
  // Intentionally a no-op until the module emits real events.
  if (process.env.NODE_ENV !== "production") return;
  await publish({
    ctx: _ctx,
    source: "TIME",
    name: "time.placeholder.emitted",
    payload: {},
  });
}
