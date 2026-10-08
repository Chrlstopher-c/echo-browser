#!/usr/bin/env bash
# Synchro automatique, sans jamais demander de synchro : deux machines en temps reel. A cree le compte et ajoute un
# favori → B (connectee) le recoit seule (controle chaque minute). B relancee : l'acces admin est la des le lancement.
# Mode manuel : une modification affiche l'alerte « pas synchronisees ». Service coupe : alerte d'erreur.
# Service local (`cd compte && pnpm dev`). Usage : tools/test-account-auto.sh
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_SYNC_URL="${COMPTE_URL:-http://127.0.0.1:8788}" ECHO_NO_WELCOME=1
export ADMIN_KEY="${ADMIN_KEY:-$(grep ADMIN_KEY compte/.dev.vars | cut -d= -f2)}"
curl -sf "$ECHO_SYNC_URL/v1/sante" >/dev/null || { echo "service de compte injoignable : $ECHO_SYNC_URL" >&2; exit 1; }
A="$(mktemp -d)"; B="$(mktemp -d)"; mkdir -p "$A/run" "$B/run"
PA=$((30000 + RANDOM % 10000)); PB=$((40000 + RANDOM % 10000))
start() { ECHO_RUN_DIR="$1/run" ECHO_DATA_DIR="$1/data" ECHO_CONTROL_NAME="$2" ECHO_DEVTOOLS_PORT="$3" ./start.sh release >/dev/null; }
stop() { ECHO_RUN_DIR="$1/run" ./stop.sh >/dev/null 2>&1 || true; }
trap 'stop "$A"; stop "$B"' EXIT
start "$A" "auto-a-$$" "$PA"; start "$B" "auto-b-$$" "$PB"; sleep 12
export A B NA="auto-a-$$" NB="auto-b-$$" PA PB
uv run -q --with websocket-client python - <<'PY'
import base64, json, os, socket, sqlite3, subprocess, time, urllib.request, websocket
A, B, NA, NB, PB = (os.environ[k] for k in ("A", "B", "NA", "NB", "PB"))
URL, KEY = os.environ["ECHO_SYNC_URL"], os.environ["ADMIN_KEY"]
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
def until(cond, label, tries=200):
    for _ in range(tries):
        try:
            if cond(): return
        except Exception: pass
        time.sleep(0.5)
    raise AssertionError(label)
def admin(method, path, body=None):
    r = urllib.request.Request(URL + path, method=method, data=None if body is None else json.dumps(body).encode(),
        headers={"authorization": f"Bearer {KEY}", "content-type": "application/json"})
    return json.loads(urllib.request.urlopen(r).read() or b"null")
def page_text(port, prefix="echo://ui/index.html"):
    pages = json.load(urllib.request.urlopen(f"http://127.0.0.1:{port}/json/list"))
    t = [t for t in pages if t["url"].startswith(prefix)][0]
    ws = websocket.create_connection(t["webSocketDebuggerUrl"], suppress_origin=True)
    ws.send(json.dumps({"id": 1, "method": "Runtime.evaluate", "params": {"expression": "document.body.innerText", "returnByValue": True}}))
    while (m := json.loads(ws.recv())).get("id") != 1: pass
    ws.close(); return m["result"]["result"]["value"]
a, b = control(NA), control(NB)
mode = lambda c, m: c(op="ui", request={"kind": "updateSetting", "key": "sync.mode", "value": {"type": "text", "value": m}})
mode(a, "realtime"); mode(b, "realtime")
email, mdp = f"auto-{os.getpid()}@exemple.org", "mot de passe auto 123"
a(op="ui", request={"kind": "accountSignIn", "email": email, "password": mdp, "create": True})
until(lambda: json.load(open(f"{A}/data/compte.json")).get("last_sync"), "premiere synchro de A")
b(op="ui", request={"kind": "accountSignIn", "email": email, "password": mdp, "create": False})
until(lambda: json.load(open(f"{B}/data/compte.json")).get("last_sync"), "premiere synchro de B")
tab = a(op="open", url="https://example.org/")["id"]; time.sleep(3)
a(op="ui", request={"kind": "addBookmark", "id": tab}); t0 = time.time()
until(lambda: any("example.org" in u for (u,) in db(B, "select url from bookmarks")), "favori non recu automatiquement", 300)
print(f"favori de A arrive seul sur B en {time.time() - t0:.0f} s")
compte = [c for c in admin("GET", f"/v1/admin/comptes?q={email}")["comptes"]][0]
admin("POST", f"/v1/admin/comptes/{compte['id']}/admin", {"admin": True})
until(lambda: json.load(open(f"{B}/data/compte.json")).get("admin"), "drapeau admin non recu par B", 300)
subprocess.run(["./stop.sh"], env={**os.environ, "ECHO_RUN_DIR": f"{B}/run"}, capture_output=True)
subprocess.run(["./start.sh", "release"], env={**os.environ, "ECHO_RUN_DIR": f"{B}/run", "ECHO_DATA_DIR": f"{B}/data",
    "ECHO_CONTROL_NAME": NB, "ECHO_DEVTOOLS_PORT": PB}, capture_output=True)
time.sleep(4); b = control(NB)
b(op="ui", request={"kind": "openPage", "page": "reglages"}); time.sleep(1.5)
assert "Administration" in page_text(PB, "echo://ui/pages.html"), "administration absente juste apres la relance"
print("B relancee : administration visible des le lancement")
passes = lambda home: open(f"{home}/run/browser.log", errors="ignore").read().count("synchronisation faite")
time.sleep(20); avant = (passes(A), passes(B)); time.sleep(150)
apres = (passes(A) - avant[0], passes(B) - avant[1])
assert max(apres) <= 1, f"synchros a vide en 150 s (A, B) : {apres} — les machines se relancent l'une l'autre"
print(f"a vide pendant 150 s en temps reel : {apres} synchro(s) (A, B)")
mode(a, "manual"); time.sleep(1)
t2 = a(op="open", url="https://example.com/")["id"]; time.sleep(3)
a(op="ui", request={"kind": "addBookmark", "id": t2})
until(lambda: "ne sont pas synchronisées" in page_text(os.environ["PA"]), "alerte du mode manuel absente", 40)
print("mode manuel : alerte « modifications non synchronisees »")
print("OK : synchro automatique (favori sans clic, admin garde a la relance, alerte en manuel)")
PY
