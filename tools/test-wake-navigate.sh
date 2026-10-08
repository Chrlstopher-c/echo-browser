#!/usr/bin/env bash
# Une adresse demandee pendant le reveil d'un onglet endormi n'est pas perdue : l'onglet finit sur elle, pas sur sa
# page d'avant la veille. Instance isolee.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import time
from banc import *
a, b, c = (page(f"{n}.html", f"<p>{n}", n) for n in "abc")
call(op="open", url=a); time.sleep(1.5)
call(op="open", url=b); time.sleep(1.5)
ta = [t for t in tabs() if t["url"] == a][0]
call(op="sleep", id=ta["id"])
until(lambda: [t for t in tabs() if t["id"] == ta["id"]][0]["asleep"], "onglet non endormi")
call(op="activate", id=ta["id"])
call(op="navigate", id=ta["id"], url=c)
time.sleep(4)
fin = [t for t in tabs() if t["id"] == ta["id"]][0]
assert fin["url"] == c, f"navigation perdue au reveil : {fin['url']}"
print("OK : navigation pendant le reveil conservee")
PY
