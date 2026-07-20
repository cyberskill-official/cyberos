# Data deletion runbook (CyberOS)

This is the operational half of the promise made at
<https://cyberskill.world/en/cyberos/delete-account>. That page is the URL submitted to Google Play
and the App Store; both stores require a deletion path, and Play checks that the page exists and is reachable without signing in. A promise on that page that this runbook cannot execute is a compliance failure, so the two move together: **if the data model changes, change both.**

Deletion is currently manual and operator-run. Automating it (a self-serve request endpoint plus a scheduled purge job) is worth a task; until that lands, this is the path.

## Scope

Two request types, and they are not the same:

- **Subject deletion.** One person asks for their account and data to be removed. Their shared-channel messages are retained but de-attributed, because the content belongs to the organisation, not to them. This is the common case.
- **Workspace deletion.** An administrator closes the organisation's CyberOS account. Everything in the tenant goes.

## Preconditions

1. The request arrived **from the address the person signs in with**, or from an administrator of their workspace. That is the identity check - do not act on a forwarded request.
2. You have a fresh database backup, and you know how to restore it.
3. You are running against production knowingly. Everything below is inside an explicit transaction with a preflight count; read the counts before you type `COMMIT`.

## Subject deletion

### 1. Resolve the subject

```sql
SELECT id, tenant_id, handle, display_name, email, status
FROM subjects
WHERE lower(email) = lower('person@example.com');
```

Exactly one row. If zero, the person never had an account (reply and stop). If more than one, they belong to several tenants - confirm which workspace the request covers before going further.

Note the `id` (subject) and `tenant_id`. Everything below is parameterised on those two.

### 2. Create or find the tenant's tombstone subject

Shared-channel messages are re-pointed at this row rather than deleted, so the conversation stays readable for colleagues while ceasing to be attributable to the person.

```sql
INSERT INTO subjects (tenant_id, handle, display_name, kind, status)
VALUES ('<tenant_id>', '@deleted-user', 'Deleted user', 'system', 'active')
ON CONFLICT (tenant_id, handle) DO NOTHING;

SELECT id FROM subjects WHERE tenant_id = '<tenant_id>' AND handle = '@deleted-user';
```

### 3. Preflight: count what you are about to touch

```sql
BEGIN;

SELECT
  (SELECT count(*) FROM chat_devices        WHERE subject_id = '<subject_id>')          AS devices,
  (SELECT count(*) FROM chat_read_markers   WHERE subject_id = '<subject_id>')          AS read_markers,
  (SELECT count(*) FROM chat_reactions      WHERE subject_id = '<subject_id>')          AS reactions,
  (SELECT count(*) FROM chat_mentions       WHERE subject_id = '<subject_id>')          AS mentions,
  (SELECT count(*) FROM chat_channel_prefs  WHERE subject_id = '<subject_id>')          AS prefs,
  (SELECT count(*) FROM chat_channel_members WHERE subject_id = '<subject_id>')         AS memberships,
  (SELECT count(*) FROM chat_attachments    WHERE uploader_subject_id = '<subject_id>') AS attachments,
  (SELECT count(*) FROM chat_messages       WHERE sender_subject_id = '<subject_id>')   AS messages,
  (SELECT count(*) FROM chat_channels c
     JOIN chat_channel_members m ON m.channel_id = c.id
    WHERE c.kind = 'direct' AND m.subject_id = '<subject_id>')                          AS dm_channels;
```

`messages` is the number that will be de-attributed, not deleted. `dm_channels` is the number that will be destroyed outright, taking their messages and attachments with them.

### 4. Delete

```sql
-- Direct-message channels the person was in. ON DELETE CASCADE on chat_messages and
-- chat_attachments means this removes both sides of every DM, which is intended: a DM is private
-- between the two of them and is not an organisational record.
DELETE FROM chat_channels
WHERE kind = 'direct'
  AND id IN (SELECT channel_id FROM chat_channel_members WHERE subject_id = '<subject_id>');

-- Files they uploaded to shared channels. The bytes live in chat_attachments.data.
DELETE FROM chat_message_attachments
WHERE attachment_id IN (SELECT id FROM chat_attachments WHERE uploader_subject_id = '<subject_id>');
DELETE FROM chat_attachments WHERE uploader_subject_id = '<subject_id>';

-- Per-person signals.
DELETE FROM chat_devices       WHERE subject_id = '<subject_id>';
DELETE FROM chat_read_markers  WHERE subject_id = '<subject_id>';
DELETE FROM chat_reactions     WHERE subject_id = '<subject_id>';
DELETE FROM chat_mentions      WHERE subject_id = '<subject_id>';
DELETE FROM chat_channel_prefs WHERE subject_id = '<subject_id>';
DELETE FROM chat_channel_members WHERE subject_id = '<subject_id>';

-- De-attribute what remains in shared channels.
UPDATE chat_messages
SET sender_subject_id = '<tombstone_subject_id>'
WHERE sender_subject_id = '<subject_id>';

-- Identity.
DELETE FROM oidc_subject_link WHERE subject_id = '<subject_id>';
UPDATE oidc_login_history SET subject_id = NULL WHERE subject_id = '<subject_id>';
DELETE FROM subject_roles WHERE subject_id = '<subject_id>';
DELETE FROM mfa_factors   WHERE subject_id = '<subject_id>';
DELETE FROM subjects      WHERE id = '<subject_id>';
```

### 5. Verify, then commit

```sql
SELECT count(*) FROM subjects      WHERE id = '<subject_id>';              -- 0
SELECT count(*) FROM chat_devices  WHERE subject_id = '<subject_id>';      -- 0
SELECT count(*) FROM chat_messages WHERE sender_subject_id = '<subject_id>'; -- 0

COMMIT;
```

If any count is not zero, `ROLLBACK` and work out why before retrying. A partial deletion is worse than none: it looks done and is not.

### 6. Close the loop

Reply to the requester confirming the deletion and the date. Note the request and the date in the workspace's admin record. Play and the GDPR both give you 30 days; do it in days, not weeks.

## Workspace deletion

An administrator closing the organisation's account. Resolve the tenant, then delete in FK order.

```sql
SELECT id, slug, display_name FROM tenants WHERE slug = '<slug>';

BEGIN;
DELETE FROM chat_channels WHERE tenant_id = '<tenant_id>';   -- cascades messages + attachments
DELETE FROM chat_devices  WHERE tenant_id = '<tenant_id>';
DELETE FROM subjects      WHERE tenant_id = '<tenant_id>';
DELETE FROM tenants       WHERE id = '<tenant_id>';
COMMIT;
```

Confirm to the administrator in writing when it is done.

## What is NOT covered here

Server access and error logs. They may contain the person's subject id or email for up to 90 days, after which they rotate out. This is disclosed on the deletion page, and is permitted: security and abuse investigation is a legitimate reason to retain them for a bounded window. Do not extend that window without changing the published page first.

Rolling database backups age out within 30 days. A deletion is not re-applied to a backup; if a backup is ever restored, re-run this runbook for any request completed since that backup was taken.
