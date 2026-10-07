#!/usr/bin/env bash
# Profils : chacun ses onglets ; revenir sur un profil retrouve son dernier onglet ; les comptes (cookies) sont
# separes entre le profil par defaut et un autre ; le profil des onglets survit a la relance. Serveur local de test.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)" ECHO_CONTROL_NAME="test-$$" ECHO_DATA_DIR="$(mktemp -d)"
PORT=$((20000 + RANDOM % 20000))
python3 - "$PORT" <<'PYS' &
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
class H(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200); self.send_header("Content-Type", "text/html")
        if self.path.startswith("/set"): self.send_header("Set-Cookie", "compte=perso; Max-Age=86400; Path=/")
        self.end_headers(); self.wfile.write(("<title>c</title><p>cookies=[%s]</p>" % self.headers.get("Cookie", "")).encode())
    def log_message(self, *a): pass
ThreadingHTTPServer(("127.0.0.1", int(sys.argv[1])), H).serve_forever()
PYS
SERVER=$!
trap './stop.sh >/dev/null 2>&1 || true; kill $SERVER 2>/dev/null || true' EXIT
cat > "$ECHO_RUN_DIR/h.py" <<'PYH'
import json, os, socket, time
s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = s.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
def tabs(): return call(op="tabs")["tabs"]
def active(): return [t for t in tabs() if t["active"]][0]
def space(i): call(op="ui", request={"kind": "setSpace", "id": i}); time.sleep(3)
def page(path):
    call(op="open", url=f"http://127.0.0.1:{os.environ['TPORT']}{path}")
    for _ in range(30):
        time.sleep(0.5)
        t = call(op="read", id=active()["id"])
        if "cookies=" in t.get("text", ""): return t["text"]
    raise AssertionError("page non chargee")
PYH
export TPORT=$PORT
./start.sh release >/dev/null; sleep 14
python3 - <<PY
exec(open("$ECHO_RUN_DIR/h.py").read())
space("graphite")
page("/set"); assert "compte=perso" in page("/"), "cookie non pose dans le profil par defaut"
perso = active()["id"]
space("sable")
assert active()["space"] == "sable" and active()["id"] != perso, f"le profil sable doit avoir son propre onglet : {active()}"
assert "compte=perso" not in page("/"), "les comptes du profil par defaut fuient dans le profil sable"
space("graphite")
assert active()["space"] == "graphite", f"retour au profil par defaut : {active()}"
print("profils ok")
PY
./stop.sh >/dev/null; sleep 3; ./start.sh release >/dev/null; sleep 14
python3 - <<PY
exec(open("$ECHO_RUN_DIR/h.py").read())
spaces = sorted({t["space"] for t in tabs()})
assert spaces == ["graphite", "sable"], f"profils perdus a la relance : {spaces}"
print("OK : profils (onglets, comptes, relance)")
PY
