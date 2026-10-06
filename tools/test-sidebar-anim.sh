#!/usr/bin/env bash
# Barre repliee : a l'ouverture (bord gauche) la page est repoussee progressivement, a la fermeture elle revient.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)"
export ECHO_CONTROL_NAME="test-$$"
trap './stop.sh >/dev/null 2>&1 || true' EXIT
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
def ui(**r): return call(op="ui", request=r)
def page_x(): return call(op="layout")["page"]["x"]
def sample(seconds=0.9):
    xs, t0 = [], time.time()
    while time.time() - t0 < seconds:
        xs.append(page_x()); time.sleep(0.008)
    return xs
base = page_x(); assert base > 100, f"page non decalee par la barre : {base}"
ui(kind="setSidebarCollapsed", collapsed=True)
closing = sample(); time.sleep(0.3)
closed = page_x(); assert closed < 20, f"page non etendue apres repli : {closed}"
steps = len({x for x in closing})
assert steps >= 4, f"fermeture non animee : {sorted(set(closing))}"
ui(kind="revealSidebar", reveal=True)
opening = sample()
assert opening[-1] > 100, "la page n'est pas repoussee a l'ouverture"
assert len(set(opening)) >= 4, f"ouverture non animee : {sorted(set(opening))}"
assert opening == sorted(opening), "mouvement non monotone"
ui(kind="revealSidebar", reveal=False)
back = sample(); assert back[-1] == closed, "page non revenue en plein"
print(f"OK : animation (fermeture {steps} etapes, ouverture {len(set(opening))} etapes, page {base}px -> {closed} -> {opening[-1]}px)")
PY
