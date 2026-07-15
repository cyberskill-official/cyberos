-- TASK-BRAIN-101 — Layer-2 ingest pipeline · core schema
--
-- Layer 2 is a READ scale-out of Layer 1 (the append-only chain in BRAIN
-- Personal). Per DEC-070, Layer 1 is the source of truth; Layer 2 holds
-- materialized projections (pgvector for similarity; graph edges in the relational
-- l2_edge table) with chain_anchor verification on read.
--
-- This migration creates the projection tables. The pgvector extension is loaded
-- by services/dev/postgres-init.sql at container init.

-- ---------------------------------------------------------------------------
-- l2_memory — projection of every memory row from Layer 1.
-- ---------------------------------------------------------------------------
CREATE TABLE l2_memory (
    -- Identity
    tenant_id           UUID NOT NULL,                              -- per DEC-073 (per-tenant cursor)
    seq                 BIGINT NOT NULL,                            -- monotonic seq from L1
    path                TEXT NOT NULL,                              -- canonical store path
    -- Content
    body                TEXT NOT NULL,                              -- markdown body
    frontmatter         JSONB NOT NULL DEFAULT '{}'::jsonb,
    -- Layer-1 cross-reference
    chain_anchor        BYTEA NOT NULL,                             -- SHA-256(prev_hash || body) — per §1 #4
    chain_anchor_hex    TEXT GENERATED ALWAYS AS (encode(chain_anchor, 'hex')) STORED,
    -- Embedding for similarity
    embedding           VECTOR(1024),                               -- bge-m3 dims; nullable until first embed
    -- Audit
    ingested_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Identity is the (tenant, seq, path) tuple; the same path may be
    -- written multiple times under different seqs (versions).
    PRIMARY KEY (tenant_id, seq, path)
);

CREATE INDEX l2_memory_chain_anchor_idx ON l2_memory (chain_anchor_hex);
CREATE INDEX l2_memory_path_idx          ON l2_memory (tenant_id, path);
CREATE INDEX l2_memory_ingested_at_idx   ON l2_memory (ingested_at DESC);

-- Approximate nearest-neighbour index for similarity queries (TASK-BRAIN-108
-- search-api). HNSW preferred over IVFFlat for low-recall-budget reads.
-- Created when first row lands — keep migration cheap.
-- CREATE INDEX l2_memory_embedding_hnsw_idx
--     ON l2_memory USING hnsw (embedding vector_cosine_ops)
--     WITH (m = 16, ef_construction = 64);

-- ---------------------------------------------------------------------------
-- l2_entity — extracted entities (people, orgs, projects, decisions).
-- Populated alongside l2_memory by services/brain/src/layer2/entity_extract.rs.
-- ---------------------------------------------------------------------------
CREATE TABLE l2_entity (
    tenant_id   UUID NOT NULL,
    entity_id   UUID NOT NULL DEFAULT gen_random_uuid(),
    kind        TEXT NOT NULL,        -- 'person' | 'org' | 'project' | 'decision' | 'doc'
    name        TEXT NOT NULL,
    source_seq  BIGINT NOT NULL,      -- seq of the L1 row that introduced it
    source_path TEXT NOT NULL,
    embedding   VECTOR(1024),
    properties  JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, entity_id)
);

CREATE INDEX l2_entity_kind_idx ON l2_entity (tenant_id, kind);
CREATE INDEX l2_entity_name_idx ON l2_entity USING GIN (to_tsvector('simple', name));

-- ---------------------------------------------------------------------------
-- l2_edge — graph edges between entities (cites | implements | supersedes …).
-- Traversed with recursive CTEs (Phase-3 typed link extraction); no graph extension needed.
-- ---------------------------------------------------------------------------
CREATE TABLE l2_edge (
    tenant_id   UUID NOT NULL,
    edge_id     UUID NOT NULL DEFAULT gen_random_uuid(),
    src_entity  UUID NOT NULL,
    dst_entity  UUID NOT NULL,
    kind        TEXT NOT NULL,        -- 'cites' | 'implements' | 'supersedes' | 'refines'
    properties  JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, edge_id)
);

CREATE INDEX l2_edge_src_idx ON l2_edge (tenant_id, src_entity);
CREATE INDEX l2_edge_dst_idx ON l2_edge (tenant_id, dst_entity);
