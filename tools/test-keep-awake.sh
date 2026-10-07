#!/usr/bin/env bash
# « Garder éveillé » : un onglet coché n'est ni endormi ni allégé (sa page reste vivante), décoché il retombe dans la
# veille normale, et le choix survit à une relance. Délais de veille raccourcis. Instance isolée.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)" ECHO_CONTROL_NAME="test-$$" ECHO_DATA_DIR="$(mktemp -d)"
export ECHO_SLEEP_AFTER_S=4 ECHO_TRIM_AFTER_S=2
PORT=$((20000 + RANDOM % 20000))
python3 - "$PORT" <<'PYS' &
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
PAGE = b"<title>t</title><p id=p></p><script>let n=0;setInterval(()=>{n++;p.textContent='n='+n},250)</script>"
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
def connect():
    for _ in range(60):
        try:
            s = socket.socket(socket.AF_UNIX)
            s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
            return s.makefile("rw")
        except OSError:
            time.sleep(0.5)
    raise AssertionError("prise de controle absente")
f = connect()
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
def tab(i): return [t for t in call(op="tabs")["tabs"] if t["id"] == i][0]
def counter(i): return int(re.search(r"n=(\d+)", call(op="read", id=i)["text"]).group(1))
url = f"http://127.0.0.1:{os.environ['TPORT']}/"
kept = call(op="open", url=url)["id"]; time.sleep(2)
other = call(op="open", url=url)["id"]; time.sleep(2)
call(op="ui", request={"kind": "keepTabAwake", "id": kept, "keep": True})
call(op="open", url="about:blank"); time.sleep(25)
assert tab(kept)["keepAwake"], "option non retenue"
assert not tab(kept)["asleep"], "onglet garde eveille endormi"
assert tab(other)["asleep"], "la veille normale ne fonctionne plus (temoin non endormi)"
n1 = counter(kept); time.sleep(1.5); n2 = counter(kept)
assert n2 > n1, f"page gardee eveillee figee : {n1} -> {n2}"
f = None
PY
./stop.sh >/dev/null; sleep 2; ./start.sh release >/dev/null; sleep 30
TPORT=$PORT python3 - <<'PY'
import json, os, socket, time
s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = s.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
kept = [t for t in call(op="tabs")["tabs"] if t.get("keepAwake")]
assert len(kept) == 1, f"choix perdu a la relance : {call(op='tabs')['tabs']}"
assert not kept[0]["asleep"], "onglet garde eveille laisse endormi apres la relance"
call(op="ui", request={"kind": "keepTabAwake", "id": kept[0]["id"], "keep": False}); time.sleep(25)
assert [t for t in call(op="tabs")["tabs"] if t["id"] == kept[0]["id"]][0]["asleep"], "decoche, l'onglet ne s'endort plus"
print("OK : garder eveille (veille et allegement epargnes, relance, decochage)")
PY
