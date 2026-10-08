#!/usr/bin/env bash
# Prelude commun des tests : instance isolee (dossiers, socket, port DevTools propres) + serveur de pages local.
# Usage : `source tools/banc.sh` depuis la racine ; definit W (dossier jetable), PORT (pages de $W), lance Echo.
# `BANC_WELCOME=1` garde l'accueil du premier lancement ; `BANC_NO_START=1` laisse le test lancer Echo lui-meme.
set -euo pipefail
W="$(mktemp -d)"; PORT=$((29000 + RANDOM % 500))
python3 tools/serveur-test.py "$PORT" "$W" >/dev/null 2>&1 & SERVEUR=$!
export ECHO_RUN_DIR="$W/run" ECHO_DATA_DIR="$W/data" ECHO_CONTROL_NAME="banc-$$" W PORT
[ "${BANC_WELCOME:-0}" = 1 ] || export ECHO_NO_WELCOME=1
export ECHO_DEVTOOLS_PORT=$((30000 + RANDOM % 9000))
export PYTHONPATH="$PWD/tools${PYTHONPATH:+:$PYTHONPATH}"
mkdir -p "$W/run"
trap './stop.sh >/dev/null 2>&1 || true; kill $SERVEUR 2>/dev/null || true; rm -rf "$W"' EXIT
if [ "${BANC_NO_START:-0}" != 1 ]; then ./start.sh release >/dev/null; sleep 8; fi
