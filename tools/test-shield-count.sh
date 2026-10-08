#!/usr/bin/env bash
# Bouclier : « bloqués sur cette page » est celui de l'onglet actif (plus celui du premier onglet), et repart de zero
# a chaque nouvelle page. Instance isolee.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import time
from banc import *
ads = "".join(f'<img src="https://pagead2.googlesyndication.com/pagead/imgad?id={i}">' for i in range(4))
pub = page("pub.html", f"<p>pub {ads}", "pub")
calme = page("calme.html", "<p>rien a bloquer", "calme")
BUTTON = "[...document.querySelectorAll('button[aria-label]')].find(b=>/Protection|[Bb]ouclier/.test(b.getAttribute('aria-label')))"
count = lambda: int((ev(f"{BUTTON}?.textContent") or "").strip() or 0)
def charge(url):
    call(op="open", url=url)
    until(lambda: active()["url"] == url and not active()["loading"], f"{url} non charge")
    time.sleep(1.5)
charge(pub)
until(lambda: count() >= 4, f"blocages non comptes sur la page de pub : {count()}", tries=40)
print(f"page de pub : {count()} bloques")
charge(calme)
until(lambda: count() == 0, f"la page calme montre le compteur d'un autre onglet : {count()}")
print("page calme : 0")
call(op="activate", id=[t for t in tabs() if t["url"] == pub][0]["id"])
until(lambda: count() >= 4, "retour sur la page de pub : compteur perdu")
call(op="navigate", id=active()["id"], url=calme)
until(lambda: active()["url"] == calme and count() == 0, f"nouvelle page dans le meme onglet : compteur non remis a zero ({count()})")
print("OK : compteur du bouclier par onglet et par page")
PY
