#!/usr/bin/env bash
# Veille : un onglet endormi puis reveille retrouve son defilement. Serveur local de test lance (et arrete) ici.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)"
export ECHO_CONTROL_NAME="test-$$"
export ECHO_DATA_DIR="$(mktemp -d)"
PORT=$((20000 + RANDOM % 20000))
python3 - "$PORT" <<'PYS' &
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
PAGE = b"<title>t</title><body style='margin:0'><p id=p style='position:fixed;top:0'>y=0</p><div style='height:9000px'></div>" \
       b"<script>const u=()=>p.textContent='y='+Math.round(scrollY);addEventListener('scroll',u);setInterval(u,200)</script>"
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
def y(tab):
    for _ in range(40):
        out = call(op="read", id=tab)
        m = re.search(r"y=(\d+)", out.get("text", ""))
        if m: return int(m.group(1))
        time.sleep(0.25)
    raise AssertionError(("page illisible", out))
call(op="open", url=f"http://127.0.0.1:{os.environ['TPORT']}/")
time.sleep(3)
tab = [t for t in call(op="tabs")["tabs"] if t["active"]][0]["id"]
for _ in range(5):
    call(op="wheel", dy=600); time.sleep(0.3)
time.sleep(1.5)
before = y(tab); assert before > 1000, f"la page n'a pas defile : {before}"
call(op="open", url="https://example.org/"); time.sleep(3)
assert call(op="sleep", id=tab)["ok"]; time.sleep(1)
assert [t for t in call(op="tabs")["tabs"] if t["id"] == tab][0]["asleep"], "onglet non endormi"
call(op="activate", id=tab); time.sleep(4)
after = y(tab)
assert abs(after - before) < 60, f"defilement non restitue : {before} -> {after}"
print(f"OK : veille + defilement ({before} -> {after})")
PY
