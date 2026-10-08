#!/usr/bin/env bash
# Compte Echo de bout en bout : deux instances isolees (deux « machines ») sur un service de compte (local par defaut,
# `cd compte && pnpm dev`). A : premier lancement → presentation ; compte cree ; favori ajoute. B : connexion au meme
# compte → recoit le favori et l'historique ; change le moteur de recherche → A le recoit ; efface une visite → A
# l'efface ; montre les donnees du serveur dans les Reglages ; supprime le compte → plus de connexion possible. Usage : COMPTE_URL=… tools/test-account-sync.sh
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_SYNC_URL="${COMPTE_URL:-http://127.0.0.1:8788}"
curl -sf "$ECHO_SYNC_URL/v1/sante" >/dev/null || { echo "service de compte injoignable : $ECHO_SYNC_URL" >&2; exit 1; }
A="$(mktemp -d)"; B="$(mktemp -d)"
start() { # dossier nom port
  ECHO_RUN_DIR="$1/run" ECHO_DATA_DIR="$1/data" ECHO_CONTROL_NAME="$2" ECHO_DEVTOOLS_PORT="$3" ./start.sh release >/dev/null
}
PA=$((30000 + RANDOM % 10000)); PB=$((40000 + RANDOM % 10000))
stop() { ECHO_RUN_DIR="$1/run" ./stop.sh >/dev/null 2>&1 || true; }
trap 'stop "$A"; stop "$B"' EXIT
mkdir -p "$A/run" "$B/run"
start "$A" "compte-a-$$" "$PA"; start "$B" "compte-b-$$" "$PB"; sleep 14
uv run -q --with websocket-client python - "$A" "$B" "compte-a-$$" "compte-b-$$" "$PA" "$PB" <<'PY'
import json, os, socket, sqlite3, sys, time, urllib.request, websocket
A, B, NA, NB, PA, PB = sys.argv[1:7]
def page_eval(port, prefix, js):
    t = [t for t in json.load(urllib.request.urlopen(f"http://127.0.0.1:{port}/json/list")) if t["url"].startswith(prefix)][0]
    ws = websocket.create_connection(t["webSocketDebuggerUrl"], suppress_origin=True)
    ws.send(json.dumps({"id": 1, "method": "Runtime.evaluate", "params": {"expression": js, "returnByValue": True}}))
    while (m := json.loads(ws.recv())).get("id") != 1: pass
    ws.close(); return m["result"]["result"].get("value")
MACHINES = ("(()=>{const b=[...document.querySelectorAll('button,[role=radio],label')].find(x=>x.innerText.trim()==='Machines');"
            "if(!b)return 'absent';b.click();return 'ok'})()")
def control(name):
    s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{name}.sock")
    f = s.makefile("rw")
    def call(**r):
        f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
    return call
def db(home, sql):
    con = sqlite3.connect(f"file:{home}/data/library.db?mode=ro", uri=True)
    try: return con.execute(sql).fetchall()
    finally: con.close()
def until(cond, label, tries=120):
    for _ in range(tries):
        if cond(): return
        time.sleep(0.5)
    raise AssertionError(label)
a, b = control(NA), control(NB)
assert any("pages.html#bienvenue" in t["url"] for t in a(op="tabs")["tabs"]), "premier lancement sans presentation"
email, mdp = f"essai-echo-{os.getpid()}@exemple.org", "mot de passe de test 123"
a(op="ui", request={"kind": "accountSignIn", "email": email, "password": mdp, "create": True})
until(lambda: os.path.exists(f"{A}/data/compte.json"), "compte non cree sur A")
tab = a(op="open", url="https://example.org/")["id"]; time.sleep(3)
a(op="ui", request={"kind": "addBookmark", "id": tab}); time.sleep(1)
until(lambda: json.load(open(f"{A}/data/compte.json")).get("last_sync"), "premiere synchro de A")
a(op="ui", request={"kind": "accountSync"}); time.sleep(8)
b(op="ui", request={"kind": "accountSignIn", "email": email, "password": mdp, "create": False})
until(lambda: any("example.org" in u for (u,) in db(B, "select url from bookmarks")), "le favori de A n'arrive pas sur B", 160)
print("favori de A recu par B")
b(op="ui", request={"kind": "openPage", "page": "bibliotheque"}); time.sleep(3)
if page_eval(PB, "echo://ui/pages.html", MACHINES) != "ok":
    onglets = json.load(open(f"{B}/data/compte.json")).get("kinds", {}).get("onglets")
    raise AssertionError(f"section Machines absente sur B ; onglets connus de B : {json.dumps(onglets)[:600]}")
