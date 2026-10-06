#!/usr/bin/env bash
# Sites « jamais endormis » : l'onglet listé reste éveillé, un autre onglet inactif s'endort.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)" ECHO_CONTROL_NAME="test-$$" ECHO_DATA_DIR="$(mktemp -d)" ECHO_SLEEP_AFTER_S=20
trap './stop.sh >/dev/null 2>&1 || true' EXIT
./start.sh release >/dev/null; sleep 12
python3 - <<'PY'
import json, os, socket, time
sock = socket.socket(socket.AF_UNIX)
sock.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
stream = sock.makefile("rw")
def call(**r):
    stream.write(json.dumps(r) + "\n"); stream.flush()
    return json.loads(stream.readline())
call(op="ui", request={"kind": "updateSetting", "key": "tabs.neverSleep", "value": {"type": "text", "value": "example.org"}})
call(op="open", url="https://example.org/"); time.sleep(4)
call(op="open", url="https://example.com/"); time.sleep(4)
call(op="open", url="https://www.iana.org/"); time.sleep(50)
by = {t["url"].split("/")[2]: t for t in call(op="tabs")["tabs"] if t["url"].startswith("http")}
assert not by["example.org"]["asleep"], "site liste endormi"
assert by["example.com"]["asleep"], "site non liste reste eveille"
print("OK : jamais endormi")
PY
