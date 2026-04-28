/**
 * @cyberos/subgraph-kit — every module bootstraps its Apollo subgraph
 * through `startSubgraph()`. This is what enforces:
 *  - the same Apollo Server 5 + Express composition (DEC-024 — no NestJS)
 *  - tenant context extraction from the JWT (FR-AUTH-007)
 *  - traceparent propagation
 *  - uniform error formatting (CyberOSError → GraphQL extensions.code)
 *
 * Modules pass in the schema, resolvers, and a port. Everything else is the
 * same shape across all 21 subgraphs.
 */

import { ApolloServer, type BaseContext } from "@apollo/server";
import { expressMiddleware } from "@apollo/server/express4";
import { buildSubgraphSchema } from "@apollo/subgraph";
import express, { type Express } from "express";
import type { DocumentNode } from "graphql";
import { initObservability, generateTraceparent, type Logger } from "@cyberos/observability";
import {
  CyberOSError,
  type ModuleCode,
  type RequestContext,
  type Residency,
  type TenantId,
  type MemberId,
} from "@cyberos/shared";

export interface CyberOSContext extends BaseContext {
  readonly request: RequestContext;
  readonly logger: Logger;
}

export interface StartSubgraphOpts {
  module: ModuleCode;
  port: number;
  typeDefs: DocumentNode;
  // The resolver tree. Typed as `unknown` because @apollo/subgraph is structural here.
  resolvers: unknown;
  /** Optional Express middleware to mount before the GraphQL handler. */
  configure?: (app: Express) => void;
}

export async function startSubgraph(opts: StartSubgraphOpts): Promise<void> {
  const logger = initObservability({ module: opts.module });
  const app = express();

  app.get("/health", (_req, res) => {
    res.json({ ok: true, module: opts.module });
  });

  if (opts.configure) opts.configure(app);

  const server = new ApolloServer<CyberOSContext>({
    schema: buildSubgraphSchema({ typeDefs: opts.typeDefs, resolvers: opts.resolvers as never }),
    formatError: (formatted, raw) => {
      if (raw instanceof CyberOSError) {
        return {
          ...formatted,
          extensions: {
            ...(formatted.extensions ?? {}),
            code: raw.code,
            httpStatus: raw.httpStatus,
            details: raw.details,
          },
        };
      }
      return formatted;
    },
  });
  await server.start();

  app.use(
    "/graphql",
    express.json(),
    expressMiddleware(server, {
      context: async ({ req }): Promise<CyberOSContext> => {
        const request = extractRequestContext(req as express.Request);
        return { request, logger: logger.child({ traceparent: request.traceparent }) };
      },
    }),
  );

  app.listen(opts.port, () => {
    logger.info({ port: opts.port }, `subgraph ${opts.module} listening`);
  });
}

/**
 * Extract the RequestContext from headers. AUTH issues the JWT and the
 * `x-tenant-id` header (FR-AUTH-007); every other module trusts the gateway
 * (GraphOS Router) to forward both.
 *
 * For local dev where the router isn't running, default to a sandbox tenant.
 */
function extractRequestContext(req: express.Request): RequestContext {
  const traceparent = (req.header("traceparent") as string | undefined) ?? generateTraceparent();
  const tenantHeader = req.header("x-tenant-id");
  const tenantId = (tenantHeader ?? "tnt_local_dev") as TenantId;
  const memberId = (req.header("x-member-id") as MemberId | undefined) ?? null;
  const residency = (req.header("x-residency") as Residency | undefined) ?? "vn-hcm";
  const roles = parseList(req.header("x-roles"));
  const scopes = parseList(req.header("x-scopes"));
  return Object.freeze({
    traceparent,
    tenantId,
    memberId,
    residency,
    roles,
    scopes,
    startedAt: new Date(),
  });
}

function parseList(header: string | undefined): readonly string[] {
  if (!header) return Object.freeze([]);
  return Object.freeze(header.split(",").map((s) => s.trim()).filter(Boolean));
}
