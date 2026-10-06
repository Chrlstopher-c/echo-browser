#!/usr/bin/env bash
# Conteneurs : un cookie pose dans un conteneur n'apparait ni dans un autre, ni dans le contexte commun,
# et il survit a une relance. Serveur local de test lance (et arrete) par ce script.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)"
export ECHO_CONTROL_NAME="test-$$"
export ECHO_DATA_DIR="$(mktemp -d)"
PORT=$((20000 + RANDOM % 20000))
python3 - "$PORT" <<'PYS' &
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer as HTTPServer
class H(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        if self.path.startswith("/set"):
            self.send_header("Set-Cookie", "compte=alice; Max-Age=86400; Path=/")
        self.end_headers()
        self.wfile.write(("<title>t</title><p>cookies=[%s]</p>" % self.headers.get("Cookie", "")).encode())
    def log_message(self, *a): pass
HTTPServer(("127.0.0.1", int(sys.argv[1])), H).serve_forever()
PYS
SERVER=$!
trap './stop.sh >/dev/null 2>&1 || true; kill $SERVER 2>/dev/null || true' EXIT
cat > "$ECHO_RUN_DIR/h.py" <<'PYH'
import json, os, socket, time
sock = socket.socket(socket.AF_UNIX)
sock.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
stream = sock.makefile("rw")
def call(**r):
    stream.write(json.dumps(r) + "\n"); stream.flush()
    return json.loads(stream.readline())
def cookies(path, container):
    before = {t["id"] for t in call(op="tabs")["tabs"]}
    call(op="open", url=f"http://127.0.0.1:{os.environ['TPORT']}{path}", container=container)
    new = None
    for _ in range(40):
        time.sleep(0.25)
        fresh = [t for t in call(op="tabs")["tabs"] if t["id"] not in before]
        if fresh:
            new = fresh[0]; break
    assert new is not None, "onglet non ouvert"
    assert new["container"] == container, ("mauvais conteneur", new)
    for _ in range(40):
        time.sleep(0.5)
        out = call(op="read", id=new["id"])
        if out.get("ok") and "cookies=" in out["text"]:
            return out["text"]
    raise AssertionError(("page non chargee", new, out))
PYH
export TPORT=$PORT
./start.sh release >/dev/null; sleep 12
python3 - <<PY
exec(open("$ECHO_RUN_DIR/h.py").read())
cookies("/set", "pro")
assert "compte=alice" in cookies("/", "pro"), "le cookie n'est pas rendu au meme conteneur"
assert "compte=alice" not in cookies("/", "perso"), "le cookie fuit vers un autre conteneur"
assert "compte=alice" not in cookies("/", None), "le cookie fuit vers le contexte commun"
call(op="open", url="echo://ui/nouvel-onglet.html", container="perso")
time.sleep(3)
home = [t for t in call(op="tabs")["tabs"] if t["container"] == "perso" and t["url"].startswith("echo://")]
assert home and home[-1]["title"] == "Nouvel onglet", f"page d'accueil KO dans un conteneur : {home}"
print("isolation ok")
PY
./stop.sh >/dev/null; sleep 3
./start.sh release >/dev/null; sleep 12
python3 - <<PY
exec(open("$ECHO_RUN_DIR/h.py").read())
assert "compte=alice" in cookies("/", "pro"), "le cookie n'a pas survecu a la relance"
print("OK : conteneurs")
PY
