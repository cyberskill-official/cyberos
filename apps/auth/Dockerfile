# Multi-stage Dockerfile for a CyberOS module subgraph.
# Override at build time with --build-arg PORT=4001.

FROM node:20-bookworm-slim AS base
ENV PNPM_HOME=/pnpm
ENV PATH=$PNPM_HOME:$PATH
RUN corepack enable

FROM base AS deps
WORKDIR /repo
COPY pnpm-workspace.yaml pnpm-lock.yaml package.json ./
COPY apps apps
COPY packages packages
RUN pnpm install --frozen-lockfile --filter "@cyberos/auth..."

FROM deps AS build
WORKDIR /repo
RUN pnpm --filter "@cyberos/auth..." build

FROM base AS runtime
WORKDIR /app
ENV NODE_ENV=production
COPY --from=build /repo/apps/auth/dist ./dist
COPY --from=build /repo/apps/auth/package.json ./package.json
COPY --from=build /repo/node_modules ./node_modules
EXPOSE 4001
CMD ["node", "--enable-source-maps", "dist/index.js"]
