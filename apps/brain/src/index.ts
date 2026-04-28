/**
 * @cyberos/brain — Universal Knowledge Layer subgraph entry point.
 *
 * Phase: P0 · Port: 4006 · GraphQL namespace: brain
 *
 * Owns FRs: see docs/feature-requests/P0/BRAIN/.
 *
 * Stays minimal on purpose: bootstrap goes through @cyberos/subgraph-kit,
 * which enforces the canonical Apollo + Express + tenant-context shape.
 */

import { startSubgraph } from "@cyberos/subgraph-kit";
import { typeDefs } from "./graphql/schema.ts";
import { resolvers } from "./graphql/resolvers/index.ts";

const PORT = Number(process.env.PORT_BRAIN ?? 4006);

await startSubgraph({
  module: "BRAIN",
  port: PORT,
  typeDefs,
  resolvers,
});
