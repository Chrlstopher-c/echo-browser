#!/usr/bin/env bash
# Fiches au focus : entrer dans un champ reconnu (e-mail) fait proposer les fiches dans la barre ; un clic sur la fiche
# remplit le formulaire. Un champ de recherche ne propose rien. Instance isolee.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import json, time
from banc import *
card = {"id": "f1", "name": "Perso", "fields": {"givenName": "Ada", "email": "ada@example.com"}}
setting("forms.cards", "text", json.dumps([card]))
url = page("form.html", "<style>input{display:block;margin:40px;width:400px;height:40px}</style>"
           "<input type=search name=q placeholder=Recherche><input name=prenom placeholder=Prénom>"
           "<input type=email name=email placeholder=E-mail>", "formulaire")
call(op="open", url=url)
until(lambda: active()["url"] == url and not active()["loading"], "page non chargee")
time.sleep(1)
status = lambda: ev("[...document.querySelectorAll('[role=status]')].map(e=>e.innerText).join('|')") or ""
with_page_focus(url, "document.querySelector('[name=q]').focus()"); time.sleep(1)
assert "Remplir ce formulaire" not in status(), "un champ de recherche propose les fiches"
with_page_focus(url, "document.querySelector('[name=email]').focus()")
until(lambda: "Remplir ce formulaire" in status(), f"pas de proposition au focus : {status()}")
ev("[...document.querySelectorAll('[role=status] button')].find(b=>b.textContent==='Perso').click()")
until(lambda: ev("document.querySelector('[name=email]').value", url) == "ada@example.com", "fiche non appliquee")
assert ev("document.querySelector('[name=prenom]').value", url) == "Ada"
print("OK : fiches proposees au focus d'un champ, remplissage depuis la barre")
PY
