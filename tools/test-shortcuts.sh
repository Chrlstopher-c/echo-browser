#!/usr/bin/env bash
# Raccourcis et fichiers locaux : Ctrl+W, Ctrl+Maj+T rouvre, Ctrl+T, Ctrl+moins, Ctrl+O ouvre le dialogue du systeme
# (le cas AZERTY — lettre lue dans le caractere, pas la position — est couvert par les tests de shortcuts.rs : une
# frappe injectee voit son caractere recalcule par Chromium depuis le code) ; un dossier et un fichier tapes par leur chemin
# s'ouvrent (liste du dossier, contenu du fichier). Instance isolee, bus D-Bus prive.
set -euo pipefail
cd "$(dirname "$0")/.."
W="$(mktemp -d)"; PORT=$((29500 + RANDOM % 400))
mkdir -p "$W/dossier test"; echo "contenu du fichier texte" > "$W/dossier test/note.txt"
echo '<!doctype html><title>page</title><p>page</p>' > "$W/index.html"
python3 tools/serveur-test.py "$PORT" "$W" >/dev/null 2>&1 & SERVEUR=$!
export ECHO_RUN_DIR="$W/run" ECHO_DATA_DIR="$W/data" ECHO_CONTROL_NAME="raccourcis-$$" ECHO_NO_WELCOME=1 W PORT
mkdir -p "$W/run"
# Bus D-Bus prive : sans lui, le dialogue d'ouverture passe par le portail de la session de l'utilisateur et s'ouvre
# sur SON ecran (vu le 08/10). Lance et arrete par ce script.
BUS="$(dbus-daemon --session --fork --print-address=1 --print-pid=1)"
export DBUS_SESSION_BUS_ADDRESS="$(echo "$BUS" | head -1)"; BUS_PID="$(echo "$BUS" | tail -1)"
trap './stop.sh >/dev/null 2>&1 || true; kill $SERVEUR $BUS_PID 2>/dev/null || true' EXIT
./start.sh release >/dev/null; sleep 8
python3 - <<'PY'
import json, os, socket, subprocess, time
W, PORT = os.environ["W"], os.environ["PORT"]
s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = s.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
tabs = lambda: call(op="tabs")["tabs"]
def key(code, ch="", *mods): call(op="key", code=code, ch=ch, mods=list(mods)); time.sleep(1.2)
page = f"http://localhost:{PORT}/"
call(op="open", url=page); time.sleep(2); call(op="open", url=page + "?b"); time.sleep(2)
n = len(tabs())
key(0x57, "w", "ctrl")
assert len(tabs()) == n - 1, f"Ctrl+W n'a pas ferme l'onglet : {n} -> {len(tabs())}"
key(0x54, "t", "ctrl", "shift")
assert len(tabs()) == n and any(t["url"].endswith("?b") for t in tabs()), "Ctrl+Maj+T n'a pas rouvert l'onglet"
print("Ctrl+W ferme, Ctrl+Maj+T rouvre")
key(0x54, "t", "ctrl"); assert len(tabs()) == n + 1, "Ctrl+T"
key(0x57, "w", "ctrl"); assert len(tabs()) == n, "Ctrl+W sur le nouvel onglet"
call(op="activate", id=[t for t in tabs() if t["url"].endswith("?b")][0]["id"]); time.sleep(0.5)
key(0xBD, "-", "ctrl")
zoom = [t for t in tabs() if t["url"].endswith("?b")][0].get("zoom", 1)
assert zoom < 1, f"Ctrl+moins sans effet : zoom {zoom}"
print(f"Ctrl+T, Ctrl+W, Ctrl+moins (zoom {zoom:.2f})")
# Ctrl+O : la demande de dialogue part (le dialogue lui-meme passe par le portail du bureau : sur le bus prive du
# test, il n'y en a pas — sur une vraie session, c'est le selecteur du systeme) et Echo reste debout.
key(0x4F, "o", "ctrl"); time.sleep(1.5)
assert "dialogue d'ouverture demande" in open(os.environ["ECHO_RUN_DIR"] + "/browser.log", errors="ignore").read()
assert call(op="tabs")["ok"], "Echo tombe sur Ctrl+O"
print("Ctrl+O : dialogue demande, Echo debout")
tab = tabs()[-1]["id"]
call(op="ui", request={"kind": "navigate", "id": tab, "input": f"{W}/dossier test"}); time.sleep(2.5)
texte = call(op="read", id=tab)["text"]
assert "note.txt" in texte, f"dossier non liste : {texte[:200]!r}"
call(op="ui", request={"kind": "navigate", "id": tab, "input": f"{W}/dossier test/note.txt"}); time.sleep(2)
assert "contenu du fichier texte" in call(op="read", id=tab)["text"], "fichier non ouvert"
print("OK : raccourcis (Ctrl+W/T/Maj+T/moins/O) et fichiers / dossiers ouverts par leur chemin")
PY
