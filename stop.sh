#!/usr/bin/env bash
# Arrete Echo Browser par son identifiant enregistre, jamais par motif de nom.
set -euo pipefail
cd "$(dirname "$0")"

if [ ! -f logs/browser.pid ]; then
  echo "aucun identifiant enregistre — rien a arreter"
  exit 0
fi

PID="$(cat logs/browser.pid)"
if kill -0 "$PID" 2>/dev/null; then
  kill "$PID"
  for _ in $(seq 1 20); do
    kill -0 "$PID" 2>/dev/null || break
    sleep 0.25
  done
  kill -0 "$PID" 2>/dev/null && kill -9 "$PID" || true
  echo "echo-browser arrete (pid $PID)"
else
  echo "le processus $PID ne tourne plus"
fi
rm -f logs/browser.pid
