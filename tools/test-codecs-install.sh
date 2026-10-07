#!/usr/bin/env bash
# Release publique : installer le decodeur complet depuis Reglages, redemarrer, lire une video H.264 + AAC sans GPU.
# Usage : tools/test-codecs-install.sh [archive .tar.xz] (defaut : dist-release/, fabriquee par package-release.sh)
set -euo pipefail
cd "$(dirname "$0")/.."
ARCHIVE="${1:-$(ls dist-release/*.tar.xz | head -1)}"
WORK="$(mktemp -d)"
export ECHO_RUN_DIR="$WORK/run" ECHO_CONTROL_NAME="test-$$" ECHO_DATA_DIR="$WORK/data" ECHO_HWDEC=0 ECHO_NO_ANCHOR=1
mkdir -p "$ECHO_RUN_DIR" "$ECHO_DATA_DIR" && tar -xf "$ARCHIVE" -C "$WORK"
ffmpeg -loglevel error -f lavfi -i testsrc=d=8:s=640x360:r=30 -f lavfi -i sine=f=440:d=8 \
  -c:v libx264 -pix_fmt yuv420p -c:a aac -shortest "$ECHO_RUN_DIR/va.mp4"
cat > "$ECHO_RUN_DIR/h.html" <<'H'
<title>h</title><p id=p></p><video id=b muted src=va.mp4></video><script>b.play().catch(()=>{});
setInterval(()=>p.textContent=`t=${b.currentTime.toFixed(1)} err=${b.error?b.error.code:0} audio=${b.webkitAudioDecodedByteCount>0}`,300)</script>
H
python3 -m http.server 0 --bind 127.0.0.1 -d "$ECHO_RUN_DIR" >/dev/null 2>&1 & SERVER=$!
setsid "$WORK"/echo-browser-*/echo-browser.sh >"$WORK/log.txt" 2>&1 & BROWSER=$!
trap 'kill -- -$BROWSER 2>/dev/null || true; kill $SERVER 2>/dev/null || true' EXIT
sleep 12
PORT=$(ss -ltnp | grep "pid=$SERVER," | grep -oP '127.0.0.1:\K\d+') python3 - <<'PY'
import json, os, re, socket, time
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
def play():
    i = call(op="open", url=f"http://127.0.0.1:{os.environ['PORT']}/h.html")["id"]; time.sleep(5)
    return call(op="read", id=i)["text"].strip()
def settings():
    i = call(op="open", url="echo://ui/pages.html#reglages")["id"]; time.sleep(3)
    return call(op="read", id=i)["text"]
assert "demandent le décodeur complet" in settings(), "section Video absente des reglages"
before = play()
assert "err=4" in before, f"le decodeur libre lit deja le H.264 ? {before}"
call(op="ui", request={"kind": "installVideoCodecs"})
lib = os.path.join(os.environ["ECHO_DATA_DIR"], "codecs", "libffmpeg.so")
for _ in range(120):
    if os.path.exists(lib): break
    time.sleep(0.5)
assert os.path.exists(lib), "decodeur non telecharge"
call(op="ui", request={"kind": "restartBrowser"}); time.sleep(14)
f = connect()
after = play()
t = float(re.search(r"t=([\d.]+)", after).group(1))
assert t > 2 and "err=0" in after and "audio=true" in after, f"H.264/AAC non lus apres installation : {after}"
assert "Décodeur complet actif" in settings(), "reglages : etat actif non affiche"
print(f"OK : decodeur complet installe et actif ({before} -> {after})")
PY
