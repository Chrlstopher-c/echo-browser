#!/usr/bin/env bash
# Signaux anonymes : coupes par defaut → rien n'est compte ni envoye ; actives → le jour fini part en un lot (vu par le
# service), puis est efface sur la machine ; seul, il ne depasse pas le seuil k (aucun domaine montre). Service local
# (`cd compte && pnpm dev`). Instance isolee ; les comptes sont ranges sous « hier » pour partir tout de suite.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_SYNC_URL="${COMPTE_URL:-http://127.0.0.1:8788}" ECHO_NO_WELCOME=1 ECHO_SIGNALS_SHIFT_DAYS=1 ECHO_SIGNALS_SEND_S=5
export ADMIN_KEY="${ADMIN_KEY:-$(grep ADMIN_KEY compte/.dev.vars | cut -d= -f2)}"
curl -sf "$ECHO_SYNC_URL/v1/sante" >/dev/null || { echo "service de compte injoignable : $ECHO_SYNC_URL" >&2; exit 1; }
W="$(mktemp -d)"; PORT=$((28000 + RANDOM % 2000))
echo '<!doctype html><title>signal</title><p>x</p>' > "$W/index.html"
python3 tools/serveur-test.py "$PORT" "$W" >/dev/null 2>&1 & SERVEUR=$!
export ECHO_RUN_DIR="$W/run" ECHO_DATA_DIR="$W/data" ECHO_CONTROL_NAME="signaux-$$" ECHO_DEVTOOLS_PORT=$((30000 + RANDOM % 9000))
export W PORT
mkdir -p "$W/run"
trap './stop.sh >/dev/null 2>&1 || true; kill $SERVEUR 2>/dev/null || true' EXIT
./start.sh release >/dev/null; sleep 8
uv run -q python - <<'PY'
import json, os, socket, sqlite3, time, urllib.request
W, PORT, URL, KEY = os.environ["W"], os.environ["PORT"], os.environ["ECHO_SYNC_URL"], os.environ["ADMIN_KEY"]
s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = s.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
def lots():
    r = urllib.request.Request(URL + "/v1/admin/signaux", headers={"authorization": f"Bearer {KEY}"})
    vue = json.loads(urllib.request.urlopen(r).read())
    return sum(l["n"] for l in vue["lotsParJour"]), vue
def waiting():
    con = sqlite3.connect(f"file:{W}/data/library.db?mode=ro", uri=True)
    try: return con.execute("select count(*) from signals_day").fetchone()[0]
    finally: con.close()
def visit():
    tab = call(op="open", url=f"http://localhost:{PORT}/")["id"]; time.sleep(2)
    call(op="navigate", id=tab, url=f"http://127.0.0.1:{PORT}/"); time.sleep(2)
avant, _ = lots()
visit(); time.sleep(12)
assert lots()[0] == avant and waiting() == 0, "signaux comptes ou envoyes sans accord"
print("coupe par defaut : rien compte, rien envoye")
call(op="ui", request={"kind": "updateSetting", "key": "signals.share", "value": {"type": "flag", "value": True}})
time.sleep(1); visit()
for _ in range(40):
    if lots()[0] == avant + 1: break
    time.sleep(0.5)
else:
    raise AssertionError("lot non recu par le service")
time.sleep(1.5)
assert waiting() == 0, "jour envoye mais pas efface sur la machine"
_, vue = lots()
assert vue["seuil"] <= 1 or not any(s["cle"] in ("localhost", "127.0.0.1") for s in vue["sites"]), "un envoi seul depasse k"
print(f"active : lot recu, efface localement, invisible sous le seuil k={vue['seuil']}")
call(op="ui", request={"kind": "updateSetting", "key": "signals.share", "value": {"type": "flag", "value": False}})
visit(); time.sleep(1)
assert waiting() == 0, "coupe a nouveau mais encore compte"
print("OK : signaux anonymes (rien sans accord, lot du jour fini envoye puis efface, seuil k respecte)")
PY
