/**
 * @cyberos/email — Email Client subgraph entry point.
 *
 * Phase: P1 · Port: 4013 · GraphQL namespace: email
 *
 * Owns FRs: see docs/feature-requests/P1/EMAIL/.
 *
 * Stays minimal on purpose: bootstrap goes through @cyberos/subgraph-kit,
 * which enforces the canonical Apollo + Express + tenant-context shape.
 */

import { startSubgraph } from "@cyberos/subgraph-kit";
import { typeDefs } from "./graphql/schema.ts";
import { resolvers } from "./graphql/resolvers/index.ts";

const PORT = Number(process.env.PORT_EMAIL ?? 4013);

await startSubgraph({
  module: "EMAIL",
  port: PORT,
  typeDefs,
  resolvers,
});
