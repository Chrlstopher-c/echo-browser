#!/usr/bin/env bash
# Lance Echo Browser. Le binaire a besoin de trouver libcef.so : c'est tout l'objet de ce script.
set -euo pipefail
cd "$(dirname "$0")"

export CEF_PATH="${CEF_PATH:-$HOME/.local/share/cef}"
export ECHO_DATA_DIR="${ECHO_DATA_DIR:-$HOME/.local/share/echo-browser}"
export LD_LIBRARY_PATH="${LD_LIBRARY_PATH:-}:$CEF_PATH"

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
setsid "$BINARY" >> logs/browser.log 2>&1 < /dev/null &
echo $! > logs/browser.pid
echo "echo-browser lance (pid $(cat logs/browser.pid)) — journal : logs/browser.log"
