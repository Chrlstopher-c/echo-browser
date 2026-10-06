#!/usr/bin/env bash
# Regression : une page web ne doit pas pouvoir piloter le coeur par echo://ui/ipc.
# Reussit si l'onglet « pwned » n'apparait pas ET que le refus est journalise.
set -euo pipefail
cd "$(dirname "$0")/.."
WWW="$(mktemp -d)"
cat > "$WWW/csrf.html" <<'HTML'
<title>csrf</title><script>
fetch('echo://ui/ipc',{method:'POST',headers:{'content-type':'text/plain'},
body:JSON.stringify({kind:'newTab',url:'https://example.org/'})}).catch(()=>{});
</script>
HTML
python3 -m http.server 18931 --bind 127.0.0.1 --directory "$WWW" >/dev/null 2>&1 &
SRV=$!
cleanup() { kill "$SRV" 2>/dev/null || true; ./stop.sh >/dev/null 2>&1 || true; rm -rf "$WWW"; }
trap cleanup EXIT
sleep 1
ECHO_DATA_DIR="$(mktemp -d)" ECHO_SLEEP_AFTER_S=99999 ECHO_BENCH_WAKE_AT_S=12 \
  ECHO_BENCH_URLS="http://127.0.0.1:18931/csrf.html" ./start.sh release >/dev/null
sleep 28
LOG="$(sed 's/\x1b\[[0-9;]*m//g' logs/browser.log)"
grep -q "demande de pilotage refusee" <<<"$LOG" || { echo "ECHEC : refus non journalise"; exit 1; }
if grep -q "Example Domain" <<<"$LOG"; then echo "ECHEC : la page a ouvert un onglet"; exit 1; fi
echo "OK : pilotage depuis une page web refuse"
