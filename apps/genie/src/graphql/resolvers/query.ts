import type { CyberOSContext } from "@cyberos/subgraph-kit";

export const Query = {
  genieHealth: (_root: unknown, _args: unknown, _ctx: CyberOSContext) => ({
    ok: true,
    module: "GENIE",
    version: "0.1.0",
  }),
};
