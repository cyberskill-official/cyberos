/**
 * @cyberos/time — Time Tracking subgraph entry point.
 *
 * Phase: P1 · Port: 4009 · GraphQL namespace: time
 *
 * Owns FRs: see docs/feature-requests/P1/TIME/.
 *
 * Stays minimal on purpose: bootstrap goes through @cyberos/subgraph-kit,
 * which enforces the canonical Apollo + Express + tenant-context shape.
 */

import { startSubgraph } from "@cyberos/subgraph-kit";
import { typeDefs } from "./graphql/schema.ts";
import { resolvers } from "./graphql/resolvers/index.ts";

const PORT = Number(process.env.PORT_TIME ?? 4009);

await startSubgraph({
  module: "TIME",
  port: PORT,
  typeDefs,
  resolvers,
});
