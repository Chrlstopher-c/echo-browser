#!/usr/bin/env bash
# Mise a jour automatique, de bout en bout, avec un faux serveur de releases (forme de l'API GitHub) :
#  1. archive alteree → refusee, rien n'est prepare ;
#  2. version suivante → preparee, bandeau « pret », Redemarrer → nouvelle version en place, onglet garde ;
#  3. version cassee → le lanceur revient seul a la precedente apres deux demarrages rates.
# Usage : tools/test-update.sh [archive .tar.xz] (defaut : la plus recente de dist-release/). Instance isolee.
set -euo pipefail
cd "$(dirname "$0")/.."
ARCHIVE="$(readlink -f "${1:-$(ls -t dist-release/*.tar.xz | head -1)}")"
WORK="$(mktemp -d)"; SRV="$WORK/srv"; mkdir -p "$SRV" "$WORK/opt"
export ECHO_RUN_DIR="$WORK/run" ECHO_CONTROL_NAME="test-$$" ECHO_DATA_DIR="$WORK/data" ECHO_NO_ANCHOR=1
export ECHO_DEVTOOLS_PORT=$((30000 + RANDOM % 20000)) ECHO_UPDATE_DELAY_S=3
mkdir -p "$ECHO_RUN_DIR" "$ECHO_DATA_DIR"
tar -xf "$ARCHIVE" -C "$WORK/opt" && mv "$WORK"/opt/echo-browser-* "$WORK/opt/echo-browser"
INSTALL="$WORK/opt/echo-browser"
CURRENT=$(python3 -c "import json,sys;print(json.load(open(sys.argv[1]))['version'])" "$INSTALL/release.json")
NEXT=$(python3 -c "import sys;a=sys.argv[1].split('.');a[-1]=str(int(a[-1])+1);print('.'.join(a))" "$CURRENT")
# Version suivante : la meme archive, annoncee plus recente.
mkdir -p "$WORK/b" && tar -xf "$ARCHIVE" -C "$WORK/b" && mv "$WORK"/b/echo-browser-* "$WORK/b/echo-browser-$NEXT-linux-x64"
python3 -c "import json,sys;p=sys.argv[1];d=json.load(open(p));d['version']=sys.argv[2];json.dump(d,open(p,'w'))" \
  "$WORK/b/echo-browser-$NEXT-linux-x64/release.json" "$NEXT"
tar -C "$WORK/b" -cJf "$SRV/b.tar.xz" "echo-browser-$NEXT-linux-x64"
(cd "$SRV" && sha256sum b.tar.xz | sed "s/ b.tar.xz/ echo-browser-$NEXT-linux-x64.tar.xz/" > b.sha256)
echo alteree > "$SRV/mode"
python3 - "$SRV" "$NEXT" <<'PYS' > "$WORK/srv.log" 2>&1 &
import hashlib, http.server, json, os, socketserver, sys
srv, nxt = sys.argv[1], sys.argv[2]
class H(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *a, **k): super().__init__(*a, directory=srv, **k)
    def do_GET(self):
        if self.path != "/latest": return super().do_GET()
        port = self.server.server_address[1]
        digest = open(os.path.join(srv, "b.sha256")).read().split()[0]
        if open(os.path.join(srv, "mode")).read().strip() == "alteree": digest = "0" * 64
        name = f"echo-browser-{nxt}-linux-x64.tar.xz"
        body = json.dumps({"tag_name": f"v{nxt}", "assets": [
            {"name": name, "browser_download_url": f"http://127.0.0.1:{port}/b.tar.xz", "digest": "sha256:" + digest},
            {"name": name + ".sha256", "browser_download_url": f"http://127.0.0.1:{port}/b.sha256"}]}).encode()
        self.send_response(200); self.send_header("Content-Type", "application/json"); self.end_headers(); self.wfile.write(body)
    def log_message(self, *a): pass
s = socketserver.ThreadingTCPServer(("127.0.0.1", 0), H); open(os.path.join(srv, "port"), "w").write(str(s.server_address[1]))
s.serve_forever()
PYS
SERVER=$!
sleep 1; export ECHO_UPDATE_URL="http://127.0.0.1:$(cat "$SRV/port")/latest"
BROWSER=""
launch() { setsid "$INSTALL/echo-browser.sh" >>"$WORK/log.txt" 2>&1 & BROWSER=$!; }
cleanup() { [ -n "$BROWSER" ] && kill -- -"$BROWSER" 2>/dev/null; kill $SERVER 2>/dev/null; true; }
trap cleanup EXIT
launch; sleep 14
CURRENT="$CURRENT" NEXT="$NEXT" INSTALL="$INSTALL" SRV="$SRV" uv run -q --with websocket-client python - <<'PY'
import json, os, socket, time, urllib.request, websocket
def connect():
    for _ in range(80):
        try:
            s = socket.socket(socket.AF_UNIX)
            s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
            return s.makefile("rw")
        except OSError:
            time.sleep(0.5)
    raise AssertionError("prise de controle absente")
f = connect()
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
def ui(js):
    for _ in range(40):
        try:
            t = [t for t in json.load(urllib.request.urlopen(f"http://127.0.0.1:{os.environ['ECHO_DEVTOOLS_PORT']}/json/list"))
                 if t["url"].startswith("echo://ui/index.html")][0]; break
        except Exception: time.sleep(0.5)
    ws = websocket.create_connection(t["webSocketDebuggerUrl"], suppress_origin=True)
    ws.send(json.dumps({"id": 1, "method": "Runtime.evaluate", "params": {"expression": js, "returnByValue": True}}))
    while (m := json.loads(ws.recv())).get("id") != 1: pass
    ws.close(); return m["result"]["result"].get("value")
STRIP = "document.querySelector('[aria-label=\"Mise à jour\"]')"
staged = os.environ["INSTALL"] + ".maj"
time.sleep(10)
assert not os.path.exists(staged), "archive alteree preparee quand meme"
print("1. archive alteree refusee")
open(os.path.join(os.environ["SRV"], "mode"), "w").write("bonne")
call(op="open", url="https://example.org/")
call(op="ui", request={"kind": "checkForUpdates"})
for _ in range(120):
    if ui(f"{STRIP} ? {STRIP}.innerText : ''"): break
    time.sleep(0.5)
text = ui(f"{STRIP} ? {STRIP}.innerText : ''") or ""
assert os.environ["NEXT"] in text and os.path.isdir(staged), f"pas de bandeau de mise a jour : {text!r}"
assert ui(f"(()=>{{const b=[...{STRIP}.querySelectorAll('button')].find(b=>b.innerText.trim()==='Redémarrer');b.click();return !!b}})()")
time.sleep(18)
f = connect()
version = json.load(open(os.path.join(os.environ["INSTALL"], "release.json")))["version"]
assert version == os.environ["NEXT"], f"version installee : {version}"
assert os.path.isdir(os.environ["INSTALL"] + ".precedent"), "ancienne version non gardee"
assert any(t["url"] == "https://example.org/" for t in call(op="tabs")["tabs"]), "onglet perdu"
print(f"2. mise a jour {os.environ['CURRENT']} → {version}, onglet garde")
PY
# 3. Version cassee : le lanceur doit revenir seul a la precedente.
kill -- -"$BROWSER" 2>/dev/null; BROWSER=""; sleep 3
cp -a "$INSTALL" "$INSTALL.maj"
printf '#!/bin/sh\nexit 1\n' > "$INSTALL.maj/echo-browser"
python3 -c "import json,sys;p=sys.argv[1];d=json.load(open(p));d['version']='99.0.0';json.dump(d,open(p,'w'))" "$INSTALL.maj/release.json"
for i in 1 2; do "$INSTALL/echo-browser.sh" >/dev/null 2>&1 || true; done
launch; sleep 14
python3 - "$INSTALL" "$NEXT" <<'PY'
import json, os, socket, sys
version = json.load(open(os.path.join(sys.argv[1], "release.json")))["version"]
assert version == sys.argv[2], f"pas de retour a la version precedente : {version}"
s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
print(f"3. version cassee ecartee, retour a {version}")
PY
echo "OK : mise a jour automatique (refus d'archive alteree, installation au redemarrage, retour arriere)"
