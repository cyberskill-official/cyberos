/**
 * @cyberos/chat — Internal Chat subgraph entry point.
 *
 * Phase: P0 · Port: 4005 · GraphQL namespace: chat
 *
 * Owns FRs: see docs/feature-requests/P0/CHAT/.
 *
 * Stays minimal on purpose: bootstrap goes through @cyberos/subgraph-kit,
 * which enforces the canonical Apollo + Express + tenant-context shape.
 */

import { startSubgraph } from "@cyberos/subgraph-kit";
import { typeDefs } from "./graphql/schema.ts";
import { resolvers } from "./graphql/resolvers/index.ts";

const PORT = Number(process.env.PORT_CHAT ?? 4005);

await startSubgraph({
  module: "CHAT",
  port: PORT,
  typeDefs,
  resolvers,
});
