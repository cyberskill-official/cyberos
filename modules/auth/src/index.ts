/**
 * @cyberos/auth — Federation v2 Subgraph
 *
 * Responsibilities:
 *   - JWT issuance (RS256) + OIDC discovery endpoint
 *   - Member / Session / APIKey data models
 *   - Tenant isolation via 3-layer model (JWT → middleware → RLS)
 *   - MCP tools: auth.whoami, auth.issue_token, auth.revoke_token
 *
 * Entry point — starts Express + Apollo subgraph.
 * See SRS §4.1 for full FR/NFR list.
 */

import 'newrelic'; // must be first import
import express from 'express';
import { ApolloServer } from '@apollo/server';
import { expressMiddleware } from '@apollo/server/express4';
import { buildSubgraphSchema } from '@apollo/subgraph';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { typeDefs as scalarsTypeDefs } from 'graphql-scalars';
import { resolvers } from './resolvers/index.js';
import { tenantMiddleware } from './middleware/tenant.js';
import { createContext } from './context.js';
import { startMcpServer } from './mcp/server.js';
import logger from './logger.js';

const PORT = parseInt(process.env.AUTH_PORT ?? '4001', 10);

const sdl = readFileSync(resolve(import.meta.dirname, '../schema.graphql'), 'utf-8');

const server = new ApolloServer({
  schema: buildSubgraphSchema({ typeDefs: sdl, resolvers }),
  plugins: [
    // newrelic apollo plugin registered last per DEC-014
    (await import('@newrelic/apollo-server-plugin')).default,
  ],
});

await server.start();

const app = express();
app.use(express.json());
app.use(tenantMiddleware);
app.use('/graphql', expressMiddleware(server, { context: createContext }));

app.get('/health', (_req, res) => res.json({ ok: true, module: 'auth' }));

app.listen(PORT, () => {
  logger.info({ port: PORT }, 'auth module started');
});

// MCP server on separate port (shared via API Gateway in prod)
await startMcpServer();
