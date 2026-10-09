#!/usr/bin/env bash
# Profils au pied de la barre : une installation neuve n'a qu'un profil, dont le nom est affiche ; chaque pastille
# offre une cible d'au moins 24 px ; un profil d'origine deja utilise (dossier present) reste visible. Instance isolee.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import time
from banc import *
radios = lambda: ev("[...document.querySelectorAll('[role=radiogroup][aria-label=Profils] [role=radio]')]"
                    ".map(r=>[r.getAttribute('aria-label'), r.innerText, Math.round(r.getBoundingClientRect().height),"
                    "Math.round(r.getBoundingClientRect().width), r.getAttribute('aria-checked')])")
until(lambda: len(radios()) >= 1, "aucune pastille de profil")
r = radios()
assert len(r) == 1, f"installation neuve : un seul profil attendu, {r}"
label, text, h, w, checked = r[0]
assert text.strip() == "Personnel" and checked == "true", r
assert h >= 24 and w >= 24, f"cible trop petite : {w}x{h}"
assert label == "Profil Personnel", label
print(f"neuf : {r}")
ev("document.querySelector('[aria-label=\"Nouveau profil\"]').click()")
until(lambda: len(radios()) == 2, f"creation de profil : {radios()}")
active = [x for x in radios() if x[4] == "true"][0]
assert active[1].strip().startswith("Profil 2"), active
assert active[0] == "Profil 2", f"libelle doublé : {active[0]}"
print(f"OK : pastilles de profil ({radios()})")
PY
