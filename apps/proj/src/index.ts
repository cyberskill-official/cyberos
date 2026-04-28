/**
 * @cyberos/proj — Projects & Tasks subgraph entry point.
 *
 * Phase: P1 · Port: 4008 · GraphQL namespace: proj
 *
 * Owns FRs: see docs/feature-requests/P1/PROJ/.
 *
 * Stays minimal on purpose: bootstrap goes through @cyberos/subgraph-kit,
 * which enforces the canonical Apollo + Express + tenant-context shape.
 */

import { startSubgraph } from "@cyberos/subgraph-kit";
import { typeDefs } from "./graphql/schema.ts";
import { resolvers } from "./graphql/resolvers/index.ts";

const PORT = Number(process.env.PORT_PROJ ?? 4008);

await startSubgraph({
  module: "PROJ",
  port: PORT,
  typeDefs,
  resolvers,
});
