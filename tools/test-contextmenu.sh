#!/usr/bin/env bash
# Clic droit simule dans une vraie page : le menu s'ouvre, puis se referme. Instance isolee. Sur le bureau de test sway,
# passer SWAYSOCK (fenetres comptees dans son arbre) ; sinon Hyprland.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)"
export ECHO_CONTROL_NAME="test-$$"
trap 'cp "$ECHO_RUN_DIR/browser.log" /tmp/claude-1000/dt.log 2>/dev/null; ./stop.sh >/dev/null 2>&1 || true' EXIT
ECHO_DATA_DIR="$(mktemp -d)" ./start.sh release >/dev/null
sleep 12
python3 - <<'PY'
import json, os, socket, time
sock = socket.socket(socket.AF_UNIX)
sock.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
stream = sock.makefile("rw")
def call(**r):
    stream.write(json.dumps(r) + "\n"); stream.flush()
    return json.loads(stream.readline())
assert call(op="open", url="https://example.org/")["ok"]
time.sleep(6)
assert not call(op="menu")["open"]
assert call(op="click", x=300, y=200, button="right")["ok"]
time.sleep(1.5)
assert call(op="menu")["open"], "le menu contextuel ne s'ouvre pas sur la page"
time.sleep(1)
assert call(op="menu")["open"], "le menu se referme tout seul"
call(op="click", x=500, y=400); time.sleep(1)
assert not call(op="menu")["open"], "un clic gauche ailleurs ne ferme pas le menu"
call(op="click", x=300, y=200, button="right"); time.sleep(1)
call(op="click", x=500, y=400, button="right"); time.sleep(1)
assert call(op="menu")["open"], "un second clic droit doit rouvrir le menu au nouvel endroit"
call(op="click", x=500, y=300); time.sleep(1)
import subprocess
def windows():
    pid = int(open(os.environ["ECHO_RUN_DIR"] + "/browser.pid").read())
    def tree(p):
        out = [p]
        for c in subprocess.run(["pgrep", "-P", str(p)], capture_output=True, text=True).stdout.split():
            out += tree(int(c))
        return out
    pids = set(tree(pid))
    if os.environ.get("SWAYSOCK"):  # bureau de test sway (sans ecran) : fenetres lues dans son arbre
        def views(node):
            found = [node] if node.get("pid") in pids and node.get("type") in ("con", "floating_con") else []
            for child in node.get("nodes", []) + node.get("floating_nodes", []):
                found += views(child)
            return found
        return views(json.loads(subprocess.check_output(["swaymsg", "-t", "get_tree"])))
    return [c for c in json.loads(subprocess.check_output(["hyprctl", "clients", "-j"])) if c["pid"] in pids]
call(op="devtools"); time.sleep(3)
assert call(op="devtools_open")["open"], "F12 : devtools non ouverts"
assert len(windows()) == 1, f"devtools hors de la fenetre : {len(windows())} fenetres"
call(op="devtools"); time.sleep(2)
assert not call(op="devtools_open")["open"], "F12 : devtools non refermes"
assert call(op="tabs")["ok"] and len(windows()) == 1, "fermer les devtools a ferme Echo"
call(op="devtools"); time.sleep(3)
assert call(op="devtools_open")["open"], "reouverture des devtools"
call(op="ui", request={"kind": "resizeDevTools", "dx": 150}); time.sleep(1)
call(op="ui", request={"kind": "closeDevTools"}); time.sleep(1)
assert not call(op="devtools_open")["open"], "la croix ne referme pas les devtools"
assert call(op="tabs")["ok"] and len(windows()) == 1, "la croix a ferme Echo"
call(op="click", x=300, y=200, button="right"); time.sleep(1)
call(op="ui", request={"kind": "runContextMenu", "action": "viewSource"}); time.sleep(2)
src = [t for t in call(op="tabs")["tabs"] if t["title"].startswith("view-source:")]
assert src, f"code source non ouvert en onglet : {[(t['title'], t['url']) for t in call(op='tabs')['tabs']]}"
assert len(windows()) == 1, "code source ouvert hors de la fenetre"
print("OK : clic droit + devtools + code source")
PY
