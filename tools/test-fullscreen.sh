#!/usr/bin/env bash
# Plein ecran d'une page : bord a bord (la page occupe toute la fenetre, sans marge) et Echap en sort. Instance isolee.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import time
from banc import *
url = page("video.html", "<style>body{margin:0}button{position:fixed;inset:0;width:100%;height:100%}</style>"
           "<button onclick=\"document.documentElement.requestFullscreen()\">plein</button>", "video")
call(op="open", url=url)
until(lambda: active()["url"] == url and not active()["loading"], "page non chargee")
time.sleep(1)
avant = ev("[innerWidth, innerHeight]", url)
call(op="click", x=500, y=400); time.sleep(2)
until(lambda: ev("!!document.fullscreenElement", url), "la page n'est pas passee en plein ecran")
plein = ev("[innerWidth, innerHeight, screen.width, screen.height]", url)
print(f"avant {avant}, plein ecran {plein}")
assert plein[0] >= plein[2] - 1 and plein[1] >= plein[3] - 1, f"plein ecran pas bord a bord : {plein}"
call(op="key", code=27, ch="\u001b"); time.sleep(1.5)
until(lambda: not ev("!!document.fullscreenElement", url), "Echap ne sort pas du plein ecran")
apres = ev("[innerWidth, innerHeight]", url)
assert apres == avant, f"la page ne retrouve pas sa place : {avant} -> {apres}"
print("OK : plein ecran bord a bord, Echap en sort")
PY
