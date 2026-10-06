#!/usr/bin/env bash
# Banc memoire : ouvre les memes pages dans Echo Browser puis dans un Chrome isole,
# et releve le PSS cumule de l'arbre de processus (mesure honnete : les pages partagees comptent une fois).
# Usage : tools/bench-ram.sh echo|chrome [attente_s]
set -euo pipefail
cd "$(dirname "$0")/.."
TARGET="${1:?echo ou chrome}"
WAIT="${2:-70}"
URLS="${BENCH_URLS:-https://fr.wikipedia.org/wiki/Linux https://github.com/torvalds/linux https://www.lemonde.fr https://www.reddit.com https://news.ycombinator.com https://www.youtube.com https://developer.mozilla.org/fr/ https://stackoverflow.com https://www.bbc.com/news https://www.twitch.tv}"

tree() { # pids de l'arbre sous $1
  local p; echo "$1"
  for p in $(pgrep -P "$1" 2>/dev/null || true); do tree "$p"; done
}
pss_mo() { # PSS cumule en Mo des pids sur stdin
  local sum=0 p v
  while read -r p; do
    v=$(awk '/^Pss:/{print $2}' "/proc/$p/smaps_rollup" 2>/dev/null || echo 0)
    sum=$((sum + ${v:-0}))
  done
  echo $((sum / 1024))
}

cleanup() {
  if [ "$TARGET" = "echo" ]; then
    ./stop.sh >/dev/null
  else
    kill "${ROOT:-0}" 2>/dev/null || true
    sleep 3
    rm -rf "${PROFILE:-/nonexistent}"
  fi
}
trap cleanup EXIT

if [ "$TARGET" = "echo" ]; then
  ECHO_DATA_DIR="$(mktemp -d)" ECHO_BENCH_URLS="$URLS" ./start.sh release >/dev/null
  ROOT=$(cat logs/browser.pid)
else
  PROFILE="$(mktemp -d)"
  setsid google-chrome-stable --user-data-dir="$PROFILE" --no-first-run --no-default-browser-check \
    --new-window $URLS >/dev/null 2>&1 < /dev/null &
  sleep 1
  ROOT=$(pgrep -f -- "--user-data-dir=$PROFILE" | head -1)
fi

sleep "$WAIT"
PIDS=$(tree "$ROOT")
echo "$TARGET : $(echo "$PIDS" | wc -l) processus, PSS cumule = $(echo "$PIDS" | pss_mo) Mo"

for p in $PIDS; do
  v=$(awk '/^Pss:/{print int($2/1024)}' "/proc/$p/smaps_rollup" 2>/dev/null || echo 0)
  t=$(tr '\0' ' ' < "/proc/$p/cmdline" 2>/dev/null | grep -oE -- '--type=[a-z-]+' | head -1 || true)
  pp=$(ps -o ppid= -p "$p" | tr -d ' ')
  [ "$t" = "--type=zygote" ] && [ "$pp" != "$ROOT" ] && t="--fils-de-zygote-$pp"
  echo "$v ${t:---type=navigateur}"
done | sort -rn | awk '{n[$2]+=$1; c[$2]++} END{for(k in n) printf "  %-22s %2d proc  %5d Mo\n", k, c[k], n[k]}' | sort -k4 -rn

