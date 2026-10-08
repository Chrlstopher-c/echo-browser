#!/usr/bin/env bash
# Dossiers d'onglets : clic droit sur un onglet → « Dossier avec cet onglet », relancer, retrouver le rangement.
# Les gestes passent par la page de la barre (evenements DOM), pas par des coordonnees : la mise en page bouge.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_NO_WELCOME=1 ECHO_RUN_DIR="$(mktemp -d)" ECHO_CONTROL_NAME="test-$$"
export ECHO_DATA_DIR="$(mktemp -d)"
export ECHO_DEVTOOLS_PORT=$((30000 + RANDOM % 9000))
trap './stop.sh >/dev/null 2>&1 || true; rm -rf "$ECHO_RUN_DIR" "$ECHO_DATA_DIR"' EXIT
check() {
  uv run -q --with websocket-client python - "$1" <<'PY'
import json, os, socket, sys, time, urllib.request, websocket
s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = s.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
def ev(js):
    pages = json.load(urllib.request.urlopen(f"http://127.0.0.1:{os.environ['ECHO_DEVTOOLS_PORT']}/json/list"))
    t = [t for t in pages if t["url"].startswith("echo://ui/index.html")][0]
    ws = websocket.create_connection(t["webSocketDebuggerUrl"], suppress_origin=True)
    ws.send(json.dumps({"id": 1, "method": "Runtime.evaluate", "params": {"expression": js, "returnByValue": True}}))
    while (m := json.loads(ws.recv())).get("id") != 1: pass
    ws.close(); return m["result"]["result"].get("value")
example = lambda: [t for t in call(op="tabs")["tabs"] if "example" in t["url"]][0]
if sys.argv[1] == "ranger":
    call(op="open", url="https://example.org/"); time.sleep(5)
    assert ev("""(()=>{const row=[...document.querySelectorAll('*')]
        .filter(e=>e.children.length===0&&e.textContent.trim()==='Example Domain').pop();
      if(!row)return 0;const r=row.getBoundingClientRect();
      row.dispatchEvent(new MouseEvent('contextmenu',
        {bubbles:true,cancelable:true,clientX:r.x+5,clientY:r.y+5,button:2}));return 1})()""") == 1, \
        "onglet absent de la barre"
    time.sleep(0.8)
    assert ev("""(()=>{const b=[...document.querySelectorAll('button,[role=menuitem]')]
        .find(x=>x.textContent.trim()==='Dossier avec cet onglet');
      if(!b)return 0;b.click();return 1})()""") == 1, "entree « Dossier avec cet onglet » absente du menu"
    for _ in range(20):
        if example()["folder"] is not None: break
        time.sleep(0.25)
    assert example()["folder"] is not None, "onglet non range dans le dossier"
    print("rangement ok")
else:
    assert example()["folder"] is not None, "rangement perdu apres relance"
    print("OK : dossiers (clic droit sur l'onglet, rangement garde a la relance)")
PY
}
./start.sh release >/dev/null; sleep 12
check ranger
./stop.sh >/dev/null; sleep 3
./start.sh release >/dev/null; sleep 12
check relance
