#!/usr/bin/env bash
# « Examiner l'élément » : les outils ancrés s'ouvrent avec l'élément cliqué sélectionné (pas seulement la page), y compris
# quand l'ouverture du panneau fait bouger la mise en page ; la largeur du panneau survit à une relance. Instance isolée.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)" ECHO_CONTROL_NAME="test-$$" ECHO_DATA_DIR="$(mktemp -d)"
export ECHO_DEVTOOLS_PORT=$((30000 + RANDOM % 20000))
cat > "$ECHO_RUN_DIR/p.html" <<'H'
<title>p</title><body style="margin:0;display:grid;grid-template-columns:1fr 1fr;height:100vh">
<div id=gauche style="background:#345">gauche</div><div id=droite style="background:#543">droite</div>
H
trap './stop.sh >/dev/null 2>&1 || true' EXIT
./start.sh release >/dev/null; sleep 12
uv run -q --with websocket-client python - <<'PY'
import json, os, socket, time, urllib.request, websocket
s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = s.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
call(op="open", url="file://" + os.environ["ECHO_RUN_DIR"] + "/p.html"); time.sleep(3)
width = call(op="layout")["page"]["width"]
# Clic au bord droit de « gauche » : une fois le panneau ouvert, la page retrecit et ce point tombe sur « droite ».
call(op="click", x=width // 2 - 40, y=200, button="right"); time.sleep(1)
call(op="ui", request={"kind": "runContextMenu", "action": "inspect"})
port = os.environ["ECHO_DEVTOOLS_PORT"]
selected = None
for _ in range(40):
    time.sleep(0.5)
    front = [t for t in json.load(urllib.request.urlopen(f"http://127.0.0.1:{port}/json/list"))
             if t["url"].startswith("devtools://devtools/bundled/devtools_app.html")]
    if not front: continue
    ws = websocket.create_connection(front[0]["webSocketDebuggerUrl"], suppress_origin=True)
    expr = ("(async()=>{const b='devtools://devtools/bundled/';const UI=await import(b+'ui/legacy/legacy.js');"
            "const SDK=await import(b+'core/sdk/sdk.js');const n=UI.Context.Context.instance().flavor(SDK.DOMModel.DOMNode);"
            "return n?n.getAttribute('id')||n.nodeName():null})()")
    ws.send(json.dumps({"id": 1, "method": "Runtime.evaluate", "params": {"expression": expr, "awaitPromise": True, "returnByValue": True}}))
    while (m := json.loads(ws.recv())).get("id") != 1: pass
    selected = m.get("result", {}).get("result", {}).get("value")
    ws.close()
    if selected == "gauche": break
assert selected == "gauche", f"element selectionne : {selected!r}"
call(op="ui", request={"kind": "resizeDevTools", "dx": -120}); time.sleep(1.5)
print(f"OK : examiner l'element selectionne #{selected}")
PY
W1=$(sqlite3 "$ECHO_DATA_DIR"/*.db "select value from settings where key='devtools.width'" 2>/dev/null || true)
[ -n "$W1" ] && [ "$W1" != "560" ] || { echo "largeur non enregistree ($W1)"; exit 1; }
echo "OK : largeur des outils enregistree ($W1)"
