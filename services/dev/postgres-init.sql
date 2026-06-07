-- CyberOS — Postgres initial bootstrap
-- Runs once on first container start (docker-entrypoint-initdb.d).
-- Enables every extension every service depends on.

-- The POSTGRES_DB env var creates the legacy `cyberos` database before this
-- file runs. Local module testing uses one database per service because each
-- Rust service owns an independent sqlx migration sequence starting at 0001;
-- sharing one DB would collide in `_sqlx_migrations`.
CREATE DATABASE cyberos_auth;
CREATE DATABASE cyberos_memory;
CREATE DATABASE cyberos_proj;

\connect cyberos

CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS vector;        -- pgvector for embeddings
CREATE EXTENSION IF NOT EXISTS age;           -- Apache AGE for graph queries
LOAD 'age';
SET search_path = ag_catalog, "$user", public;
GRANT ALL PRIVILEGES ON DATABASE cyberos TO cyberos;

\connect cyberos_auth

-- Required for the AUTH module (gen_random_uuid, pgcrypto).
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
GRANT ALL PRIVILEGES ON DATABASE cyberos_auth TO cyberos;

\connect cyberos_memory

-- Required for the memory Layer-2 service (pgvector + Apache AGE).
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS age;
LOAD 'age';
SET search_path = ag_catalog, "$user", public;
GRANT ALL PRIVILEGES ON DATABASE cyberos_memory TO cyberos;

\connect cyberos_proj

-- Required for the PROJ schema tests and future HTTP surface.
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
GRANT ALL PRIVILEGES ON DATABASE cyberos_proj TO cyberos;

-- A single dev role with broad grants. Production deploys use per-service
-- least-privilege roles (see services/auth/migrations/0004_rls_roles.sql).
