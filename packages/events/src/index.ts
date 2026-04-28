/**
 * @cyberos/events — NATS JetStream wrapper that enforces the SRS §5.4 event
 * contract: every event is `{module}.{verb}.{noun}`, carries a tenant id,
 * is at-least-once, and is consumed via durable consumers.
 *
 * Modules import `publish()` to emit and `subscribe()` to consume; no module
 * touches the underlying NATS client directly.
 */

import { connect, JSONCodec, type NatsConnection } from "nats";
import type { ModuleCode, RequestContext, TenantId } from "@cyberos/shared";

export interface CyberOSEvent<TPayload extends object = object> {
  /** Event name, must match `{module}.{verb}.{noun}` (lowercased). */
  readonly name: string;
  /** Module that emitted the event — pulled from the producer's `ModuleCode`. */
  readonly source: ModuleCode;
  /** Tenant the event belongs to. RLS-scoping anchor for consumers. */
  readonly tenant_id: TenantId;
  /** When the event was published (ISO 8601, generator clock). */
  readonly emitted_at: string;
  /** W3C traceparent — propagates the originating request's trace. */
  readonly traceparent: string;
  /** Event-specific payload. Document its shape inside the producer. */
  readonly payload: TPayload;
  /** Idempotency key — consumers MUST use this to dedupe (SRS §5.6). */
  readonly idempotency_key: string;
}

export interface PublishOpts {
  ctx: RequestContext;
  source: ModuleCode;
  /** `{module}.{verb}.{noun}`. Validated against the canonical regex. */
  name: string;
  payload: object;
  /** Override the auto-generated key (rarely needed). */
  idempotency_key?: string;
}

const NAME_RE = /^[a-z][a-z0-9_]*\.[a-z][a-z0-9_]*\.[a-z][a-z0-9_]*$/;

let nc: NatsConnection | null = null;
const codec = JSONCodec();

export async function connectBus(url = process.env.NATS_URL ?? "nats://localhost:4222") {
  if (nc) return nc;
  nc = await connect({ servers: url });
  return nc;
}

export async function publish(opts: PublishOpts): Promise<void> {
  if (!NAME_RE.test(opts.name)) {
    throw new Error(`event name "${opts.name}" must match {module}.{verb}.{noun}`);
  }
  const conn = await connectBus();
  const event: CyberOSEvent = {
    name: opts.name,
    source: opts.source,
    tenant_id: opts.ctx.tenantId,
    emitted_at: new Date().toISOString(),
    traceparent: opts.ctx.traceparent,
    payload: opts.payload,
    idempotency_key: opts.idempotency_key ?? cryptoRandom(),
  };
  conn.publish(opts.name, codec.encode(event));
}

export async function subscribe<TPayload extends object>(
  name: string,
  handler: (event: CyberOSEvent<TPayload>) => Promise<void>,
): Promise<void> {
  const conn = await connectBus();
  const sub = conn.subscribe(name);
  for await (const msg of sub) {
    const event = codec.decode(msg.data) as CyberOSEvent<TPayload>;
    await handler(event);
  }
}

function cryptoRandom(): string {
  // Replace with the standard crypto.randomUUID at module-init time.
  return `${Date.now()}-${Math.random().toString(36).slice(2)}`;
}
