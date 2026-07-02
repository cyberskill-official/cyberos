#!/usr/bin/env bash
# Nightly backup of the chat attachment payloads (the `chat-attachments` Docker volume).
#
# Why this exists: message/channel data lives in Supabase (which has its own PITR story), but attachment
# BYTES live only in this volume on the VPS since the richer-messages cluster (CHAT_ATTACHMENT_STORE=fs).
# Losing the VPS without this backup means losing every shared file. See docs/deploy/backups.md.
#
# Install on the VPS (as the deploy user):
#   crontab -e
#   17 3 * * * bash $HOME/cyberos/deploy/vps/backup-attachments.sh >> $HOME/backups/attachments.log 2>&1
#
# Restore: stop chat, then untar into the volume the same way in reverse (see docs/deploy/backups.md).
set -euo pipefail

BACKUP_DIR="${CYBEROS_BACKUP_DIR:-$HOME/backups/chat-attachments}"
KEEP_DAYS="${CYBEROS_BACKUP_KEEP_DAYS:-14}"
# The volume name is namespaced by the compose project (see docker-compose.p0.images.yml `name:`).
PROJECT="${COMPOSE_PROJECT_NAME:-cyberos-p0}"
VOLUME="${PROJECT}_chat-attachments"

mkdir -p "$BACKUP_DIR"
STAMP="$(date +%Y%m%d-%H%M%S)"
OUT="$BACKUP_DIR/chat-attachments-$STAMP.tar.gz"

# Read-only mount of the volume into a throwaway container; tar streams out to the host.
docker run --rm -v "$VOLUME":/data:ro alpine tar -czf - -C /data . > "$OUT"
echo "backup written: $OUT ($(du -h "$OUT" | cut -f1))"

# Rotate: keep KEEP_DAYS days.
find "$BACKUP_DIR" -name 'chat-attachments-*.tar.gz' -mtime +"$KEEP_DAYS" -delete
echo "rotation done (kept <= $KEEP_DAYS days)"
