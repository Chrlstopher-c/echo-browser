#!/usr/bin/env bash
# Extensions : chrome.tabs.query({active, currentWindow}) depuis la fenetre d'une extension rend l'onglet affiche dans
# Echo, avec un identifiant reel (message au script de contenu de la page). Pont interne + polyfill. Instance isolee.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)" ECHO_CONTROL_NAME="test-$$" ECHO_DATA_DIR="$(mktemp -d)"
export ECHO_DEVTOOLS_PORT=$((30000 + RANDOM % 20000))
# Cle fixe : l'identifiant ne depend plus du chemin, l'extension se range la ou l'inventaire d'Echo la trouve.
openssl genrsa 2048 2>/dev/null | openssl rsa -pubout -outform DER 2>/dev/null > "$ECHO_RUN_DIR/key.der"
ID=$(python3 -c "import hashlib,sys;h=hashlib.sha256(open(sys.argv[1],'rb').read()).hexdigest()[:32];print(''.join(chr(97+int(c,16)) for c in h))" "$ECHO_RUN_DIR/key.der")
EXT="$ECHO_DATA_DIR/extensions/$ID"; mkdir -p "$EXT"
cat > "$EXT/manifest.json" <<J
{"manifest_version":3,"name":"Sonde","version":"1.0","key":"$(base64 -w0 "$ECHO_RUN_DIR/key.der")","permissions":["tabs"],
"action":{"default_popup":"popup.html"},"content_scripts":[{"matches":["<all_urls>"],"js":["cs.js"]}]}
J
echo "chrome.runtime.onMessage.addListener((m,s,r)=>{if(m==='ping')r('pong:'+location.href)})" > "$EXT/cs.js"
echo '<title>sonde</title><pre id=o></pre><script src="popup.js"></script>' > "$EXT/popup.html"
cat > "$EXT/popup.js" <<'J'
chrome.tabs.query({active:true,currentWindow:true},async t=>{let msg='';try{msg=await chrome.tabs.sendMessage(t[0].id,'ping')}catch(e){msg='ERR '+e}
const all=await chrome.tabs.query({});o.textContent='ACTIVE '+t.map(x=>x.url).join(',')+' MSG '+msg+' ALL '+all.length+' HTTPS '+(await chrome.tabs.query({url:'https://*/*'})).length})
J
export ECHO_FLAGS="--load-extension=$EXT"
trap './stop.sh >/dev/null 2>&1 || true' EXIT
./start.sh release >/dev/null; sleep 14
uv run -q --with websocket-client python - "$ID" <<'PY'
import json, os, socket, sys, time, urllib.request, websocket
s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = s.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
call(op="open", url="https://example.org/"); time.sleep(3)
call(op="open", url="https://example.com/"); time.sleep(4)
call(op="ui", request={"kind": "openExtensionPopup", "id": sys.argv[1], "anchor": {"x": 10, "y": 10, "width": 20, "height": 20}})
time.sleep(4)
port = os.environ["ECHO_DEVTOOLS_PORT"]
popup = [t for t in json.load(urllib.request.urlopen(f"http://127.0.0.1:{port}/json/list"))
         if t["url"] == f"chrome-extension://{sys.argv[1]}/popup.html"]
assert popup, "fenetre d'extension absente"
ws = websocket.create_connection(popup[0]["webSocketDebuggerUrl"], suppress_origin=True)
ws.send(json.dumps({"id": 1, "method": "Runtime.evaluate", "params": {"expression": "o.textContent", "returnByValue": True}}))
while (m := json.loads(ws.recv())).get("id") != 1: pass
text = m["result"]["result"].get("value", "")
assert text.startswith("ACTIVE https://example.com/ MSG pong:https://example.com/"), f"onglet actif non vu : {text!r}"
assert " ALL 3 " in text and text.endswith("HTTPS 2"), f"liste des onglets : {text!r}"
print(f"OK : tabs.query voit les onglets d'Echo ({text})")
PY
