#!/usr/bin/env bash
# Banc video : CPU consomme et charge du decodeur GPU pendant 4K VP9. Usage : ECHO_HWDEC=1|0 tools/bench-video.sh
set -uo pipefail
cd "$(dirname "$0")/.."
VIDEO="${TMPDIR:-/tmp}/echo-test4k.webm"
[ -f "$VIDEO" ] || ffmpeg -loglevel error -f lavfi -i testsrc2=size=3840x2160:rate=30 -t 20 -c:v libvpx-vp9 -b:v 8M -deadline realtime -cpu-used 8 -y "$VIDEO"
export ECHO_RUN_DIR="$(mktemp -d)" ECHO_CONTROL_NAME="hw-$$" ECHO_DATA_DIR="$(mktemp -d)" ECHO_SLEEP_AFTER_S=100000
echo '<title>v</title><body style="margin:0;background:#000"><video id=v src="file://$VIDEO" autoplay muted loop style="width:100%"></video>' > $ECHO_RUN_DIR/v.html
trap './stop.sh >/dev/null 2>&1' EXIT
./start.sh release >/dev/null; sleep 10
python3 - <<'PY'
import json, os, socket, time
s=socket.socket(socket.AF_UNIX); s.connect(f"/run/user/1000/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock"); f=s.makefile("rw")
f.write(json.dumps({"op":"open","url":"file://"+os.environ["ECHO_RUN_DIR"]+"/v.html"})+"\n"); f.flush(); f.readline()
PY
sleep 8
ROOT=$(cat $ECHO_RUN_DIR/browser.pid)
tree(){ echo $1; for c in $(pgrep -P $1); do tree $c; done; }
P=$(tree $ROOT)
snap(){ t=0; for p in $P; do v=$(awk '{print $14+$15}' /proc/$p/stat 2>/dev/null); t=$((t+${v:-0})); done; echo $t; }
a=$(snap); (nvidia-smi dmon -s u -c 6 -d 1 | tail -4 > /tmp/claude-1000/dmon.txt) ; b=$(snap)
echo "HWDEC=$ECHO_HWDEC cpu=$(( (b-a)/100 )) s CPU sur ~6 s ; dec GPU:"; awk '{print "  sm="$2" mem="$3" enc="$4" dec="$5}' /tmp/claude-1000/dmon.txt | head -3
