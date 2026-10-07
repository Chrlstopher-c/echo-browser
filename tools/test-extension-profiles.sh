#!/usr/bin/env bash
# Profils = identites : une extension du profil principal est inactive dans un autre profil, s'y active quand on l'y
# ajoute, s'y desactive quand on l'en retire, sans jamais quitter le profil principal. Les conteneurs d'un profil lui
# appartiennent. Telecharge Proton Pass depuis le catalogue (reseau requis). Instance isolee.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)" ECHO_CONTROL_NAME="test-$$" ECHO_DATA_DIR="$(mktemp -d)"
PROTON=ghmbeldphafepmbegfdlkpapadhbakde
mkdir -p "$ECHO_DATA_DIR/profile/External Extensions"
echo '{"external_update_url":"https://clients2.google.com/service/update2/crx"}' \
  > "$ECHO_DATA_DIR/profile/External Extensions/$PROTON.json"
trap './stop.sh >/dev/null 2>&1 || true' EXIT
./start.sh release >/dev/null; sleep 30
python3 - "$PROTON" <<'PY'
import json, os, socket, sys, time
s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = s.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
url = f"chrome-extension://{sys.argv[1]}/manifest.json"
def active_here():
    tab = call(op="open", url=url)["id"]; time.sleep(2.5)
    text = call(op="read", id=tab)["text"]
    call(op="close", id=tab)
    return '"manifest_version"' in text
def until(want, label, tries=30):
    for _ in range(tries):
        if active_here() == want: return
        time.sleep(3)
    raise AssertionError(label)
until(True, "Proton absent du profil principal")
call(op="ui", request={"kind": "setSpace", "id": "essai"}); time.sleep(2)
installed = os.path.join(os.environ["ECHO_DATA_DIR"], "profile", "conteneur-profil-essai", "Extensions", sys.argv[1])
for _ in range(60):
    if os.path.isdir(installed): break
    time.sleep(3)
assert os.path.isdir(installed), "Chromium n'a pas installe Proton dans le profil essai"
until(False, "Proton actif dans un profil qui ne l'a pas")
call(op="ui", request={"kind": "installExtension", "source": sys.argv[1]})
until(True, "Proton ajoute au profil essai mais inactif", tries=10)
call(op="ui", request={"kind": "removeExtension", "id": sys.argv[1]})
until(False, "Proton retire du profil essai mais encore actif", tries=10)
call(op="ui", request={"kind": "newTab", "url": "about:blank", "container": "c1"}); time.sleep(3)
assert any(t.get("container") == "profil-essai--c1" for t in call(op="tabs")["tabs"]), "conteneur non rattache au profil"
call(op="ui", request={"kind": "setSpace", "id": "graphite"}); time.sleep(2)
until(True, "Proton perdu par le profil principal", tries=5)
print("OK : extensions separees par profil (ajout, retrait), conteneur rattache au profil")
PY