time.sleep(1)
texte = page_eval(PB, "echo://ui/pages.html", "document.body.innerText")
assert "Example Domain" in texte, f"onglet de A absent des Machines de B : {texte[:300]!r}"
print("onglets de A visibles sur B (Bibliotheque → Machines)")
b(op="ui", request={"kind": "updateSetting", "key": "search.engine", "value": {"type": "text", "value": "duckduckgo"}})
time.sleep(1); b(op="ui", request={"kind": "accountSync"}); time.sleep(8)
a(op="ui", request={"kind": "accountSync"})
until(lambda: ("s:duckduckgo",) in db(A, "select value from settings where key='search.engine'"), "le reglage de B n'arrive pas sur A")
print("reglage de B recu par A")
def has_visit(home): return any("example.org" in u for (u,) in db(home, "select url from history"))
until(lambda: has_visit(B), "l'historique de A n'arrive pas sur B")
print("historique de A recu par B")
(url, seen), = db(B, "select url, max(visited_at) from history where url like '%example.org%'")
b(op="ui", request={"kind": "removeHistoryEntry", "url": url, "visitedAt": seen}); time.sleep(1)
b(op="ui", request={"kind": "accountSync"}); time.sleep(8)
a(op="ui", request={"kind": "accountSync"})
until(lambda: not has_visit(A), "la visite effacee sur B reste sur A")
print("visite effacee sur B, effacee sur A")
b(op="ui", request={"kind": "openPage", "page": "reglages"}); time.sleep(3)
AFFICHER = ("(()=>{const b=[...document.querySelectorAll('button')].find(x=>x.innerText.trim()==='Afficher');"
            "if(!b)return 'absent';b.click();return 'ok'})()")
assert page_eval(PB, "echo://ui/pages.html", AFFICHER) == "ok", "bouton des donnees stockees absent"
until(lambda: all(k in page_eval(PB, "echo://ui/pages.html", "document.body.innerText")
                  for k in ("Favoris", "Historique", "Réglages", "chiffrés")), "donnees du serveur non montrees")
print("donnees du serveur montrees dans les Reglages")
a(op="ui", request={"kind": "openPage", "page": "bibliotheque"}); time.sleep(3)
assert page_eval(PA, "echo://ui/pages.html", MACHINES) == "absent", "A se voit elle-meme dans Machines"
print("A ne se voit pas elle-meme")
compte = json.load(open(f"{A}/data/compte.json"))
assert oct(os.stat(f"{A}/data/compte.json").st_mode & 0o777) == "0o600", "fichier du compte lisible par d'autres"
a(op="ui", request={"kind": "accountSignOut"}); time.sleep(1)
assert not os.path.exists(f"{A}/data/compte.json"), "deconnexion sans effet"
b(op="ui", request={"kind": "accountDelete"})
until(lambda: not os.path.exists(f"{B}/data/compte.json"), "suppression du compte sans effet sur B")
b(op="ui", request={"kind": "accountSignIn", "email": email, "password": mdp, "create": False}); time.sleep(8)
assert not os.path.exists(f"{B}/data/compte.json"), "connexion encore possible apres suppression"
print("compte supprime : plus de connexion possible")
print("OK : compte Echo synchronise entre deux machines (favori, historique, reglage), donnees montrees, deconnexion, suppression")
PY
