#!/usr/bin/env bash
# « Redemarrer » relance vraiment Echo (la boucle de Chromium se termine malgre la fenetre d'ancrage des
# extensions), les onglets reviennent ; puis fermer la fenetre termine le processus (pas de fantome qui
# bloquerait la reouverture). Ancrage actif, comme chez l'utilisateur. Exige le bureau de test sway : SWAYSOCK=…
: "${SWAYSOCK:?SWAYSOCK du bureau de test sway requis}"
source "$(dirname "$0")/banc.sh"
PID=$(cat "$ECHO_RUN_DIR/browser.pid")
uv run -q --with websocket-client python - <<'PY'
import time, banc
banc.call(op="open", url=banc.page("r.html", "<p>relance</p>", title="relance"))
banc.until(lambda: any(t["url"].endswith("/r.html") for t in banc.tabs()), "onglet non ouvert")
time.sleep(2)
banc.ui({"kind": "restartBrowser"})
PY
for _ in $(seq 40); do grep -aq "relance du navigateur" "$ECHO_RUN_DIR/browser.log" && break; sleep 0.5; done
grep -aq "relance du navigateur" "$ECHO_RUN_DIR/browser.log" || { echo "ECHEC : la relance n'a jamais quitte la boucle"; exit 1; }
sleep 10
uv run -q --with websocket-client python - <<'PY'
import banc
banc.until(lambda: any(t["url"].endswith("/r.html") for t in banc.tabs()), "onglet perdu apres la relance", tries=40)
PY
swaymsg "[pid=$PID] kill" >/dev/null
for _ in $(seq 30); do kill -0 "$PID" 2>/dev/null || break; sleep 0.5; done
if kill -0 "$PID" 2>/dev/null; then echo "ECHEC : processus encore vivant apres fermeture de la fenetre"; exit 1; fi
echo "OK : relance effective, onglet garde, fermeture de la fenetre = fin du processus"
