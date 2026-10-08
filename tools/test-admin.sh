#!/usr/bin/env bash
# Administration dans Echo : un compte ordinaire ne voit pas la section ; rendu administrateur (cle du service), il la
# voit apres synchronisation, lit les chiffres et deconnecte un autre compte. Service local (`cd compte && pnpm dev`,
# `ADMIN_KEY` dans compte/.dev.vars). Instance isolee. Usage : tools/test-admin.sh
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_SYNC_URL="${COMPTE_URL:-http://127.0.0.1:8788}" ECHO_NO_WELCOME=1
export ADMIN_KEY="${ADMIN_KEY:-$(grep ADMIN_KEY compte/.dev.vars | cut -d= -f2)}"
curl -sf "$ECHO_SYNC_URL/v1/sante" >/dev/null || { echo "service de compte injoignable : $ECHO_SYNC_URL" >&2; exit 1; }
D="$(mktemp -d)"; mkdir -p "$D/run"; P=$((30000 + RANDOM % 10000))
export ECHO_RUN_DIR="$D/run" ECHO_DATA_DIR="$D/data" ECHO_CONTROL_NAME="admin-$$" ECHO_DEVTOOLS_PORT=$P
trap './stop.sh >/dev/null 2>&1 || true' EXIT
./start.sh release >/dev/null; sleep 12
uv run -q --with websocket-client python - "$D" "$P" <<'PY'
import json, os, socket, sys, time, urllib.request, websocket
D, P = sys.argv[1:3]
URL, KEY = os.environ["ECHO_SYNC_URL"], os.environ["ADMIN_KEY"]
def http(method, path, body=None, token=KEY):
    r = urllib.request.Request(URL + path, method=method, data=None if body is None else json.dumps(body).encode(),
        headers={"authorization": f"Bearer {token}", "content-type": "application/json"})
    with urllib.request.urlopen(r) as res: return json.loads(res.read() or b"null")
def page(js):
    t = [t for t in json.load(urllib.request.urlopen(f"http://127.0.0.1:{P}/json/list")) if "pages.html" in t["url"]][0]
    ws = websocket.create_connection(t["webSocketDebuggerUrl"], suppress_origin=True)
    ws.send(json.dumps({"id": 1, "method": "Runtime.evaluate", "params": {"expression": js, "returnByValue": True}}))
    while (m := json.loads(ws.recv())).get("id") != 1: pass
    ws.close(); return m["result"]["result"].get("value")
s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = s.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
def until(cond, label, tries=60):
    for _ in range(tries):
        if cond(): return
        time.sleep(0.5)
    raise AssertionError(label)
email = f"admin-{os.getpid()}@exemple.org"
call(op="ui", request={"kind": "accountSignIn", "email": email, "password": "mot de passe admin 123", "create": True})
until(lambda: os.path.exists(f"{D}/data/compte.json"), "compte non cree"); time.sleep(4)
call(op="ui", request={"kind": "openPage", "page": "reglages"}); time.sleep(3)
texte = lambda: page("document.body.innerText")
assert "Administration" not in texte(), "section visible pour un compte ordinaire"
print("compte ordinaire : pas d'administration")
moi = http("GET", f"/v1/admin/comptes?q={email}")["comptes"][0]
http("POST", f"/v1/admin/comptes/{moi['id']}/admin", {"admin": True})
autre = f"autre-{os.getpid()}@exemple.org"
import base64, secrets
http("POST", "/v1/inscription", {"email": autre, "cleAcces": base64.b64encode(secrets.token_bytes(32)).decode(),
     "sel": base64.b64encode(secrets.token_bytes(16)).decode(), "iterations": 600000}, token="")
call(op="ui", request={"kind": "accountSync"})
until(lambda: "Administration" in texte(), "section absente apres le drapeau admin")
call(op="ui", request={"kind": "openPage", "page": "admin"})
until(lambda: "Coffre chiffré" in texte() and autre in texte(), "tableau de bord vide")
print("administrateur : section, chiffres et comptes visibles")
clic = ("(()=>{const r=[...document.querySelectorAll('div')].filter(d=>d.innerText.startsWith('%s')&&d.querySelector('button'))"
        ".pop();const b=[...r.querySelectorAll('button')].find(x=>x.innerText.trim()==='Déconnecter');b.click();return 1})()") % autre
assert page(clic) == 1
until(lambda: http("GET", f"/v1/admin/comptes?q={autre}")["comptes"][0]["sessions"] == 0, "deconnexion depuis Echo sans effet")
print("OK : administration dans Echo (cachee aux comptes ordinaires, chiffres, deconnexion d'un compte)")
PY
