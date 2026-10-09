#!/usr/bin/env bash
# Cadenas honnete : une page http affiche « Non sécurisé » dans l'adresse ; un certificat refuse (auto-signe) donne
# l'etat « certificat refusé », jamais « Connexion chiffrée » ; le panneau resume en langage courant. Instance isolee.
cd "$(dirname "$0")/.."
BANC_NO_START=1 source tools/banc.sh
TLS=$((PORT + 700))
openssl req -x509 -newkey rsa:2048 -nodes -keyout "$W/cle.pem" -out "$W/cert.pem" -days 1 -subj "/CN=faux.example" 2>/dev/null
python3 - "$TLS" "$W" >/dev/null 2>&1 <<'PY' & TLS_PID=$!
import http.server, ssl, sys
srv = http.server.HTTPServer(("127.0.0.1", int(sys.argv[1])), http.server.SimpleHTTPRequestHandler)
ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER); ctx.load_cert_chain(f"{sys.argv[2]}/cert.pem", f"{sys.argv[2]}/cle.pem")
srv.socket = ctx.wrap_socket(srv.socket, server_side=True); srv.serve_forever()
PY
trap './stop.sh >/dev/null 2>&1 || true; kill $SERVEUR $TLS_PID 2>/dev/null || true; rm -rf "$W"' EXIT
./start.sh release >/dev/null; sleep 8
TLS=$TLS uv run -q --with websocket-client python - <<'PY'
import os, time
from banc import *
http_url = page("clair.html", "<p>page en clair", "en clair")
call(op="open", url=http_url)
until(lambda: active()["url"] == http_url and not active()["loading"], "page http non chargee")
label = lambda: ev("document.querySelector('[data-unsafe]')?.getAttribute('data-unsafe')+'|'+document.querySelector('[data-unsafe]')?.textContent")
until(lambda: label() == "insecure|Non sécurisé", f"http : {label()}")
print(f"http : {label()}")
bad = f"https://127.0.0.1:{os.environ['TLS']}/"
call(op="open", url=bad); time.sleep(4)
until(lambda: label().startswith("invalid|"), f"certificat refuse, adresse : {label()}")
lock = ev("document.querySelector('[aria-label=\"Sécurité du site\"]')?.getAttribute('title')")
assert "chiffrée" not in lock and "refusé" in lock, lock
ui({"kind": "openSidebarSheet", "sheet": "network"})
until(lambda: "certificat refusé" in (ev("document.querySelector('[data-summary]')?.innerText") or ""), "resume du panneau")
print(f"OK : cadenas honnete (http, certificat refuse : {lock!r}), resume du panneau")
PY
