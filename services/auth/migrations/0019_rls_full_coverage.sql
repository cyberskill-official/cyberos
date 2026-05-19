-- [ignoring loop detection]
-- Migration 0019: rls_full_coverage (No-op)
--
-- This migration originally tried to retroactively enable RLS and grant privileges
-- on tables created in migrations 0007..0018.
-- However, those migrations already successfully created their respective RLS policies
-- and granted permissions at creation time.
-- To prevent "policy already exists" errors during db migration, this file is preserved
-- as a no-op.
SELECT 1;
