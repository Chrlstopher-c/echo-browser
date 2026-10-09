#!/usr/bin/env bash
# Accueil du premier lancement : il configure (theme, moteur, import depuis un faux Chrome), propose Precedent, ne
# montre pas la bulle d'astuce pendant qu'il s'affiche, et ne reste pas dans l'historique de l'onglet. Instance isolee.
cd "$(dirname "$0")/.."
BANC_WELCOME=1 BANC_NO_START=1 source tools/banc.sh
export ECHO_IMPORT_HOME="$W/maison"
mkdir -p "$ECHO_IMPORT_HOME/.config/google-chrome/Default"
cat > "$ECHO_IMPORT_HOME/.config/google-chrome/Default/Bookmarks" <<'JSON'
{"roots":{"bookmark_bar":{"children":[{"type":"url","name":"Favori importe","url":"https://example.org/importe"}]}}}
JSON
./start.sh release >/dev/null; sleep 8
uv run -q --with websocket-client python - <<'PY'
import time
from banc import *
P = "echo://ui/pages.html"
step = lambda: ev("document.querySelector('[data-step]')?.dataset.step", P)
click = lambda text: ev(f"[...document.querySelectorAll('button')].find(b=>b.textContent.trim()==={text!r}).click()", P)
until(lambda: step() == "bienvenue", "accueil absent au premier lancement")
assert "À découvrir" not in (ev("[...document.querySelectorAll('[role=status]')].map(e=>e.getAttribute('aria-label')).join('|')") or "")
click("Suivant"); until(lambda: step() == "theme", "etape theme")
click("Clair"); until(lambda: ev("localStorage.getItem('echo.scheme.now')") == '"light"', "le theme choisi ne s'applique pas")
click("Précédent"); until(lambda: step() == "bienvenue", "Precedent")
click("Suivant"); click("Suivant"); until(lambda: step() == "moteur", "etape moteur")
click("Qwant"); time.sleep(1)
click("Suivant"); until(lambda: step() == "import", "etape import")
until(lambda: "Google Chrome" in ev("document.body.innerText", P), "Chrome non detecte")
click("Importer favoris et historique")
until(lambda: any(b["url"] == "https://example.org/importe" for b in [] ) or "Importé" in ev("document.body.innerText", P), "import")
print("accueil : theme, moteur, import")
while step() != "compte" and step() is not None and ev("[...document.querySelectorAll('button')].some(b=>b.textContent.trim()==='Suivant')", P):
    click("Suivant"); time.sleep(0.5)
click("Commencer"); time.sleep(2)
assert active()["url"].startswith("echo://ui/nouvel-onglet"), active()["url"]
assert active()["canGoBack"] is False, "Precedent ramenerait a l'accueil"
call(op="open", url="echo://ui/pages.html#bibliotheque"); time.sleep(2)
assert "Favori importe" in ev("document.body.innerText", "echo://ui/pages.html"), "favori importe absent"
type_address("meteo")
until(lambda: options() and "qwant.com" in options()[0], f"moteur choisi a l'accueil non retenu : {options()}")
print("OK : accueil qui configure (theme, Precedent, moteur, import Chrome), pas de bulle pendant l'accueil")
PY
