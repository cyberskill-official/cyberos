-- Run once on first Postgres init (local dev)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgvector";
CREATE EXTENSION IF NOT EXISTS "pg_jsonschema";
-- PGroonga requires separate install; skip for local unless explicitly needed
-- CREATE EXTENSION IF NOT EXISTS "pgroonga";
