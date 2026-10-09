#!/usr/bin/env bash
# Navigation privee : Ctrl+Maj+N ouvre un onglet marque « Privé » ; ce qui y est visite n'entre pas dans l'historique,
# ses cookies ne touchent pas le profil, et l'onglet ne revient pas apres une relance. Instance isolee.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import time
from banc import *
secret = page("secret.html", "<p>secret<script>document.cookie='prive=1; max-age=3600'</script>", "page secrete")
normal = page("normal.html", "<p>normal<script>document.title='normal '+(document.cookie||'sans cookie')</script>", "normal")
call(op="open", url=page("depart.html", "<p>depart", "depart")); time.sleep(1.5)
call(op="key", code=0x4E, ch="n", mods=["ctrl", "shift"]); time.sleep(3)
until(lambda: "Privé" in (ev("[...document.querySelectorAll('[data-container]')].map(e=>e.dataset.container).join()") or ""),
      "onglet prive non marque")
labels = ev("[...document.querySelectorAll('[role=tab]')].map(e=>e.getAttribute('aria-label'))")
assert not any((l or "").startswith("echo://") for l in labels), f"onglet titre par son adresse interne : {labels}"
NEW = [t["url"] for t in targets() if t["url"].startswith("echo://ui/nouvel-onglet")]
assert any("Navigation privée" in (ev("document.body.innerText", u) or "") for u in NEW), "nouvel onglet prive pas neutre"
call(op="navigate", id=active()["id"], url=secret)
until(lambda: active()["url"] == secret and not active()["loading"], "page secrete non chargee")
time.sleep(1)
call(op="open", url=normal)
until(lambda: active()["title"].startswith("normal"), "page normale")
assert active()["title"] == "normal sans cookie", f"cookie prive visible hors navigation privee : {active()['title']}"
ui({"kind": "openPage", "page": "bibliotheque"}); time.sleep(2)
P = "echo://ui/pages.html"
ev("[...document.querySelectorAll('[role=tab]')].find(b=>b.textContent.startsWith('Historique')).click()", P); time.sleep(1)
hist = ev("document.body.innerText", P)
assert "page secrete" not in hist and "normal" in hist, hist[:300]
import os
assert not os.path.exists(os.path.join(os.environ["ECHO_DATA_DIR"], "profile", "conteneur-prive")), "dossier prive ecrit"
print("OK : navigation privee (marque Privé, ni historique ni cookie partage)")
PY
