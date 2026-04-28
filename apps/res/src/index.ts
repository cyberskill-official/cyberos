/**
 * @cyberos/res — Resource Allocation subgraph entry point.
 *
 * Phase: P3 · Port: 4018 · GraphQL namespace: res
 *
 * Owns FRs: see docs/feature-requests/P3/RES/.
 *
 * Stays minimal on purpose: bootstrap goes through @cyberos/subgraph-kit,
 * which enforces the canonical Apollo + Express + tenant-context shape.
 */

import { startSubgraph } from "@cyberos/subgraph-kit";
import { typeDefs } from "./graphql/schema.ts";
import { resolvers } from "./graphql/resolvers/index.ts";

const PORT = Number(process.env.PORT_RES ?? 4018);

await startSubgraph({
  module: "RES",
  port: PORT,
  typeDefs,
  resolvers,
});
