#!/usr/bin/env bash
# Clic droit et raccourcis de confort : l'entree du bouclier dit ce qu'elle fera, « Rechercher dans la page » est
# proposee, Ctrl+Maj+C copie l'adresse (bulle « Adresse copiée »). Instance isolee.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import time
from banc import *
url = page("p.html", "<style>body{height:100vh}input{position:fixed;left:20px;top:200px;width:300px}</style>"
           "<p id=t>pomme de terre</p><input id=champ>", "p")
call(op="open", url=url)
until(lambda: active()["url"] == url and not active()["loading"], "page non chargee")
time.sleep(1)
texte = context_menu(url, 600, 500)
assert "Désactiver le bouclier sur ce site" in texte, texte
assert "Rechercher dans la page" in texte, texte
assert "Traduire la page en français" in texte, texte
call(op="click", x=800, y=600); time.sleep(0.5)
ev("getSelection().selectAllChildren(document.getElementById('t'));1", url)
texte = context_menu(url, 40, 22)
assert "Rechercher « pomme de terre » sur Google" in texte, texte
call(op="click", x=800, y=600); time.sleep(0.5)
texte = context_menu(url, 100, 210)
assert "mode lecture" not in texte, f"mode lecture propose dans un champ : {texte}"
call(op="click", x=800, y=600); time.sleep(0.5)
call(op="key", code=0x43, ch="c", mods=["ctrl", "shift"])
until(lambda: "Adresse copiée" in (ev("[...document.querySelectorAll('[role=status]')].map(e=>e.innerText).join('|')") or ""),
      "Ctrl+Maj+C sans effet")
ui({"kind": "openPage", "page": "reglages"}); time.sleep(3)
P = "echo://ui/pages.html"
ev("[...document.querySelectorAll('button')].find(b=>b.textContent.trim()==='Installer Bitwarden').click()", P)
until(lambda: any("nngceckbapebfimnlniiiahkandclblb" in t["url"] for t in tabs()), "Installer Bitwarden n'ouvre pas sa fiche")
print("OK : clic droit explicite (bouclier, recherche, traduction), Ctrl+Maj+C, mots de passe")
PY
