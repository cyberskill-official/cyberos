import type { CyberOSContext } from "@cyberos/subgraph-kit";

export const Query = {
  docHealth: (_root: unknown, _args: unknown, _ctx: CyberOSContext) => ({
    ok: true,
    module: "DOC",
    version: "0.1.0",
  }),
};
