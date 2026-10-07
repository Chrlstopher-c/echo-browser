#!/usr/bin/env bash
# Clic droit « Copier l'image » : le presse-papiers recoit une image PNG. Sauvegarde et restaure le presse-papiers.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)" ECHO_CONTROL_NAME="test-$$" ECHO_DATA_DIR="$(mktemp -d)"
wl-paste --no-newline > "$ECHO_RUN_DIR/clip.bak" 2>/dev/null || true
trap 'wl-copy < "$ECHO_RUN_DIR/clip.bak" 2>/dev/null || true; ./stop.sh >/dev/null 2>&1 || true' EXIT
magick -size 64x48 xc:'#3366cc' "$ECHO_RUN_DIR/img.jpg"
printf '<title>i</title><body style="margin:0"><img src="img.jpg" style="width:600px;height:400px;display:block">' > "$ECHO_RUN_DIR/img.html"
./start.sh release >/dev/null; sleep 14
python3 - <<'PY'
import json, os, socket, subprocess, time
s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = s.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
call(op="open", url="file://" + os.environ["ECHO_RUN_DIR"] + "/img.html"); time.sleep(3)
call(op="click", x=200, y=150, button="right"); time.sleep(1)
call(op="ui", request={"kind": "runContextMenu", "action": "copyImage"}); time.sleep(3)
types = subprocess.run(["wl-paste", "--list-types"], capture_output=True, text=True).stdout
assert "image/png" in types, f"pas d'image dans le presse-papiers : {types}"
print("OK : copier l'image")
PY
