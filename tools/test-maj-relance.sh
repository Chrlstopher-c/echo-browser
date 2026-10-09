#!/usr/bin/env bash
# Mise a jour preparee + « Redemarrer », parcours de l'archive installee : un second lancement pendant qu'Echo est
# ouvert ne bascule pas de version et ne compte pas d'essai ; la relance bascule, la nouvelle version demarre (fenetre,
# essai confirme). Exige le bureau de test sway : SWAYSOCK=… Usage : tools/test-maj-relance.sh [archive .tar.xz].
set -euo pipefail
cd "$(dirname "$0")/.."
: "${SWAYSOCK:?SWAYSOCK du bureau de test sway requis}"
ARCHIVE="${1:-$(ls -t dist-release/*.tar.xz | head -1)}"
WORK="$(mktemp -d)"; OPT="$WORK/opt"
export ECHO_RUN_DIR="$WORK/run" ECHO_DATA_DIR="$WORK/data" ECHO_CONTROL_NAME="test-$$" ECHO_NO_WELCOME=1
export ECHO_DEVTOOLS_PORT=$((30000 + RANDOM % 20000))
mkdir -p "$ECHO_RUN_DIR" "$OPT" && tar -xf "$ARCHIVE" -C "$OPT"
mv "$OPT"/echo-browser-*/ "$OPT/echo-browser" && cp -a "$OPT/echo-browser" "$OPT/echo-browser.maj"
trap 'pkill -f "^$OPT/" 2>/dev/null || true; sleep 2; rm -rf "$WORK"' EXIT
alive() { python3 - <<'PY'
import json, os, socket, sys
try:
    s = socket.socket(socket.AF_UNIX); s.settimeout(3)
    s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
    f = s.makefile("rw"); f.write(json.dumps({"op": "tabs"}) + "\n"); f.flush(); sys.exit(0 if json.loads(f.readline())["ok"] else 1)
except Exception:
    sys.exit(1)
PY
}
wait_alive() { for _ in $(seq 60); do alive && return 0; sleep 0.5; done; return 1; }
fail() { echo "ECHEC : $1"; exit 1; }
setsid "$OPT/echo-browser/echo-browser.sh" >"$WORK/1.txt" 2>&1 &
wait_alive || fail "premier lancement sans reponse"
[ -d "$OPT/echo-browser.precedent" ] || fail "la mise a jour preparee n'a pas ete appliquee au premier lancement"
cp -a "$OPT/echo-browser.precedent" "$OPT/echo-browser.maj"; rm -f "$OPT/echo-browser/.essai-demarrage"
PID=$(pgrep -f "^$OPT/echo-browser/echo-browser$")
"$OPT/echo-browser/echo-browser.sh" >"$WORK/2.txt" 2>&1 || true
[ -d "$OPT/echo-browser.maj" ] || fail "un second lancement a bascule de version sous l'instance ouverte"
[ ! -f "$OPT/echo-browser/.essai-demarrage" ] || fail "un second lancement a compte un essai"
python3 - <<'PY'
import json, os, socket
s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = s.makefile("rw"); f.write(json.dumps({"op": "ui", "request": {"kind": "restartBrowser"}}) + "\n"); f.flush(); f.readline()
PY
for _ in $(seq 40); do [ -d "$OPT/echo-browser.maj" ] || break; sleep 0.5; done
[ ! -d "$OPT/echo-browser.maj" ] || fail "la relance n'a pas applique la mise a jour"
sleep 3; wait_alive || fail "pas de navigateur apres la relance"
for _ in $(seq 30); do [ -f "$OPT/echo-browser/.essai-demarrage" ] || break; sleep 0.5; done
[ ! -f "$OPT/echo-browser/.essai-demarrage" ] || fail "la nouvelle version n'a pas confirme son demarrage"
swaymsg -t get_tree | grep -q "\"pid\": $PID" || fail "aucune fenetre apres la relance"
echo "OK : second lancement sans bascule ni essai ; relance → mise a jour appliquee, fenetre, demarrage confirme"
