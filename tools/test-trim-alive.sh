#!/usr/bin/env bash
# Allegement memoire des onglets d'arriere-plan : la page doit rester vivante (JS actif, ecouteurs en place), sans rechargement.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)" ECHO_CONTROL_NAME="test-$$" ECHO_DATA_DIR="$(mktemp -d)" ECHO_TRIM_AFTER_S=2
PORT=$((20000 + RANDOM % 20000))
python3 - "$PORT" <<'PYS' &
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
PAGE = b"<title>t</title><p id=p></p><script>let n=0;const t0=Date.now();setInterval(()=>{n++;" \
       b"p.textContent='n='+n+' age='+Math.round((Date.now()-t0)/1000)},250)</script>"
class H(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200); self.send_header("Content-Type", "text/html"); self.end_headers(); self.wfile.write(PAGE)
    def log_message(self, *a): pass
ThreadingHTTPServer(("127.0.0.1", int(sys.argv[1])), H).serve_forever()
PYS
SERVER=$!
trap './stop.sh >/dev/null 2>&1 || true; kill $SERVER 2>/dev/null || true' EXIT
./start.sh release >/dev/null; sleep 12
TPORT=$PORT python3 - <<'PY'
import json, os, re, socket, time
sock = socket.socket(socket.AF_UNIX)
sock.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
stream = sock.makefile("rw")
def call(**r):
    stream.write(json.dumps(r) + "\n"); stream.flush()
    return json.loads(stream.readline())
def state(tab):
    m = re.search(r"n=(\d+) age=(\d+)", call(op="read", id=tab).get("text", ""))
    assert m, "page illisible"
    return int(m.group(1)), int(m.group(2))
tab = call(op="open", url=f"http://127.0.0.1:{os.environ['TPORT']}/")["id"]; time.sleep(3)
call(op="open", url="about:blank"); time.sleep(10)
assert [t for t in call(op="tabs")["tabs"] if t["id"] == tab][0].get("asleep") is False, "onglet endormi"
call(op="activate", id=tab); time.sleep(2)
n1, age1 = state(tab); time.sleep(1.5); n2, age2 = state(tab)
assert n2 > n1, f"JavaScript de la page arrete apres allegement : n={n1} -> {n2}, fige a {age1} s"
assert age1 >= 12, f"page rechargee (age {age1} s)"
print(f"OK : page vivante apres allegement (age {age2} s, n {n1} -> {n2})")
PY
