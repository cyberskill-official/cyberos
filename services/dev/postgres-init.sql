-- CyberOS — Postgres initial bootstrap
-- Runs once on first container start (docker-entrypoint-initdb.d).
-- Enables every extension every service depends on.

-- Required for the AUTH module (gen_random_uuid, pgcrypto).
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Required for the BRAIN module (FR-BRAIN-101 Layer-2 ingest).
CREATE EXTENSION IF NOT EXISTS vector;        -- pgvector for embeddings
-- Graph edges live in the relational l2_edge table (traversed via recursive CTEs); no graph extension.

-- A single dev role with broad grants. Production deploys use per-service
-- least-privilege roles (see services/auth/migrations/0004_rls_roles.sql).
GRANT ALL PRIVILEGES ON DATABASE cyberos TO cyberos;
