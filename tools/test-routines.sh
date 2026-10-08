#!/usr/bin/env bash
# Routines : la meme suite de trois sites (localhost → 127.0.0.1 → 127.0.0.2) parcourue trois fois → Echo propose une
# routine en bas de la barre ; « Creer » → elle est enregistree ; l'ouvrir rouvre les trois pages. Instance isolee.
set -euo pipefail
cd "$(dirname "$0")/.."
W="$(mktemp -d)"; PORT=$((26000 + RANDOM % 4000))
echo '<!doctype html><title>site</title><p>site</p>' > "$W/index.html"
python3 -m http.server "$PORT" --directory "$W" >/dev/null 2>&1 & SERVEUR=$!
export ECHO_RUN_DIR="$W/run" ECHO_DATA_DIR="$W/data" ECHO_CONTROL_NAME="routine-$$" ECHO_DEVTOOLS_PORT=$((30000 + RANDOM % 9000))
export ECHO_NO_WELCOME=1 ECHO_ROUTINE_GAP_S=0 W PORT
mkdir -p "$W/run"
trap './stop.sh >/dev/null 2>&1 || true; kill $SERVEUR 2>/dev/null || true' EXIT
./start.sh release >/dev/null; sleep 8
uv run -q --with websocket-client python - <<'PY'
import json, os, socket, sqlite3, time, urllib.request, websocket
W, PORT, DT = os.environ["W"], os.environ["PORT"], os.environ["ECHO_DEVTOOLS_PORT"]
s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = s.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
def ui(js):
    t = [t for t in json.load(urllib.request.urlopen(f"http://127.0.0.1:{DT}/json/list")) if t["url"].startswith("echo://ui/index.html")][0]
    ws = websocket.create_connection(t["webSocketDebuggerUrl"], suppress_origin=True)
    ws.send(json.dumps({"id": 1, "method": "Runtime.evaluate", "params": {"expression": js, "returnByValue": True}}))
    while (m := json.loads(ws.recv())).get("id") != 1: pass
    ws.close(); return m["result"]["result"].get("value")
sites = [f"http://localhost:{PORT}/", f"http://127.0.0.1:{PORT}/", f"http://127.0.0.2:{PORT}/"]
tab = call(op="open", url="about:blank")["id"]
for tour in range(3):
    for url in sites:
        call(op="navigate", id=tab, url=url); time.sleep(1.5)
for _ in range(20):
    if "Vous ouvrez souvent" in ui("document.body.innerText"): break
    time.sleep(0.5)
else:
    raise AssertionError("routine non proposee")
print("proposition de routine affichee")
ui("(()=>{[...document.querySelectorAll('button')].find(b=>b.innerText.trim()==='Créer').click();return 1})()"); time.sleep(1)
con = sqlite3.connect(f"file:{W}/data/library.db?mode=ro", uri=True)
rows = con.execute("select id, name, urls from routines").fetchall(); con.close()
assert len(rows) == 1 and len(json.loads(rows[0][2])) == 3, f"routine non enregistree : {rows}"
avant = len(call(op="tabs")["tabs"])
call(op="ui", request={"kind": "routineOpen", "id": rows[0][0]}); time.sleep(3)
apres = call(op="tabs")["tabs"]
assert len(apres) == avant + 3, f"routine ouverte : {len(apres) - avant} onglet(s) au lieu de 3"
print("OK : routines (suite repetee 3 fois → proposition → routine creee → ouverte d'un geste)")
PY
