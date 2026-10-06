#!/usr/bin/env bash
# Fenetre rangee sur un autre espace Hyprland : la page passe en « hidden », puis revient « visible ». Instance isolee.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)" ECHO_CONTROL_NAME="vis-$$" ECHO_DATA_DIR="$(mktemp -d)"
VIS="$ECHO_RUN_DIR/vis.html"
cat > "$VIS" <<'H'
<title>v</title><p id=p></p><script>let n=0;function f(){n++;requestAnimationFrame(f)}f();
setInterval(()=>{p.textContent='vis='+document.visibilityState+' raf='+n},300)</script>
H
trap './stop.sh >/dev/null 2>&1' EXIT
./start.sh release >/dev/null; sleep 12
python3 - <<'PY'
import json, os, socket, time, subprocess
sock = socket.socket(socket.AF_UNIX)
sock.connect(f"/run/user/1000/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = sock.makefile("rw")
def call(**r):
    f.write(json.dumps(r)+"\n"); f.flush(); return json.loads(f.readline())
t = call(op="open", url="file://"+os.environ["ECHO_RUN_DIR"]+"/vis.html")["id"]; time.sleep(3)
def state(): return call(op="read", id=t)["text"].strip()
pid = int(open(os.environ["ECHO_RUN_DIR"]+"/browser.pid").read())
def kids(p):
    o=[p]
    try:
        for c in subprocess.check_output(["pgrep","-P",str(p)]).split(): o+=kids(int(c))
    except subprocess.CalledProcessError: pass
    return o
ps=set(kids(pid))
w=[c for c in json.loads(subprocess.check_output(["hyprctl","clients","-j"])) if c["pid"] in ps][0]
assert "vis=visible" in state()
subprocess.run(["hyprctl","dispatch","movetoworkspacesilent",f"special:vis,address:{w['address']}"],capture_output=True)
time.sleep(3); a=state(); time.sleep(3); b=state()
assert "vis=hidden" in b, f"page non masquee : {b}"
subprocess.run(["hyprctl","dispatch","movetoworkspacesilent",f"{w['workspace']['id']},address:{w['address']}"],capture_output=True)
time.sleep(2); assert "vis=visible" in state(), "page non reaffichee"
print("OK : occlusion suivie")
PY
