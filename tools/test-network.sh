#!/usr/bin/env bash
# Panneau Reseau : une page locale (localhost) charge un script tiers (127.0.0.1). Le panneau montre les deux domaines ;
# « Bloquer » le tiers → au rechargement, plus aucune requete vers lui (vu par le serveur) ; regle gardee a la relance ;
# isolement strict → aucun tiers. Instance isolee. Usage : tools/test-network.sh
set -euo pipefail
cd "$(dirname "$0")/.."
W="$(mktemp -d)"; PORT=$((20000 + RANDOM % 5000))
cat > "$W/index.html" <<H
<!doctype html><title>reseau</title><p>page</p><script src="http://127.0.0.1:$PORT/tiers.js"></script><img src="/logo.png">
H
echo "document.title='tiers charge'" > "$W/tiers.js"
echo '<!doctype html><title>geo</title><script>navigator.geolocation.getCurrentPosition(()=>{},()=>{})</script>' > "$W/geo.html"; printf '\x89PNG' > "$W/logo.png"
python3 -m http.server "$PORT" --directory "$W" >"$W/serveur.log" 2>&1 & SERVEUR=$!
export ECHO_RUN_DIR="$W/run" ECHO_DATA_DIR="$W/data" ECHO_CONTROL_NAME="reseau-$$" ECHO_DEVTOOLS_PORT=$((30000 + RANDOM % 9000))
export ECHO_NO_WELCOME=1 W PORT
mkdir -p "$W/run"
trap './stop.sh >/dev/null 2>&1 || true; kill $SERVEUR 2>/dev/null || true' EXIT
./start.sh release >/dev/null; sleep 8
uv run -q --with websocket-client python - <<'PY'
import json, os, socket, subprocess, time, urllib.request, websocket
W, PORT, DT = os.environ["W"], os.environ["PORT"], os.environ["ECHO_DEVTOOLS_PORT"]
def connect():
    s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
    f = s.makefile("rw")
    def call(**r):
        f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
    return call
def ui(js):
    t = [t for t in json.load(urllib.request.urlopen(f"http://127.0.0.1:{DT}/json/list")) if t["url"].startswith("echo://ui/index.html")][0]
    ws = websocket.create_connection(t["webSocketDebuggerUrl"], suppress_origin=True)
    ws.send(json.dumps({"id": 1, "method": "Runtime.evaluate", "params": {"expression": js, "returnByValue": True}}))
    while (m := json.loads(ws.recv())).get("id") != 1: pass
    ws.close(); return m["result"]["result"].get("value")
hits = lambda: open(f"{W}/serveur.log").read().count("GET /tiers.js")
def until(cond, label, tries=40):
    for _ in range(tries):
        if cond(): return
        time.sleep(0.5)
    raise AssertionError(label)
call = connect()
page = f"http://localhost:{PORT}/"
tab = call(op="open", url=page)["id"]; time.sleep(3)
assert ui("(()=>{const b=document.querySelector('[aria-label=\"Réseau\"]');b.click();return !!b})()"), "bouton Reseau absent"
until(lambda: "127.0.0.1" in ui("document.body.innerText") and "localhost" in ui("document.body.innerText"), "domaines absents du panneau")
print("panneau : domaines de la page et tiers visibles")
segment = "(t)=>{const b=[...document.querySelectorAll('button,[role=tab]')].find(x=>x.innerText.trim()===t);b.click();return 1}"
ui(f"({segment})('Journal')")
until(lambda: "Premier contact avec 127.0.0.1" in ui("document.body.innerText"), "premier contact du tiers absent du journal")
geo = call(op="open", url=f"http://localhost:{PORT}/geo.html")["id"]
until(lambda: "Refuser" in ui("document.body.innerText"), "question de position absente")
ui(f"({segment})('Refuser')")
until(lambda: "Position demandé — refusée" in ui("document.body.innerText"), "decision de position absente du journal")
call(op="close", id=geo); time.sleep(1)
ui(f"({segment})('Domaines')")
print("journal : premier contact avec le tiers, position demandee et refusee")
avant = hits()
assert ui("(()=>{const b=[...document.querySelectorAll('button')].find(x=>x.innerText.trim()==='Bloquer');b.click();return 1})()") == 1
time.sleep(1); call(op="navigate", id=tab, url=page); time.sleep(3)
assert hits() == avant, f"le tiers bloque a encore ete demande ({hits() - avant} fois)"
assert "127.0.0.1" in json.load(open(f"{W}/data/reseau.json"))["blocked"].get("localhost", []), "regle non enregistree"
print("tiers bloque sur ce site : plus demande au rechargement, regle enregistree")
subprocess.run(["./stop.sh"], capture_output=True); subprocess.run(["./start.sh", "release"], capture_output=True); time.sleep(8)
call = connect(); avant = hits()
tab = call(op="open", url=page)["id"]; time.sleep(3)
assert hits() == avant, "regle perdue a la relance"
print("regle gardee a la relance")
regles = json.load(open(f"{W}/data/reseau.json")); regles["blocked"] = {}; regles["strict"] = []
assert ui("(()=>{document.querySelector('[aria-label=\"Réseau\"]').click();return 1})()") == 1; time.sleep(1)
until(lambda: "Débloquer" in ui("document.body.innerText"), "bouton Debloquer absent")
ui("(()=>{[...document.querySelectorAll('button')].find(x=>x.innerText.trim()==='Débloquer').click();return 1})()"); time.sleep(1)
call(op="navigate", id=tab, url=page); time.sleep(3)
assert hits() == avant + 1, "debloque mais toujours bloque"
ui("(()=>{document.querySelector('[aria-label=\"Isolement strict\"]').click();return 1})()"); time.sleep(1)
avant = hits(); call(op="navigate", id=tab, url=page); time.sleep(3)
assert hits() == avant, "isolement strict : un tiers est passe"
print("OK : reseau (domaines visibles, blocage par site garde a la relance, deblocage, isolement strict)")
PY
