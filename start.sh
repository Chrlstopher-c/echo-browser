#!/usr/bin/env bash
# Lance Echo Browser. Le binaire a besoin de trouver libcef.so : c'est tout l'objet de ce script.
set -euo pipefail
cd "$(dirname "$0")"

export CEF_PATH="${CEF_PATH:-$HOME/.local/share/cef}"
export ECHO_DATA_DIR="${ECHO_DATA_DIR:-$HOME/.local/share/echo-browser}"
export LD_LIBRARY_PATH="${LD_LIBRARY_PATH:-}:$CEF_PATH"
export ECHO_UI_DIR="${ECHO_UI_DIR:-$PWD/ui/dist}"

if [ ! -f "$CEF_PATH/libcef.so" ]; then
  echo "libcef.so introuvable dans $CEF_PATH — lancer d'abord : bash tools/fetch-cef.sh" >&2
  exit 1
fi

PROFILE="${1:-release}"
BINARY="target/$PROFILE/echo-browser"
if [ ! -x "$BINARY" ]; then
  echo "binaire absent : cargo build -p echo-shell --$PROFILE" >&2
  exit 1
fi

mkdir -p logs
: > logs/browser.log
# `setsid` se dedouble quand l'appelant est deja chef de groupe : $! designerait alors
# le lanceur, pas le navigateur, et stop.sh tuerait un processus deja mort. L'enfant
# inscrit donc lui-meme son identifiant, juste avant de se remplacer par le binaire.
setsid bash -c 'echo $$ > logs/browser.pid; exec "$0"' "$BINARY" >> logs/browser.log 2>&1 < /dev/null &
for _ in $(seq 1 40); do
  [ -s logs/browser.pid ] && break
  sleep 0.05
done
echo "echo-browser lance (pid $(cat logs/browser.pid)) — journal : logs/browser.log"
