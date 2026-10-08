#!/usr/bin/env bash
# Palette d'adresse : la saisie propose les onglets ouverts et l'historique, Ctrl+K donne le focus a l'adresse,
# les fleches + Entree choisissent : un onglet ouvert est active, une page de l'historique est ouverte. Instance isolee.
set -euo pipefail
cd "$(dirname "$0")/.."
W="$(mktemp -d)"; PORT=$((29000 + RANDOM % 500))
for n in alpha beta gamma; do echo "<!doctype html><meta charset=utf-8><title>$n palette</title><p>$n" > "$W/$n.html"; done
python3 tools/serveur-test.py "$PORT" "$W" >/dev/null 2>&1 & SERVEUR=$!
export ECHO_RUN_DIR="$W/run" ECHO_DATA_DIR="$W/data" ECHO_CONTROL_NAME="palette-$$" ECHO_NO_WELCOME=1 W PORT
export ECHO_DEVTOOLS_PORT=$((30000 + RANDOM % 9000))
mkdir -p "$W/run"
trap './stop.sh >/dev/null 2>&1 || true; kill $SERVEUR 2>/dev/null || true; rm -rf "$W"' EXIT
./start.sh release >/dev/null; sleep 8
uv run -q --with websocket-client python - <<'PY'
import json, os, socket, time, urllib.request, websocket
PORT, DT = os.environ["PORT"], os.environ["ECHO_DEVTOOLS_PORT"]
s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = s.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
def ev(js):
    t = [t for t in json.load(urllib.request.urlopen(f"http://127.0.0.1:{DT}/json/list")) if t["url"].startswith("echo://ui/index.html")][0]
    ws = websocket.create_connection(t["webSocketDebuggerUrl"], suppress_origin=True)
    ws.send(json.dumps({"id": 1, "method": "Runtime.evaluate", "params": {"expression": js, "returnByValue": True}}))
    while (m := json.loads(ws.recv())).get("id") != 1: pass
    ws.close(); return m["result"]["result"].get("value")
def until(cond, label, tries=30):
    for _ in range(tries):
        try:
            if cond(): return
        except Exception:
            pass
        time.sleep(0.5)
    raise AssertionError(label)
INPUT = "document.querySelector('input[aria-label=\"Adresse\"]')"
# Le sway de test n'a pas de clavier : la fenetre n'a jamais le focus systeme, l'evenement de focus est donc simule.
def taper(text):
    ev(f"(()=>{{const i={INPUT};i.focus();if(!document.hasFocus())i.dispatchEvent(new FocusEvent('focusin',{{bubbles:true}}));const set=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set;"
       f"set.call(i,{json.dumps(text)});i.dispatchEvent(new Event('input',{{bubbles:true}}));return 1}})()")
def touche(key):
    ev(f"(()=>{{{INPUT}.dispatchEvent(new KeyboardEvent('keydown',{{key:{json.dumps(key)},bubbles:true}}));return 1}})()")
options = lambda: ev("[...document.querySelectorAll('[role=option]')].map(o=>o.getAttribute('aria-selected')+'|'+o.textContent)")
base = f"http://localhost:{PORT}"
# Les suggestions du moteur arrivent par le reseau et decaleraient les lignes : coupees ici (test-settings les couvre).
call(op="ui", request={"kind": "updateSetting", "key": "search.suggest", "value": {"type": "flag", "value": False}})
call(op="open", url=f"{base}/alpha.html"); time.sleep(2)
call(op="open", url=f"{base}/beta.html"); time.sleep(2)
beta = [t for t in call(op="tabs")["tabs"] if "beta" in t["url"]][0]
call(op="navigate", id=beta["id"], url=f"{base}/gamma.html"); time.sleep(2)
tabs = call(op="tabs")["tabs"]
alpha = [t for t in tabs if "alpha" in t["url"]][0]
assert ev(f"{INPUT}.blur(),1") == 1
call(op="key", code=75, ch="k", mods=["ctrl"]); time.sleep(1)
assert ev(f"document.activeElement==={INPUT}"), "Ctrl+K ne donne pas le focus a l'adresse"
print("Ctrl+K : focus sur l'adresse")
taper("palette")
until(lambda: len(options()) >= 3, f"suggestions absentes : {options()}")
textes = options()
assert any("alpha palette" in t for t in textes) and any("beta palette" in t for t in textes), textes
kinds = ev("[...document.querySelectorAll('[role=option]')].map(o=>o.dataset.kind)")
assert 'tab' in kinds and 'history' in kinds, f"sections attendues : onglets + historique, recu {kinds}"
print(f"suggestions : {len(textes)} (onglets + historique)")
touche("ArrowDown")
until(lambda: options()[0].startswith("true"), "fleche bas ne selectionne pas")
idx = [i for i, t in enumerate(options()) if "alpha palette" in t][0]
for _ in range(idx): touche("ArrowDown")
until(lambda: options()[idx].startswith("true") and "alpha" in options()[idx], "selection pas sur alpha")
touche("Enter")
until(lambda: [t for t in call(op="tabs")["tabs"] if t["active"]][0]["id"] == alpha["id"], "Entree n'active pas l'onglet ouvert")
assert len(call(op="tabs")["tabs"]) == len(tabs), "un onglet en trop a ete ouvert"
print("Entree sur un onglet ouvert : active, rien d'ouvert en plus")
taper("beta")
until(lambda: any("beta palette" in t for t in options()), "historique beta non propose")
idx = [i for i, t in enumerate(options()) if "beta palette" in t][0]
for _ in range(idx + 1): touche("ArrowDown")
touche("Enter")
until(lambda: "beta" in [t for t in call(op="tabs")["tabs"] if t["active"]][0]["url"], "Entree n'ouvre pas la page de l'historique")
assert options() == [], "la liste reste ouverte apres le choix"
print("OK : palette d'adresse (Ctrl+K, onglets ouverts et historique, choix au clavier)")
PY
