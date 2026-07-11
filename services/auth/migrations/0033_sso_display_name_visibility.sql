-- FR-AUTH-111 — SSO display names.
--
-- There is deliberately NO BACKFILL of display_name here (§1 #7), and that is the whole point of this file.
--
-- We can see exactly WHICH rows are wrong: display_name is byte-equal to the person's email, which is what
-- the bug wrote and what no human would ever choose. What we cannot see is what those rows should SAY. The
-- ID token that carried the person's name was verified, read for `sub` and `email`, and discarded; the name
-- was never persisted anywhere. The only honest sources for it are the IdP (which means a login) or a human
-- (which means typing).
--
-- Deriving a name from the email local-part looks like a fix and is a guess. It would render `van-anh.vu@`
-- as `Van Anh Vu` where the person is called `Vũ Vân Anh` — wrong in exactly the way that matters most to a
-- Vietnamese company, and wrong in a way that then LOOKS deliberate, so nobody reports it. A visible bug is
-- better than a confident error. Every affected subject self-heals on their next sign-in via
-- display_name::heal, with no migration and no administrator action.
--
-- What this migration DOES is make the damage countable, so the repair can be watched draining to zero
-- rather than assumed. A self-healing fix is invisible: without this view there is no way to know whether it
-- worked, or how many people are still affected.

CREATE OR REPLACE VIEW subjects_display_name_unset AS
    SELECT id, tenant_id, handle, email, created_at
      FROM subjects
     WHERE kind = 'human'
       AND email IS NOT NULL
       AND (display_name IS NULL OR display_name = '' OR display_name = email);

COMMENT ON VIEW subjects_display_name_unset IS
    'FR-AUTH-111: humans still wearing their email address as a display name. Drains to zero as people sign in.';

GRANT SELECT ON subjects_display_name_unset TO cyberos_ro;
