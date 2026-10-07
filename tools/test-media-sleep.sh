#!/usr/bin/env bash
# Veille : un onglet qui lit une video (meme muette) ne s'endort pas ; une video en pause, si.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)" ECHO_CONTROL_NAME="test-$$" ECHO_DATA_DIR="$(mktemp -d)" ECHO_SLEEP_AFTER_S=20
ffmpeg -loglevel error -f lavfi -i testsrc2=size=320x240:rate=25 -f lavfi -i sine=frequency=440 -t 6 -c:v libx264 -pix_fmt yuv420p -c:a aac -shortest -y "$ECHO_RUN_DIR/v.mp4"
cat > "$ECHO_RUN_DIR/joue.html" <<'H'
<title>joue</title><body style="margin:0;height:100vh"><video id=v src="v.mp4" loop></video>
<script>document.addEventListener('click', () => { v.muted = false; v.volume = 0.05; v.play() })</script>
H
echo '<title>pause</title><video src="v.mp4" muted></video>' > "$ECHO_RUN_DIR/pause.html"
trap './stop.sh >/dev/null 2>&1 || true' EXIT
./start.sh release >/dev/null; sleep 14
python3 - <<'PY'
import json, os, socket, time
s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = s.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
base = "file://" + os.environ["ECHO_RUN_DIR"]
call(op="open", url=base + "/joue.html"); time.sleep(3)
call(op="click", x=300, y=300); time.sleep(2)  # lecture avec le son, comme un utilisateur
call(op="open", url=base + "/pause.html"); time.sleep(3)
call(op="open", url="https://example.org/"); time.sleep(50)
by = {t["title"]: t for t in call(op="tabs")["tabs"]}
assert not by["joue"]["asleep"], "un onglet en lecture s'est endormi"
assert by["pause"]["asleep"], "un onglet sans lecture devait s'endormir"
print("OK : pas de veille pendant une lecture")
PY
