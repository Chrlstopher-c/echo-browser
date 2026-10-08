#!/usr/bin/env bash
# Surveiller une page : clic droit → « Surveiller cette page » (le texte devient la reference) ; le contenu change cote
# serveur ; nouvelle visite → bandeau avec la ligne ajoutee et la ligne retiree. Une imitation de la console hors de la
# fenetre de lecture est ignoree ; « Ne plus surveiller » retire la page. Instance isolee.
set -euo pipefail
cd "$(dirname "$0")/.."
W="$(mktemp -d)"; PORT=$((29000 + RANDOM % 900))
printf '<!doctype html><meta charset="utf-8"><title>prix</title><h1>Casque</h1><p>Prix : 20 €</p><p>Stock : 3</p>' > "$W/prix.html"
python3 tools/serveur-test.py "$PORT" "$W" >/dev/null 2>&1 & SERVEUR=$!
export ECHO_RUN_DIR="$W/run" ECHO_DATA_DIR="$W/data" ECHO_CONTROL_NAME="surveille-$$" ECHO_DEVTOOLS_PORT=$((30000 + RANDOM % 9000))
export ECHO_NO_WELCOME=1 W PORT
mkdir -p "$W/run"
trap './stop.sh >/dev/null 2>&1 || true; kill $SERVEUR 2>/dev/null || true' EXIT
./start.sh release >/dev/null; sleep 8
uv run -q --with websocket-client python - <<'PY'
import json, os, socket, sqlite3, time, urllib.request, websocket
W, PORT, DT = os.environ["W"], os.environ["PORT"], os.environ["ECHO_DEVTOOLS_PORT"]
page = f"http://localhost:{PORT}/prix.html"
s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = s.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
def cdp(prefix, method, params):
    t = [t for t in json.load(urllib.request.urlopen(f"http://127.0.0.1:{DT}/json/list")) if t["url"].startswith(prefix)][0]
    ws = websocket.create_connection(t["webSocketDebuggerUrl"], suppress_origin=True)
    ws.send(json.dumps({"id": 1, "method": method, "params": params}))
    while (m := json.loads(ws.recv())).get("id") != 1: pass
    ws.close(); return m.get("result", {})
sidebar = lambda: cdp("echo://ui/index.html", "Runtime.evaluate", {"expression": "document.body.innerText", "returnByValue": True})["result"]["value"]
def right_click():
    for kind in ("mousePressed", "mouseReleased"):
        cdp(page, "Input.dispatchMouseEvent", {"type": kind, "x": 100, "y": 100, "button": "right", "clickCount": 1})
    time.sleep(1)
def watched():
    con = sqlite3.connect(f"file:{W}/data/library.db?mode=ro", uri=True)
    try: return con.execute("select count(*), max(text is not null) from watched_pages").fetchone()
    finally: con.close()
tab = call(op="open", url=page)["id"]; time.sleep(3)
right_click(); call(op="ui", request={"kind": "runContextMenu", "action": "watchPage"}); time.sleep(3)
assert watched() == (1, 1), f"reference non prise : {watched()}"
print("page surveillee, reference prise")
cdp(page, "Runtime.evaluate", {"expression": "console.debug('echo:texte:'+JSON.stringify('faux contenu imite'))"}); time.sleep(1)
assert "a changé" not in sidebar(), "une imitation de la console a ete crue"
open(f"{W}/prix.html", "w").write('<!doctype html><meta charset="utf-8"><title>prix</title><h1>Casque</h1><p>Prix : 18 €</p><p>Stock : 3</p>')
call(op="navigate", id=tab, url=page)
for _ in range(20):
    texte = sidebar()
    if "a changé" in texte: break
    time.sleep(0.5)
else:
    raise AssertionError("bandeau de changement absent")
assert "Prix : 18 €" in texte and "Prix : 20 €" in texte, f"diff incomplet : {texte[-300:]!r}"
print("nouvelle visite : bandeau avec + Prix : 18 € et − Prix : 20 €")
right_click(); call(op="ui", request={"kind": "runContextMenu", "action": "unwatchPage"}); time.sleep(1)
assert watched()[0] == 0, "page toujours surveillee"
print("OK : surveiller une page (reference, changement montre, imitation ignoree, retrait)")
PY
