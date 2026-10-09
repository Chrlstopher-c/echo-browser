#!/usr/bin/env bash
# Decouvrabilite et profils : l'Aide (F1) liste les fonctions et ouvre le panneau Securite et reseau ; le cadenas de
# l'adresse l'ouvre aussi ; le « + » des pastilles cree un profil et y passe ; Reglages → Profils supprime et
# reinitialise (onglets fermes, dossier efface au lancement suivant). Une installation neuve a un seul profil. Instance
# isolee.
set -euo pipefail
cd "$(dirname "$0")/.."
W="$(mktemp -d)"; PORT=$((29000 + RANDOM % 500))
echo '<!doctype html><title>site</title><p>site</p>' > "$W/index.html"
python3 tools/serveur-test.py "$PORT" "$W" >/dev/null 2>&1 & SERVEUR=$!
export ECHO_RUN_DIR="$W/run" ECHO_DATA_DIR="$W/data" ECHO_CONTROL_NAME="aide-$$" ECHO_NO_WELCOME=1 W PORT
export ECHO_DEVTOOLS_PORT=$((30000 + RANDOM % 9000))
mkdir -p "$W/run"
trap './stop.sh >/dev/null 2>&1 || true; kill $SERVEUR 2>/dev/null || true' EXIT
./start.sh release >/dev/null; sleep 8
uv run -q --with websocket-client python - <<'PY'
import json, os, socket, sqlite3, subprocess, time, urllib.request, websocket
W, PORT, DT = os.environ["W"], os.environ["PORT"], os.environ["ECHO_DEVTOOLS_PORT"]
def connect():
    s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
    f = s.makefile("rw")
    def call(**r):
        f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
    return call
def ev(prefix, js):
    t = [t for t in json.load(urllib.request.urlopen(f"http://127.0.0.1:{DT}/json/list")) if t["url"].startswith(prefix)][0]
    ws = websocket.create_connection(t["webSocketDebuggerUrl"], suppress_origin=True)
    ws.send(json.dumps({"id": 1, "method": "Runtime.evaluate", "params": {"expression": js, "returnByValue": True}}))
    while (m := json.loads(ws.recv())).get("id") != 1: pass
    ws.close(); return m["result"]["result"].get("value")
BAR, PAGES = "echo://ui/index.html", "echo://ui/pages.html"
click = lambda prefix, sel: ev(prefix, f"(()=>{{const b=document.querySelector({json.dumps(sel)});if(!b)return 0;b.click();return 1}})()")
press = lambda prefix, text: ev(prefix, f"(()=>{{const b=[...document.querySelectorAll('button')].filter(x=>x.innerText.trim()==={json.dumps(text)});if(!b.length)return 0;b[b.length-1].click();return 1}})()")
texte = lambda prefix: ev(prefix, "document.body.textContent")
def until(cond, label, tries=30):
    for _ in range(tries):
        try:
            if cond(): return
        except Exception:
            pass
        time.sleep(0.5)
    raise AssertionError(label)
def profils():
    con = sqlite3.connect(f"file:{W}/data/library.db?mode=ro", uri=True)
    try:
        row = con.execute("select value from settings where key='profiles.list'").fetchone()
        return json.loads(row[0][2:]) if row and row[0].startswith("s:") and len(row[0]) > 2 else []
    finally: con.close()
call = connect()
call(op="open", url=f"http://localhost:{PORT}/"); time.sleep(2.5)
assert click(BAR, '[aria-label="Sécurité du site"]') == 1, "cadenas non cliquable"
until(lambda: "Isolement strict" in texte(BAR), "le cadenas n'ouvre pas la securite du site")
print("cadenas : securite du site ouverte")
click(BAR, '[aria-label="Aide (F1)"]'); time.sleep(2.5)
until(lambda: "Raccourcis clavier" in texte(PAGES) and "Sécurité du site" in texte(PAGES), "page Aide vide")
click(BAR, "[aria-label=\"Fermer (Échap)\"]")
ev(PAGES, "(()=>{const t=[...document.querySelectorAll('p')].find(p=>p.innerText==='Sécurité du site');t.closest('div.flex').querySelector('button').click();return 1})()")
until(lambda: "Isolement strict" in texte(BAR), "le bouton de l'Aide n'ouvre pas le panneau")
print("Aide : fonctions listees, bouton qui ouvre le panneau")
try:
    click(BAR, '[aria-label="Nouveau profil"]')
except Exception:
    pass  # changer de profil recharge la barre : la reponse peut se perdre, le resultat est verifie juste apres
until(lambda: len(profils()) == 2, "le + n'a pas cree de profil")
nouveau = profils()[-1]["id"]
until(lambda: any(t["space"] == nouveau for t in call(op="tabs")["tabs"]), "pas passe au nouveau profil")
print(f"+ : profil {nouveau} cree, on y est")
call(op="ui", request={"kind": "openPage", "page": "reglages"}); time.sleep(2.5)
REG = PAGES + "#reglages"
until(lambda: "Nouveau profil" in texte(REG), "reglages non ouverts")
ev(REG, f"(()=>{{const r=[...document.querySelectorAll('input[aria-label=\"Nom du profil\"]')].pop().closest('div');[...r.querySelectorAll('button')].find(b=>b.innerText.trim()==='Supprimer').click();return 1}})()")
time.sleep(0.5)
try:
    press(REG, "Supprimer")
except Exception:
    pass  # l'onglet Reglages appartient au profil supprime : il se ferme avec lui
time.sleep(1.5)
until(lambda: len(profils()) == 1, "profil non supprime de la liste")
assert nouveau in json.load(open(f"{W}/data/profils-a-effacer.json")), "effacement du dossier non programme"
assert not any(t["space"] == nouveau for t in call(op="tabs")["tabs"]), "onglets du profil supprime encore ouverts"
print("Reglages : profil supprime (onglets fermes, dossier programme pour effacement)")
subprocess.run(["./stop.sh"], capture_output=True); subprocess.run(["./start.sh", "release"], capture_output=True); time.sleep(8)
assert not os.path.exists(f"{W}/data/profils-a-effacer.json"), "effacement non fait au lancement"
assert not any(n.startswith(f"conteneur-profil-{nouveau}") for n in os.listdir(f"{W}/data/profile")), "dossier du profil encore la"
print("OK : Aide, cadenas, profils (creation, suppression, effacement au lancement)")
PY
