#!/usr/bin/env bash
# Reprise exacte : une page avec un texte saisi, un champ mot de passe et un son a 30 s. Relance d'Echo : le texte et la
# position du son reviennent, le mot de passe n'est jamais garde (ni dans session.json). Instance isolee.
set -euo pipefail
cd "$(dirname "$0")/.."
W="$(mktemp -d)"; PORT=$((25000 + RANDOM % 5000))
python3 - "$W/son.wav" <<'P'
import sys, wave
w = wave.open(sys.argv[1], "wb"); w.setnchannels(1); w.setsampwidth(1); w.setframerate(4000)
w.writeframes(b"\x80" * 4000 * 60); w.close()
P
cat > "$W/index.html" <<'H'
<!doctype html><title>reprise</title><textarea id="note"></textarea><input name="ville">
<input type="password" id="mdp"><audio id="son" src="son.wav" preload="auto" controls></audio>
H
python3 tools/serveur-test.py "$PORT" "$W" >/dev/null 2>&1 & SERVEUR=$!
export ECHO_RUN_DIR="$W/run" ECHO_DATA_DIR="$W/data" ECHO_CONTROL_NAME="etat-$$" ECHO_DEVTOOLS_PORT=$((30000 + RANDOM % 9000))
export ECHO_NO_WELCOME=1 W PORT
mkdir -p "$W/run"
trap './stop.sh >/dev/null 2>&1 || true; kill $SERVEUR 2>/dev/null || true' EXIT
./start.sh release >/dev/null; sleep 8
uv run -q --with websocket-client python - <<'PY'
import json, os, socket, subprocess, time, urllib.request, websocket
W, PORT, DT = os.environ["W"], os.environ["PORT"], os.environ["ECHO_DEVTOOLS_PORT"]
page = f"http://localhost:{PORT}/"
def connect():
    s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
    f = s.makefile("rw")
    def call(**r):
        f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
    return call
def ev(js):
    t = [t for t in json.load(urllib.request.urlopen(f"http://127.0.0.1:{DT}/json/list")) if t["url"].startswith(page)][0]
    ws = websocket.create_connection(t["webSocketDebuggerUrl"], suppress_origin=True)
    ws.send(json.dumps({"id": 1, "method": "Runtime.evaluate", "params": {"expression": js, "returnByValue": True, "awaitPromise": True}}))
    while (m := json.loads(ws.recv())).get("id") != 1: pass
    ws.close(); return m["result"]["result"].get("value")
call = connect()
call(op="open", url=page); time.sleep(3)
ev("""(async()=>{const set=(e,v)=>{e.value=v;e.dispatchEvent(new Event('input',{bubbles:true}))};
set(note,'brouillon important');set(document.getElementsByName('ville')[0],'Bordeaux');set(mdp,'secret123');
await new Promise(r=>son.readyState>0?r():son.addEventListener('loadedmetadata',r,{once:true}));son.currentTime=30;
son.dispatchEvent(new Event('timeupdate'));return 1})()""")
time.sleep(4)
subprocess.run(["./stop.sh"], capture_output=True)
session = open(f"{W}/data/last-session.json").read()
assert "brouillon important" in session, "etat absent de la session sauvegardee"
assert "secret123" not in session, "mot de passe garde dans la session"
subprocess.run(["./start.sh", "release"], capture_output=True); time.sleep(9)
call = connect()
for _ in range(20):
    if ev("note.value") == "brouillon important" and ev("document.getElementsByName('ville')[0].value") == "Bordeaux":
        break
    time.sleep(0.5)
else:
    raise AssertionError(f"saisie non restituee : {ev('note.value')!r}")
assert ev("mdp.value") == "", "mot de passe restitue"
for _ in range(20):
    if (ev("son.currentTime") or 0) >= 29: break
    time.sleep(0.5)
else:
    raise AssertionError(f"position du son non restituee : {ev('son.currentTime')}")
print("OK : reprise exacte (saisie et position du son revenues apres relance, mot de passe jamais garde)")
PY
