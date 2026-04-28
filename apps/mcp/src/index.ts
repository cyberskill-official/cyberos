/**
 * @cyberos/mcp — MCP Server subgraph entry point.
 *
 * Phase: P0 · Port: 4003 · GraphQL namespace: mcp
 *
 * Owns FRs: see docs/feature-requests/P0/MCP/.
 *
 * Stays minimal on purpose: bootstrap goes through @cyberos/subgraph-kit,
 * which enforces the canonical Apollo + Express + tenant-context shape.
 */

import { startSubgraph } from "@cyberos/subgraph-kit";
import { typeDefs } from "./graphql/schema.ts";
import { resolvers } from "./graphql/resolvers/index.ts";

const PORT = Number(process.env.PORT_MCP ?? 4003);

await startSubgraph({
  module: "MCP",
  port: PORT,
  typeDefs,
  resolvers,
});
