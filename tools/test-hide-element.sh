#!/usr/bin/env bash
# Memoire de structure : clic droit sur une banniere d'un article → « Masquer cet element » ; un autre article du meme
# gabarit (contenu different) l'a masquee aussi, une page d'un autre gabarit non ; « Reafficher » la rend. Instance isolee.
set -euo pipefail
cd "$(dirname "$0")/.."
W="$(mktemp -d)"; PORT=$((27000 + RANDOM % 3000))
article() { # fichier titre paragraphes
  { echo "<!doctype html><title>$2</title><body style='margin:0'><header class='entete'><h1>$2</h1></header>"
    echo "<div class='banniere promo' style='height:120px;background:#c33'>Abonnez-vous</div><article class='texte'>"
    for i in $(seq "$3"); do echo "<p>Paragraphe $i de $2.</p>"; done
    echo "</article><footer class='pied'>pied</footer></body>"; } > "$W/$1"
}
article a1.html "Premier article" 3; article a2.html "Second article" 7
echo "<!doctype html><title>autre</title><body style='margin:0'><main class='grille'><div class='banniere promo'
style='height:120px'>Autre</div><section class='cartes'><p>x</p></section></main></body>" > "$W/autre.html"
python3 tools/serveur-test.py "$PORT" "$W" >/dev/null 2>&1 & SERVEUR=$!
export ECHO_RUN_DIR="$W/run" ECHO_DATA_DIR="$W/data" ECHO_CONTROL_NAME="masque-$$" ECHO_DEVTOOLS_PORT=$((30000 + RANDOM % 9000))
export ECHO_NO_WELCOME=1 W PORT
mkdir -p "$W/run"
trap './stop.sh >/dev/null 2>&1 || true; kill $SERVEUR 2>/dev/null || true' EXIT
./start.sh release >/dev/null; sleep 8
uv run -q --with websocket-client python - <<'PY'
import json, os, socket, time, urllib.request, websocket
PORT, DT = os.environ["PORT"], os.environ["ECHO_DEVTOOLS_PORT"]
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
url = lambda p: f"http://localhost:{PORT}/{p}"
shown = lambda p: cdp(url(p), "Runtime.evaluate", {"expression": "getComputedStyle(document.querySelector('.banniere')).display",
    "returnByValue": True})["result"]["value"] != "none"
def right_click(p, x, y):
    for kind in ("mousePressed", "mouseReleased"):
        cdp(url(p), "Input.dispatchMouseEvent", {"type": kind, "x": x, "y": y, "button": "right", "clickCount": 1})
    time.sleep(1)
tab = call(op="open", url=url("a1.html"))["id"]; time.sleep(3)
assert shown("a1.html")
right_click("a1.html", 200, 100)
call(op="ui", request={"kind": "runContextMenu", "action": "hideElement"}); time.sleep(1.5)
assert not shown("a1.html"), "banniere toujours visible apres « Masquer »"
call(op="navigate", id=tab, url=url("a2.html")); time.sleep(3)
assert not shown("a2.html"), "meme gabarit, banniere visible sur le second article"
print("masquee sur l'article et sur un autre article du meme gabarit")
call(op="navigate", id=tab, url=url("autre.html")); time.sleep(3)
assert shown("autre.html"), "masquee sur une page d'un autre gabarit"
print("autre gabarit : banniere visible")
call(op="navigate", id=tab, url=url("a2.html")); time.sleep(3)
right_click("a2.html", 200, 300)
call(op="ui", request={"kind": "runContextMenu", "action": "unhideElements"}); time.sleep(3)
assert shown("a2.html"), "« Reafficher » sans effet"
print("OK : memoire de structure (masquee par gabarit, pas ailleurs, reaffichee)")
PY
