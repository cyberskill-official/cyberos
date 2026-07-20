# Backups - what is covered, what you must run, how to restore

Written 2026-07-02 (module review follow-up). The P0 stack keeps state in exactly three places:

| State | Where it lives | Backup story |
|---|---|---|
| All databases (auth, chat, memory/audit, eval) | Supabase Postgres | Supabase's own backups. Verify the project tier's retention in the Supabase dashboard; enable PITR if the plan allows. Nothing to run on the VPS. |
| Chat attachment bytes | Docker volume `chat-attachments` on the VPS (CHAT_ATTACHMENT_STORE=fs since the richer-messages cluster) | NOT covered by anything by default. Run `deploy/vps/backup-attachments.sh` nightly via cron (below). |
| TLS certs / Caddy state | Docker volumes `caddy-data`/`caddy-config` | Re-issuable automatically by Caddy on a fresh host; no backup needed. |

Everything else on the VPS (images, containers, the git checkout) is reproducible from GHCR + GitHub.

## Set up the attachment backup (one-time, on the VPS)

```
mkdir -p ~/backups
crontab -e
# add:
17 3 * * * bash $HOME/cyberos/deploy/vps/backup-attachments.sh >> $HOME/backups/attachments.log 2>&1
```

Defaults: writes `~/backups/chat-attachments/chat-attachments-<stamp>.tar.gz`, keeps 14 days (`CYBEROS_BACKUP_KEEP_DAYS` to change). The tar streams from a read-only mount; chat stays up.

Off-host copies: the VPS is a single point of failure, so periodically pull a backup off the box, e.g. from any machine with SSH access: `scp vps:~/backups/chat-attachments/<latest>.tar.gz ~/cyberos-backups/`. (Object-storage sync can replace this when an S3 bucket exists; the storage seam in services/chat/src/storage.rs is also ready to move the primary bytes to S3 at that point.)

## Restore attachments

```
cd ~/cyberos/deploy/vps
docker compose --env-file .env.p0 -f docker-compose.p0.images.yml stop chat
docker run --rm -v cyberos-p0_chat-attachments:/data -v ~/backups/chat-attachments:/backup:ro \
  alpine sh -c "rm -rf /data/* && tar -xzf /backup/<CHOSEN>.tar.gz -C /data"
docker compose --env-file .env.p0 -f docker-compose.p0.images.yml up -d chat
```

Rows in `chat_attachments` whose bytes are missing after a partial restore answer 404 on download (the service treats a missing payload as not-found, never a crash).

## Disaster recovery (whole VPS lost)

1. New VPS, Docker + Caddy prerequisites per docs/deploy/p0-google-chat-runbook.md.
2. Clone the repo, restore `.env.p0` from the operator's secret store (it is never in git).
3. `bash deploy/vps/deploy.sh` (pulls images, applies migrations against Supabase, starts the stack).
4. Restore the newest attachment backup (above). DNS to the new IP; Caddy re-issues certs. RTO is dominated by DNS + image pulls; data loss = attachments since the last nightly.
