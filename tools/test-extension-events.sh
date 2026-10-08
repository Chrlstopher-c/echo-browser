#!/usr/bin/env bash
# Service workers d'extension : au demarrage, tabs.query voit les onglets d'Echo ; eveille, il recoit onUpdated quand un
# onglet change d'adresse ; endormi (veille de Chromium apres 30 s), il est reveille par un changement d'onglet et le
# recoit. Extension d'essai rangee dans l'inventaire (cle fixe, permission tabs). Instance isolee.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)" ECHO_CONTROL_NAME="test-$$" ECHO_DATA_DIR="$(mktemp -d)"
openssl genrsa 2048 2>/dev/null | openssl rsa -pubout -outform DER 2>/dev/null > "$ECHO_RUN_DIR/key.der"
ID=$(python3 -c "import hashlib,sys;h=hashlib.sha256(open(sys.argv[1],'rb').read()).hexdigest()[:32];print(''.join(chr(97+int(c,16)) for c in h))" "$ECHO_RUN_DIR/key.der")
EXT="$ECHO_DATA_DIR/extensions/$ID"; mkdir -p "$EXT"
cat > "$EXT/manifest.json" <<J
{"manifest_version":3,"name":"Sonde","version":"1.0","key":"$(base64 -w0 "$ECHO_RUN_DIR/key.der")","permissions":["tabs","storage"],
"background":{"service_worker":"bg.js"}}
J
cat > "$EXT/bg.js" <<'J'
let q = 'attente'
const ready = new Promise((done) => chrome.tabs.query({ url: 'https://example.com/*' }, (t) => { q = t.length; done() }))
const note = async (e) => {
  const { ev = [] } = await chrome.storage.session.get('ev')
  await chrome.storage.session.set({ ev: [...ev, e] })
}
chrome.tabs.onUpdated.addListener((id, change) => { if (change.url) note('maj:' + change.url + '@' + Date.now()) })
chrome.runtime.onMessage.addListener((m, s, reply) => {
  ready.then(() => chrome.storage.session.get('ev')).then(({ ev = [] }) => reply({ q, ev }))
  return true
})
J
echo '<title>s</title><pre id=o></pre><script src="s.js"></script>' > "$EXT/s.html"
echo "chrome.runtime.sendMessage('etat', (r) => { o.textContent = JSON.stringify(r) })" > "$EXT/s.js"
export ECHO_FLAGS="--load-extension=$EXT"
trap './stop.sh >/dev/null 2>&1 || true' EXIT
./start.sh release >/dev/null; sleep 45
python3 - "$ID" <<'PY'
import json, os, socket, sys, time
s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = s.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
def etat():
    t = call(op="open", url=f"chrome-extension://{sys.argv[1]}/s.html")["id"]; time.sleep(3)
    text = call(op="read", id=t)["text"].strip(); call(op="close", id=t); return json.loads(text)
page = call(op="open", url="https://example.com/")["id"]; time.sleep(3)
first = etat()
assert first["q"] == 1, f"tabs.query au demarrage ne voit pas l'onglet : {first}"
call(op="navigate", id=page, url="https://example.org/"); time.sleep(3)
second = etat()
assert any("example.org" in e for e in second["ev"]), f"onUpdated non recu : {second}"
time.sleep(45)
call(op="navigate", id=page, url="https://example.net/"); time.sleep(5)
asked = time.time() * 1000
third = etat()
woke = [int(e.split("@")[1]) for e in third["ev"] if "example.net" in e]
assert woke, f"changement fait pendant la veille non recu : {third}"
assert woke[0] < asked, f"recu seulement au reveil par la page, pas reveillee par Echo : {third}"
print(f"OK : service worker d'extension (query au demarrage, onUpdated eveille et apres la veille) {third}")
PY
