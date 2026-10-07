#!/usr/bin/env bash
# Le profil principal reste `Default` au demarrage, meme quand Chromium a note un conteneur comme « dernier profil
# utilise » (Local State) : le 08/10, ce detail a deconnecte tous les comptes du profil principal. Instance isolee.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)" ECHO_CONTROL_NAME="test-$$" ECHO_DATA_DIR="$(mktemp -d)"
export ECHO_DEVTOOLS_PORT=$((30000 + RANDOM % 20000))
trap './stop.sh >/dev/null 2>&1 || true' EXIT
run() {
uv run -q --with websocket-client python - "$1" <<'PY'
import json, os, sys, time, socket, urllib.request, websocket
s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = s.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
v = json.load(urllib.request.urlopen(f"http://127.0.0.1:{os.environ['ECHO_DEVTOOLS_PORT']}/json/version"))
ws = websocket.create_connection(v["webSocketDebuggerUrl"], suppress_origin=True)
def cmd(i, m, p={}):
    ws.send(json.dumps({"id": i, "method": m, "params": p}))
    while (r := json.loads(ws.recv())).get("id") != i: pass
    return r.get("result", r)
if sys.argv[1] == "avant":
    cmd(1, "Storage.setCookies", {"cookies": [{"name": "temoin", "value": "principal", "domain": "example.org", "path": "/",
                                                "expires": time.time() + 86400}]})
    call(op="ui", request={"kind": "newTab", "url": "https://example.com/", "container": "c1"}); time.sleep(4)
    print("conteneur cree, cookie temoin pose dans le profil principal")
else:
    names = [c["name"] for c in cmd(1, "Storage.getCookies")["cookies"]]
    assert "temoin" in names, f"le contexte par defaut n'est plus le profil principal (cookies : {names})"
    print("OK : profil principal garde malgre le dernier profil utilise")
PY
}
./start.sh release >/dev/null; sleep 12
run avant
./stop.sh >/dev/null; sleep 3
python3 - "$ECHO_DATA_DIR/profile/Local State" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
autres = [p for p in d["profile"]["info_cache"] if p != "Default"]
assert autres, "aucun conteneur dans Local State"
d["profile"]["last_used"] = autres[0]; d["profile"]["last_active_profiles"] = [autres[0]]
json.dump(d, open(sys.argv[1], "w"))
print("dernier profil utilise force a", autres[0])
PY
./start.sh release >/dev/null; sleep 12
run apres
