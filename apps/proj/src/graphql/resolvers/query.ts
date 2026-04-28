import type { CyberOSContext } from "@cyberos/subgraph-kit";

export const Query = {
  projHealth: (_root: unknown, _args: unknown, _ctx: CyberOSContext) => ({
    ok: true,
    module: "PROJ",
    version: "0.1.0",
  }),
};
