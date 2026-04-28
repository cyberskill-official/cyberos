/**
 * @cyberos/kb — Knowledge Base subgraph entry point.
 *
 * Phase: P1 · Port: 4011 · GraphQL namespace: kb
 *
 * Owns FRs: see docs/feature-requests/P1/KB/.
 *
 * Stays minimal on purpose: bootstrap goes through @cyberos/subgraph-kit,
 * which enforces the canonical Apollo + Express + tenant-context shape.
 */

import { startSubgraph } from "@cyberos/subgraph-kit";
import { typeDefs } from "./graphql/schema.ts";
import { resolvers } from "./graphql/resolvers/index.ts";

const PORT = Number(process.env.PORT_KB ?? 4011);

await startSubgraph({
  module: "KB",
  port: PORT,
  typeDefs,
  resolvers,
});
