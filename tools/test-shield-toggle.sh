#!/usr/bin/env bash
# Bouclier : couper la protection d'un site (ou globalement) recharge la page et laisse passer ses publicites,
# cadres externes compris ; la coupure globale survit a la relance. Serveur local de test lance ici.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)" ECHO_CONTROL_NAME="test-$$" ECHO_DATA_DIR="$(mktemp -d)"
PORT=$((20000 + RANDOM % 20000))
python3 - "$PORT" <<'PYS' &
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
PAGE = b"""<title>pub</title><p id=p>attente</p><script>
let n = 0; const s = document.createElement('script')
s.src = 'https://securepubads.g.doubleclick.net/tag/js/gpt.js'
s.onload = () => p.textContent = 'pub=chargee'; s.onerror = () => p.textContent = 'pub=bloquee'
document.head.append(s)</script>"""
class H(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200); self.send_header("Content-Type", "text/html"); self.end_headers(); self.wfile.write(PAGE)
    def log_message(self, *a): pass
ThreadingHTTPServer(("127.0.0.1", int(sys.argv[1])), H).serve_forever()
PYS
SERVER=$!
trap 'cp $ECHO_RUN_DIR/browser.log /tmp/claude-1000/sh.log; ./stop.sh >/dev/null 2>&1 || true; kill $SERVER 2>/dev/null || true' EXIT
export ECHO_LOG=debug
cat > "$ECHO_RUN_DIR/h.py" <<'PYH'
import json, os, socket, time
sock = socket.socket(socket.AF_UNIX)
sock.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = sock.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
def state(i):
    for _ in range(30):
        t = call(op="read", id=i)["text"]
        if "pub=" in t: return t.strip()
        time.sleep(0.3)
    return t
PYH
export TPORT=$PORT
./start.sh release >/dev/null; sleep 16
python3 - <<PY
exec(open("$ECHO_RUN_DIR/h.py").read())
i = call(op="open", url=f"http://127.0.0.1:{os.environ['TPORT']}/")["id"]; time.sleep(3)
assert state(i) == "pub=bloquee", state(i)
call(op="ui", request={"kind": "toggleShieldForSite", "id": i}); time.sleep(3)
assert state(i) == "pub=chargee", f"exception de site sans effet : {state(i)}"
call(op="ui", request={"kind": "toggleShieldForSite", "id": i}); time.sleep(3)
assert state(i) == "pub=bloquee", state(i)
call(op="ui", request={"kind": "setShieldEnabled", "enabled": False}); time.sleep(3)
assert state(i) == "pub=chargee", f"coupure globale sans effet : {state(i)}"
print("exception de site et coupure globale ok")
PY
./stop.sh >/dev/null; sleep 3; ./start.sh release >/dev/null; sleep 16
python3 - <<PY
exec(open("$ECHO_RUN_DIR/h.py").read())
i = call(op="open", url=f"http://127.0.0.1:{os.environ['TPORT']}/")["id"]; time.sleep(3)
assert state(i) == "pub=chargee", f"coupure globale perdue a la relance : {state(i)}"
print("OK : bouclier par site et global, rechargement, persistance")
PY
