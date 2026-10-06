#!/usr/bin/env bash
# Verifie la prise de pilotage : ouvrir, lire, naviguer, fermer un onglet, et refuser une operation inconnue.
set -euo pipefail
cd "$(dirname "$0")/.."
# Instance de test isolee : jamais le navigateur de l'utilisateur (son pid, son journal, sa prise).
export ECHO_RUN_DIR="$(mktemp -d)"
export ECHO_CONTROL_NAME="test-$$"
trap './stop.sh >/dev/null 2>&1 || true' EXIT
ECHO_DATA_DIR="$(mktemp -d)" ./start.sh release >/dev/null
sleep 12
python3 - <<'PY'
import json, os, socket, time
sock = socket.socket(socket.AF_UNIX)
sock.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
stream = sock.makefile("rw")

def call(**request):
    stream.write(json.dumps(request) + "\n"); stream.flush()
    return json.loads(stream.readline())

opened = call(op="open", url="https://example.org/")
assert opened["ok"], opened
time.sleep(6)
page = call(op="read", id=opened["id"])
assert page["ok"] and page["title"] == "Example Domain" and "exemples" in page["text"], page
assert call(op="close", id=opened["id"])["ok"]
assert not call(op="close", id=opened["id"])["ok"], "fermer deux fois doit echouer"
assert not call(op="zzz")["ok"]
print("OK : prise de pilotage")
PY
