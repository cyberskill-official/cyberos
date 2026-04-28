/**
 * @cyberos/learn — Career Path & Learning subgraph entry point.
 *
 * Phase: P1 · Port: 4015 · GraphQL namespace: learn
 *
 * Owns FRs: see docs/feature-requests/P1/LEARN/.
 *
 * Stays minimal on purpose: bootstrap goes through @cyberos/subgraph-kit,
 * which enforces the canonical Apollo + Express + tenant-context shape.
 */

import { startSubgraph } from "@cyberos/subgraph-kit";
import { typeDefs } from "./graphql/schema.ts";
import { resolvers } from "./graphql/resolvers/index.ts";

const PORT = Number(process.env.PORT_LEARN ?? 4015);

await startSubgraph({
  module: "LEARN",
  port: PORT,
  typeDefs,
  resolvers,
});
