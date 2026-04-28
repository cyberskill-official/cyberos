/**
 * @cyberos/cp — Client Portal subgraph entry point.
 *
 * Phase: P4 · Port: 4021 · GraphQL namespace: cp
 *
 * Owns FRs: see docs/feature-requests/P4/CP/.
 *
 * Stays minimal on purpose: bootstrap goes through @cyberos/subgraph-kit,
 * which enforces the canonical Apollo + Express + tenant-context shape.
 */

import { startSubgraph } from "@cyberos/subgraph-kit";
import { typeDefs } from "./graphql/schema.ts";
import { resolvers } from "./graphql/resolvers/index.ts";

const PORT = Number(process.env.PORT_CP ?? 4021);

await startSubgraph({
  module: "CP",
  port: PORT,
  typeDefs,
  resolvers,
});
