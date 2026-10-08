#!/usr/bin/env bash
# Fenetre etroite : sous 900 px la barre se replie d'elle-meme (la page prend la largeur), et revient a la largeur
# normale ; Ctrl+Alt+S replie et deplie. Exige le bureau de test sway : SWAYSOCK=… (la fenetre y est redimensionnee).
cd "$(dirname "$0")/.."
: "${SWAYSOCK:?SWAYSOCK du bureau de test sway requis}"
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import os, subprocess, time
from banc import *
url = page("n.html", "<p>etroite", "n")
call(op="open", url=url)
until(lambda: active()["url"] == url and not active()["loading"], "page non chargee")
time.sleep(1)
width = lambda: ev("innerWidth", url)
large = width()
pid = open(f"{os.environ['ECHO_RUN_DIR']}/browser.pid").read().strip()
sway = lambda cmd: subprocess.run(["swaymsg", f'[pid={pid}] {cmd}'], check=False, capture_output=True)
sway("floating enable, resize set 800 600"); time.sleep(2)
until(lambda: width() >= 760, f"fenetre de 800 px : la page n'a que {width()} px (barre non repliee)")
print(f"large : page {large} px ; 800 px : page {width()} px (barre repliee)")
sway("floating disable"); time.sleep(2)
until(lambda: width() == large, f"fenetre elargie : page {width()} px au lieu de {large}")
call(op="key", code=0x53, ch="s", mods=["ctrl", "alt"]); time.sleep(1.5)
until(lambda: width() > large + 100, f"Ctrl+Alt+S ne replie pas : {width()}")
call(op="key", code=0x53, ch="s", mods=["ctrl", "alt"]); time.sleep(1.5)
until(lambda: width() == large, f"Ctrl+Alt+S ne deplie pas : {width()}")
print("OK : repli en fenetre etroite, Ctrl+Alt+S")
PY
