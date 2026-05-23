#!/bin/bash
# deploy-site.sh — push deskbrid site to Caddy NUC
# Run from repo root. Requires sshpass.

set -e

NUC="coemedia2@192.168.1.114"
NUC_PASS="NUC_PASSWORD"
TARGET="/home/coemedia2/docker/apps/caddy/sites-enabled/sites/html/deskbrid/index.html"
LOCAL="site/index.html"

echo "Deploying $LOCAL → $NUC:$TARGET"

sshpass -p "$NUC_PASS" scp -o StrictHostKeyChecking=no "$LOCAL" "$NUC:/home/coemedia2/deskbrid-index.html"
sshpass -p "$NUC_PASS" ssh -o StrictHostKeyChecking=no "$NUC" \
    "echo '$NUC_PASS' | sudo -S mv /home/coemedia2/deskbrid-index.html $TARGET"

echo "✓ Deployed — https://deskbrid.patchhive.dev"
