#!/usr/bin/env bash
# Glisser un onglet sur un dossier l'y range ; le glisser vers la liste libre l'en sort. Instance isolee.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)" ECHO_CONTROL_NAME="test-$$" ECHO_DATA_DIR="$(mktemp -d)"
trap './stop.sh >/dev/null 2>&1 || true' EXIT
./start.sh release >/dev/null; sleep 14
python3 - <<'PY'
import json, os, socket, time
sock = socket.socket(socket.AF_UNIX)
sock.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = sock.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
def example(): return [t for t in call(op="tabs")["tabs"] if "example" in t["url"]][0]
call(op="open", url="https://example.org/"); time.sleep(4)
folders = json.dumps([{"id": "dtest", "name": "Travail", "collapsed": False}])
call(op="ui", request={"kind": "updateSetting", "key": "tabs.folders", "value": {"type": "text", "value": folders}})
time.sleep(1.5)
# Dossier en tete de liste (y≈141), onglets libres en dessous : Nouvel onglet (≈186), Example (≈220).
call(op="drag", target="chrome", **{"from": [120, 220], "to": [120, 141]}); time.sleep(1.5)
assert example()["folder"] == "dtest", f"non range : {example()}"
# Dans le dossier, l'onglet est sous l'en-tete (≈175) ; on le lache sur la liste libre (≈215).
call(op="drag", target="chrome", **{"from": [130, 175], "to": [130, 225]}); time.sleep(1.5)
assert example()["folder"] is None, f"non sorti : {example()}"
print("OK : glisser vers un dossier et hors du dossier")
PY
