/**
 * @cyberos/ai — AI Gateway subgraph entry point.
 *
 * Phase: P0 · Port: 4002 · GraphQL namespace: ai
 *
 * Owns FRs: see docs/feature-requests/P0/AI/.
 *
 * Stays minimal on purpose: bootstrap goes through @cyberos/subgraph-kit,
 * which enforces the canonical Apollo + Express + tenant-context shape.
 */

import { startSubgraph } from "@cyberos/subgraph-kit";
import { typeDefs } from "./graphql/schema.ts";
import { resolvers } from "./graphql/resolvers/index.ts";

const PORT = Number(process.env.PORT_AI ?? 4002);

await startSubgraph({
  module: "AI",
  port: PORT,
  typeDefs,
  resolvers,
});
