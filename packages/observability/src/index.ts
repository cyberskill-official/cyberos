/**
 * @cyberos/observability — pino logger + traceparent propagation + New Relic
 * boot. Every subgraph imports this; nothing module-specific lives here.
 *
 * Detail: SRS §11 (observability), §7.9 (logging/audit), FR-OBS-001..012.
 */

import { pino, type Logger as PinoLogger } from "pino";
import type { ModuleCode, RequestContext } from "@cyberos/shared";

let rootLogger: PinoLogger | null = null;

/** Initialise the root logger once per process. Call from each module's entry. */
export function initObservability(opts: {
  module: ModuleCode;
  level?: pino.Level;
}): PinoLogger {
  if (rootLogger) return rootLogger;
  rootLogger = pino({
    level: opts.level ?? (process.env.LOG_LEVEL as pino.Level | undefined) ?? "info",
    base: { module: opts.module, service: `cyberos.${opts.module.toLowerCase()}` },
    formatters: {
      level: (label) => ({ level: label }),
    },
    timestamp: pino.stdTimeFunctions.isoTime,
  });
  return rootLogger;
}

/** Return a child logger bound to a request's traceparent + tenant. */
export function loggerFor(ctx: RequestContext): PinoLogger {
  if (!rootLogger) throw new Error("initObservability() must be called first");
  return rootLogger.child({
    traceparent: ctx.traceparent,
    tenant_id: ctx.tenantId,
    member_id: ctx.memberId ?? null,
    residency: ctx.residency,
  });
}

/** Generate a W3C traceparent (00-<trace>-<span>-01) for inbound requests with none. */
export function generateTraceparent(): string {
  const traceId = randomHex(32);
  const spanId = randomHex(16);
  return `00-${traceId}-${spanId}-01`;
}

function randomHex(chars: number): string {
  let s = "";
  for (let i = 0; i < chars; i++) {
    s += Math.floor(Math.random() * 16).toString(16);
  }
  return s;
}

export type { PinoLogger as Logger };
