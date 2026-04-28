/**
 * @cyberos/doc — Document Signing subgraph entry point.
 *
 * Phase: P4 · Port: 4020 · GraphQL namespace: doc
 *
 * Owns FRs: see docs/feature-requests/P4/DOC/.
 *
 * Stays minimal on purpose: bootstrap goes through @cyberos/subgraph-kit,
 * which enforces the canonical Apollo + Express + tenant-context shape.
 */

import { startSubgraph } from "@cyberos/subgraph-kit";
import { typeDefs } from "./graphql/schema.ts";
import { resolvers } from "./graphql/resolvers/index.ts";

const PORT = Number(process.env.PORT_DOC ?? 4020);

await startSubgraph({
  module: "DOC",
  port: PORT,
  typeDefs,
  resolvers,
});
