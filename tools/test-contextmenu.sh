#!/usr/bin/env bash
# Clic droit simule dans une vraie page : le menu s'ouvre, puis se referme. Instance isolee.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)"
export ECHO_CONTROL_NAME="test-$$"
trap 'cp "$ECHO_RUN_DIR/browser.log" /tmp/claude-1000/ctx.log 2>/dev/null; ./stop.sh >/dev/null 2>&1 || true' EXIT
ECHO_DATA_DIR="$(mktemp -d)" ./start.sh release >/dev/null
sleep 12
python3 - <<'PY'
import json, os, socket, time
sock = socket.socket(socket.AF_UNIX)
sock.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
stream = sock.makefile("rw")
def call(**r):
    stream.write(json.dumps(r) + "\n"); stream.flush()
    return json.loads(stream.readline())
assert call(op="open", url="https://example.org/")["ok"]
time.sleep(6)
assert not call(op="menu")["open"]
assert call(op="click", x=300, y=200, button="right")["ok"]
time.sleep(1.5)
assert call(op="menu")["open"], "le menu contextuel ne s'ouvre pas sur la page"
print("OK : clic droit")
PY
