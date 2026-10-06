#!/usr/bin/env bash
# Dossiers d'onglets : creer un dossier au clic droit, y ranger un onglet, relancer, retrouver le rangement.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)"
export ECHO_CONTROL_NAME="test-$$"
export ECHO_DATA_DIR="$(mktemp -d)"
trap './stop.sh >/dev/null 2>&1 || true' EXIT
HELPER="$(mktemp)"
cat > "$HELPER" <<'PYH'
import json, os, socket, time
sock = socket.socket(socket.AF_UNIX)
sock.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
stream = sock.makefile("rw")
def call(**r):
    stream.write(json.dumps(r) + "\n"); stream.flush()
    return json.loads(stream.readline())
def example():
    return [t for t in call(op="tabs")["tabs"] if "example" in t["url"]][0]
PYH
PY=$(cat "$HELPER")
./start.sh release >/dev/null; sleep 12
python3 -c "$PY
call(op='open', url='https://example.org/'); time.sleep(5)
call(op='click', target='chrome', x=120, y=330, button='right'); time.sleep(1)
call(op='click', target='chrome', x=116, y=243); time.sleep(1.5)
call(op='click', target='chrome', x=240, y=400); time.sleep(0.5)
call(op='click', target='chrome', x=120, y=205, button='right'); time.sleep(1)
call(op='click', target='chrome', x=120, y=309); time.sleep(1.5)
assert example()['folder'] is not None, 'onglet non range dans le dossier'
print('rangement ok')
"
./stop.sh >/dev/null; sleep 3
./start.sh release >/dev/null; sleep 12
python3 -c "$PY
assert example()['folder'] is not None, 'rangement perdu apres relance'
print('OK : dossiers')
"
