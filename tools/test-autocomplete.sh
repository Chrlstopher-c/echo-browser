#!/usr/bin/env bash
# Autocompletion de l'adresse : apres une visite, taper le debut du site le complete (partie ajoutee selectionnee),
# effacer ne recomplete pas, Entree ouvre le site complete. Instance isolee.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import time
from banc import *
setting("search.suggest", "flag", False)
url = page("site.html", "<p>site", "site complete")
call(op="open", url=url); time.sleep(2)
host = url.split("/")[2]
type_address("localh")
until(lambda: ev(f"{ADDRESS}.value") == host, f"pas de completion : {ev(f'{ADDRESS}.value')!r}")
sel = ev(f"[{ADDRESS}.selectionStart, {ADDRESS}.selectionEnd]")
assert sel == [6, len(host)], f"partie ajoutee non selectionnee : {sel}"
print(f"completion : localh -> {host}, selection {sel}")
type_address("local")  # effacement d'un caractere
time.sleep(1)
assert ev(f"{ADDRESS}.value") == "local", f"recomplete apres un effacement : {ev(f'{ADDRESS}.value')!r}"
print("OK : autocompletion de l'adresse")
PY
