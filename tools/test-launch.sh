#!/usr/bin/env bash
# Lancer Echo avec des fichiers : `echo-browser dossier/ note.txt` ouvre ces pages au demarrage ; relance pendant qu'il
# tourne → les pages vont a l'instance ouverte (aucun second navigateur). Instance isolee.
set -euo pipefail
cd "$(dirname "$0")/.."
W="$(mktemp -d)"; mkdir -p "$W/dossier"; echo "premiere note" > "$W/dossier/note.txt"; echo "deuxieme" > "$W/autre.txt"
export ECHO_RUN_DIR="$W/run" ECHO_DATA_DIR="$W/data" ECHO_CONTROL_NAME="lancement-$$" ECHO_NO_WELCOME=1 W
mkdir -p "$W/run"
trap './stop.sh >/dev/null 2>&1 || true' EXIT
./start.sh release "$W/dossier/note.txt" >/dev/null; sleep 9
python3 - <<'PY'
import json, os, socket, subprocess, time
W = os.environ["W"]
s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = s.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
urls = [t["url"] for t in call(op="tabs")["tabs"]]
assert any(u.endswith("/dossier/note.txt") for u in urls), f"fichier du lancement non ouvert : {urls}"
print("lancement avec un fichier : ouvert")
env = dict(os.environ, LD_LIBRARY_PATH=os.environ.get("LD_LIBRARY_PATH", "") + ":" + os.path.expanduser("~/.local/share/cef"))
t0 = time.time()
r = subprocess.run(["target/release/echo-browser", f"{W}/autre.txt"], env=env, capture_output=True, timeout=30)
assert r.returncode == 0 and time.time() - t0 < 10, f"second lancement : code {r.returncode}, {time.time() - t0:.1f} s"
time.sleep(1.5)
urls = [t["url"] for t in call(op="tabs")["tabs"]]
assert any(u.endswith("/autre.txt") for u in urls), f"page non confiee a l'instance ouverte : {urls}"
print(f"OK : lancement avec fichiers, et relance confiee a l'instance ouverte ({time.time() - t0:.1f} s)")
PY
