#!/usr/bin/env bash
# Telechargements : la progression s'affiche au pied de la barre pendant l'arrivee, puis une bulle « Téléchargé »
# propose « Ouvrir » et « Afficher dans le dossier » ; le fichier est bien ecrit. Instance isolee.
cd "$(dirname "$0")/.."
BANC_NO_START=1 source tools/banc.sh
export XDG_DOWNLOAD_DIR="$W/recus"; mkdir -p "$XDG_DOWNLOAD_DIR"
LENT=$((PORT + 600))
python3 - "$LENT" >/dev/null 2>&1 <<'PY' & LENT_PID=$!
import http.server, sys, time
class Lent(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header("Content-Type", "application/octet-stream")
        self.send_header("Content-Disposition", "attachment; filename=archive.bin")
        self.send_header("Content-Length", str(40 * 65536))
        self.end_headers()
        for _ in range(40):
            self.wfile.write(b"x" * 65536); self.wfile.flush(); time.sleep(0.15)
    def log_message(self, *a): pass
http.server.HTTPServer(("127.0.0.1", int(sys.argv[1])), Lent).serve_forever()
PY
trap './stop.sh >/dev/null 2>&1 || true; kill $SERVEUR $LENT_PID 2>/dev/null || true; rm -rf "$W"' EXIT
./start.sh release >/dev/null; sleep 8
LENT=$LENT uv run -q --with websocket-client python - <<'PY'
import os, time
from banc import *
call(op="open", url=f"http://127.0.0.1:{os.environ['LENT']}/archive.bin")
STRIP = "document.querySelector('[aria-label=\"Téléchargements en cours\"]')"
until(lambda: "archive.bin" in (ev(f"{STRIP}?.innerText") or ""), "pas de progression visible", tries=20)
pct = ev(f"{STRIP}.innerText")
print(f"en cours : {pct!r}")
status = lambda: ev("[...document.querySelectorAll('[role=status]')].map(e=>e.innerText).join('|')") or ""
until(lambda: "Téléchargé : archive.bin" in status(), f"pas de bulle de fin : {status()}", tries=40)
assert "Ouvrir" in status() and "Afficher dans le dossier" in status(), status()
assert not ev(f"!!{STRIP}"), "la progression reste affichee apres la fin"
taille = os.path.getsize(os.path.join(os.environ["XDG_DOWNLOAD_DIR"], "archive.bin"))
assert taille == 40 * 65536, taille
print("OK : progression, bulle de fin avec Ouvrir / Afficher, fichier ecrit")
PY
