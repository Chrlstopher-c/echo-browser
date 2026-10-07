#!/usr/bin/env bash
# Extensions : chrome.tabs.create depuis la fenetre d'une extension ET depuis son arriere-plan ouvre un
# vrai onglet d'Echo (fenetre d'ancrage + reprise par le protocole de debogage). Instance isolee.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)" ECHO_CONTROL_NAME="test-$$" ECHO_DATA_DIR="$(mktemp -d)"
export ECHO_DEVTOOLS_PORT=$((30000 + RANDOM % 20000))
EXT="$ECHO_RUN_DIR/ext"; mkdir -p "$EXT"
cat > "$EXT/manifest.json" <<'J'
{"manifest_version":3,"name":"Essai Echo","version":"1.0","permissions":["tabs"],"background":{"service_worker":"bg.js"},"action":{"default_popup":"popup.html"}}
J
echo "chrome.runtime.onMessage.addListener(m => { if (m === 'bg') chrome.tabs.create({url: 'https://example.org/?depuis=fond'}) })" > "$EXT/bg.js"
echo '<title>essai</title><script src="popup.js"></script>' > "$EXT/popup.html"
echo "chrome.tabs.create({url: 'https://example.com/?depuis=popup'}); chrome.runtime.sendMessage('bg')" > "$EXT/popup.js"
export ECHO_FLAGS="--load-extension=$EXT"
trap './stop.sh >/dev/null 2>&1 || true' EXIT
./start.sh release >/dev/null; sleep 14
ID=$(curl -s "http://127.0.0.1:$ECHO_DEVTOOLS_PORT/json/list" | python3 -c "import json,sys;[print(t['url'].split('/')[2]) for t in json.load(sys.stdin) if t['url'].startswith('chrome-extension://') and 'mcndjimfalplibhknmeieoolkckkpnnc' not in t['url']]" | head -1)
python3 - "$ID" <<'PY'
import json, os, socket, sys, time
s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = s.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
call(op="open", url=f"chrome-extension://{sys.argv[1]}/popup.html"); time.sleep(6)
urls = [t["url"] for t in call(op="tabs")["tabs"]]
assert "https://example.com/?depuis=popup" in urls, f"onglet de la fenetre d'extension absent : {urls}"
assert "https://example.org/?depuis=fond" in urls, f"onglet de l'arriere-plan d'extension absent : {urls}"
print("OK : onglets ouverts par une extension")
PY
