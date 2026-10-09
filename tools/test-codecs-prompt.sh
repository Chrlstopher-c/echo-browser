#!/usr/bin/env bash
# Release publique, parcours d'un utilisateur : une video H.264 + AAC ne se lit pas → Echo propose le decodeur dans la
# barre → « Installer » → « Redemarrer » → la video se lit, onglets gardes. Clics joues dans l'interface (protocole de
# debogage). Usage : tools/test-codecs-prompt.sh [archive .tar.xz] (defaut : dist-release/, la plus recente).
set -euo pipefail
cd "$(dirname "$0")/.."
ARCHIVE="${1:-$(ls -t dist-release/*.tar.xz | head -1)}"
WORK="$(mktemp -d)"
export ECHO_RUN_DIR="$WORK/run" ECHO_CONTROL_NAME="test-$$" ECHO_DATA_DIR="$WORK/data" ECHO_HWDEC=0
export ECHO_DEVTOOLS_PORT=$((30000 + RANDOM % 20000))
mkdir -p "$ECHO_RUN_DIR" "$ECHO_DATA_DIR" && tar -xf "$ARCHIVE" -C "$WORK"
ffmpeg -loglevel error -f lavfi -i testsrc=d=30:s=640x360:r=30 -f lavfi -i sine=f=440:d=30 \
  -c:v libx264 -pix_fmt yuv420p -c:a aac -shortest "$ECHO_RUN_DIR/va.mp4"
cat > "$ECHO_RUN_DIR/h.html" <<'H'
<title>h</title><p id=p></p><video id=b muted src=va.mp4></video><script>b.play().catch(()=>{});
setInterval(()=>p.textContent=`t=${b.currentTime.toFixed(1)} err=${b.error?b.error.code:0}`,300)</script>
H
python3 -m http.server 0 --bind 127.0.0.1 -d "$ECHO_RUN_DIR" >/dev/null 2>&1 & SERVER=$!
setsid "$WORK"/echo-browser-*/echo-browser.sh >"$WORK/log.txt" 2>&1 & BROWSER=$!
trap 'kill -- -$BROWSER 2>/dev/null || true; kill $SERVER 2>/dev/null || true' EXIT
sleep 12
PORT=$(ss -ltnp | grep "pid=$SERVER," | grep -oP '127.0.0.1:\K\d+') uv run -q --with websocket-client python - <<'PY'
import json, os, re, socket, time, urllib.request, websocket
def connect():
    for _ in range(60):
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
                 if t["url"].startswith("echo://ui/index.html")][0]
            break
        except Exception:
            time.sleep(0.5)
    ws = websocket.create_connection(t["webSocketDebuggerUrl"], suppress_origin=True)
    ws.send(json.dumps({"id": 1, "method": "Runtime.evaluate", "params": {"expression": js, "returnByValue": True}}))
    while (m := json.loads(ws.recv())).get("id") != 1: pass
    ws.close(); return m["result"]["result"].get("value")
STRIP = "document.querySelector('[aria-label=\"Décodeur vidéo\"]')"
def strip_text():
    return ui(f"{STRIP} ? {STRIP}.innerText : ''") or ""
def press(label):
    return ui(f"(()=>{{const b=[...({STRIP}||document).querySelectorAll('button')].find(b=>b.innerText.trim()==='{label}');"
              f"if(b){{b.click();return true}}return false}})()")
def until(cond, label, tries=40):
    for _ in range(tries):
        if cond(): return
        time.sleep(0.5)
    raise AssertionError(label)
url = f"http://127.0.0.1:{os.environ['PORT']}/h.html"
call(op="open", url=url)
until(lambda: "Installer le décodeur" in strip_text(), f"proposition absente : {strip_text()!r}")
assert press("Installer"), "bouton Installer introuvable"
until(lambda: "Redémarrer" in strip_text(), f"pas de proposition de redemarrage : {strip_text()!r}", tries=120)
assert press("Redémarrer"), "bouton Redemarrer introuvable"
time.sleep(16)
f = connect()
tab = [t for t in call(op="tabs")["tabs"] if t["url"] == url]
assert tab, "onglet perdu au redemarrage"
call(op="activate", id=tab[0]["id"]); time.sleep(5)
text = call(op="read", id=tab[0]["id"])["text"]
t = float(re.search(r"t=([\d.]+)", text).group(1))
assert t > 2 and "err=0" in text, f"video non lue apres installation : {text}"
assert strip_text() == "", "la proposition reste affichee apres l'installation"
print(f"OK : proposition du decodeur → installation → redemarrage → video lue ({text.strip()})")
PY
