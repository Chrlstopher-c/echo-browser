#!/usr/bin/env bash
# Theme : les pages voient le theme d'Echo (prefers-color-scheme), y compris apres bascule et dans un onglet neuf.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)" ECHO_CONTROL_NAME="test-$$" ECHO_DATA_DIR="$(mktemp -d)"
cat > "$ECHO_RUN_DIR/s.html" <<'H'
<title>s</title><p id=p></p><script>
const m = matchMedia('(prefers-color-scheme: dark)')
const u = () => p.textContent = m.matches ? 'theme=sombre' : 'theme=clair'
u(); m.addEventListener('change', u)
</script>
H
trap './stop.sh >/dev/null 2>&1 || true' EXIT
./start.sh release >/dev/null; sleep 14
python3 - <<'PY'
import json, os, socket, time
sock = socket.socket(socket.AF_UNIX)
sock.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = sock.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
url = "file://" + os.environ["ECHO_RUN_DIR"] + "/s.html"
def theme(i): return call(op="read", id=i)["text"].strip()
call(op="ui", request={"kind": "setColorScheme", "dark": True})
a = call(op="open", url=url)["id"]; time.sleep(2)
assert theme(a) == "theme=sombre", theme(a)
call(op="ui", request={"kind": "setColorScheme", "dark": False}); time.sleep(1)
assert theme(a) == "theme=clair", f"bascule non suivie : {theme(a)}"
b = call(op="open", url=url)["id"]; time.sleep(2)
assert theme(b) == "theme=clair", f"onglet neuf : {theme(b)}"
print("OK : theme des pages aligne sur Echo")
PY
