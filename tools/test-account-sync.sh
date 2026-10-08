#!/usr/bin/env bash
# Compte Echo de bout en bout : deux instances isolees (deux « machines ») sur un service de compte (local par defaut,
# `cd compte && pnpm dev`). A : premier lancement → presentation ; compte cree ; favori ajoute. B : connexion au meme
# compte → recoit le favori ; change le moteur de recherche → A le recoit. Usage : COMPTE_URL=… tools/test-account-sync.sh
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_SYNC_URL="${COMPTE_URL:-http://127.0.0.1:8788}"
curl -sf "$ECHO_SYNC_URL/v1/sante" >/dev/null || { echo "service de compte injoignable : $ECHO_SYNC_URL" >&2; exit 1; }
A="$(mktemp -d)"; B="$(mktemp -d)"
start() { # dossier nom
  ECHO_RUN_DIR="$1/run" ECHO_DATA_DIR="$1/data" ECHO_CONTROL_NAME="$2" ./start.sh release >/dev/null
}
stop() { ECHO_RUN_DIR="$1/run" ./stop.sh >/dev/null 2>&1 || true; }
trap 'stop "$A"; stop "$B"' EXIT
mkdir -p "$A/run" "$B/run"
start "$A" "compte-a-$$"; start "$B" "compte-b-$$"; sleep 14
python3 - "$A" "$B" "compte-a-$$" "compte-b-$$" <<'PY'
import json, os, socket, sqlite3, sys, time
A, B, NA, NB = sys.argv[1:5]
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
b(op="ui", request={"kind": "updateSetting", "key": "search.engine", "value": {"type": "text", "value": "duckduckgo"}})
time.sleep(1); b(op="ui", request={"kind": "accountSync"}); time.sleep(8)
a(op="ui", request={"kind": "accountSync"})
until(lambda: ("s:duckduckgo",) in db(A, "select value from settings where key='search.engine'"), "le reglage de B n'arrive pas sur A")
print("reglage de B recu par A")
compte = json.load(open(f"{A}/data/compte.json"))
assert oct(os.stat(f"{A}/data/compte.json").st_mode & 0o777) == "0o600", "fichier du compte lisible par d'autres"
a(op="ui", request={"kind": "accountSignOut"}); time.sleep(1)
assert not os.path.exists(f"{A}/data/compte.json"), "deconnexion sans effet"
print("OK : compte Echo synchronise entre deux machines (favori, reglage), deconnexion")
PY
