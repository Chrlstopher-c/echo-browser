#!/usr/bin/env bash
# Reveil anticipe : un onglet endormi survole se recharge sans devenir actif ; le clic suivant est immediat.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)" ECHO_CONTROL_NAME="test-$$" ECHO_DATA_DIR="$(mktemp -d)"
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
tabs = lambda: {t["id"]: t for t in call(op="tabs")["tabs"]}
a = call(op="open", url="https://example.org/")["id"]; time.sleep(4)
b = call(op="open", url="https://example.com/")["id"]; time.sleep(4)
assert call(op="sleep", id=a)["ok"]; time.sleep(1)
assert tabs()[a]["asleep"]
call(op="ui", request={"kind": "warmTab", "id": a}); time.sleep(4)
t = tabs()
assert not t[a]["asleep"], "onglet non reveille"
assert t[b]["active"] and not t[a]["active"], "l'onglet reveille d'avance ne doit pas devenir actif"
out = call(op="read", id=a)
assert out["ok"] and "Example Domain" in out["title"], out
t0 = time.time(); call(op="activate", id=a)
assert tabs()[a]["active"]
print(f"OK : reveil anticipe (activation {int((time.time()-t0)*1000)} ms)")
PY
