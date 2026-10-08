#!/usr/bin/env bash
# Formulaires : Reglages → Formulaires cree une fiche et l'enregistre ; clic droit dans un champ → « Remplir : fiche »
# remplit les champs vides reconnus (autocomplete, nom, libelle, liste), ne touche ni a un champ deja saisi, ni au
# mot de passe, ni a la carte bancaire. Instance isolee.
set -euo pipefail
cd "$(dirname "$0")/.."
W="$(mktemp -d)"; PORT=$((29000 + RANDOM % 900))
cat > "$W/form.html" <<'HTML'
<!doctype html><meta charset="utf-8"><title>inscription</title>
<form style="display:grid;gap:6px;width:300px;padding:20px">
<input id="prenom" autocomplete="given-name">
<label>Nom de famille <input id="nom" name="nom"></label>
<input id="mail" name="user_email" type="email">
<input id="tel" placeholder="Téléphone portable">
<input id="cp" name="code_postal"><input id="ville" name="ville" value="Déjà saisi">
<select id="pays" name="pays"><option value="">--</option><option value="FR">France</option><option value="BE">Belgique</option></select>
<input id="mdp" type="password" name="password"><input id="carte" name="cardnumber" autocomplete="cc-number">
</form>
HTML
python3 tools/serveur-test.py "$PORT" "$W" >/dev/null 2>&1 & SERVEUR=$!
export ECHO_RUN_DIR="$W/run" ECHO_DATA_DIR="$W/data" ECHO_CONTROL_NAME="formulaires-$$" ECHO_NO_WELCOME=1 W PORT
export ECHO_DEVTOOLS_PORT=$((30000 + RANDOM % 9000))
mkdir -p "$W/run"
trap './stop.sh >/dev/null 2>&1 || true; kill $SERVEUR 2>/dev/null || true; rm -rf "$W"' EXIT
./start.sh release >/dev/null; sleep 8
uv run -q --with websocket-client python - <<'PY'
import json, os, socket, sqlite3, time, urllib.request, websocket
W, PORT, DT = os.environ["W"], os.environ["PORT"], os.environ["ECHO_DEVTOOLS_PORT"]
page = f"http://localhost:{PORT}/form.html"
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
ev = lambda prefix, js: cdp(prefix, "Runtime.evaluate", {"expression": js, "returnByValue": True})["result"].get("value")
def until(cond, label, tries=30):
    for _ in range(tries):
        try:
            if cond(): return
        except Exception:
            pass
        time.sleep(0.5)
    raise AssertionError(label)
def cards():
    con = sqlite3.connect(f"file:{W}/data/library.db?mode=ro", uri=True)
    try:
        row = con.execute("select value from settings where key='forms.cards'").fetchone()
        return json.loads(row[0][2:]) if row else []
    finally: con.close()
REG = "echo://ui/pages.html#reglages"
call(op="ui", request={"kind": "openPage", "page": "reglages"}); time.sleep(3)
ev(REG, "[...document.querySelectorAll('button')].find(b=>b.innerText.trim()==='Nouvelle fiche').click()")
until(lambda: len(cards()) == 1, "« Nouvelle fiche » n'enregistre rien")
# Le sway de test n'a pas de clavier : la sortie du champ est simulee par l'evenement que React ecoute.
def saisir(label, value):
    ev(REG, f"(()=>{{const i=document.querySelector('[data-fiche] input[aria-label=\"{label}\"]');"
            f"Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(i,{json.dumps(value)});"
            "i.dispatchEvent(new Event('input',{bubbles:true}));i.dispatchEvent(new FocusEvent('focusout',{bubbles:true}));return 1})()")
    time.sleep(0.8)
for label, value in [("Prénom", "Ada"), ("Nom", "Lovelace"), ("E-mail", "ada@example.com"), ("Téléphone", "0600000000"),
                     ("Code postal", "75001"), ("Ville", "Paris"), ("Pays", "France")]:
    saisir(label, value)
until(lambda: cards()[0]["fields"].get("country") == "France", f"fiche incomplete : {cards()}")
print(f"Reglages : fiche « {cards()[0]['name']} » enregistree ({len(cards()[0]['fields'])} champs)")
call(op="open", url=page); time.sleep(3)
for kind in ("mousePressed", "mouseReleased"):
    cdp(page, "Input.dispatchMouseEvent", {"type": kind, "x": 60, "y": 30, "button": "right", "clickCount": 1})
time.sleep(1.2)
menu = ev("echo://ui/menu", "document.body.innerText")
assert "Remplir : Personnel" in menu, f"entree absente du clic droit : {menu[:300]}"
call(op="ui", request={"kind": "runContextMenu", "action": "fillForm1"}); time.sleep(1.5)
got = ev(page, "Object.fromEntries([...document.querySelectorAll('input,select')].map(e=>[e.id,e.value]))")
want = {"prenom": "Ada", "nom": "Lovelace", "mail": "ada@example.com", "tel": "0600000000", "cp": "75001",
        "ville": "Déjà saisi", "pays": "FR", "mdp": "", "carte": ""}
assert got == want, f"remplissage inattendu : {got}"
print("OK : formulaires (fiche creee dans les Reglages, clic droit → remplir, saisie gardee, mot de passe et carte intacts)")
PY
