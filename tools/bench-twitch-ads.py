# Banc pubs Twitch (08/10/2026) : ouvre une chaine en direct dans un onglet d'Echo et observe 70 s — etiquette de pub
# affichee (relevee toutes les 2 s) et avancee de la video. Profil neuf = pub d'arrivee quasi systematique.
# Usage : uv run --with websocket-client python tools/bench-twitch-ads.py <port debogage> <prise de controle> <chaine> [script.js]
# (instance isolee, ECHO_DEVTOOLS_PORT fixe ; script.js optionnel injecte au debut de la page, pour comparer).
import json, os, socket, sys, threading, time, urllib.request, websocket
port, ctl, channel = sys.argv[1], sys.argv[2], sys.argv[3]
script = open(sys.argv[4]).read() if len(sys.argv) > 4 else None
s = socket.socket(socket.AF_UNIX); s.connect(ctl); f = s.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
for t in call(op="tabs")["tabs"]:
    if "twitch.tv" in t["url"]: call(op="close", id=t["id"])
time.sleep(2)
call(op="open", url=f"https://www.twitch.tv/{channel}"); time.sleep(5)
page = [t for t in json.load(urllib.request.urlopen(f"http://127.0.0.1:{port}/json/list")) if t["type"] == "page" and f"twitch.tv/{channel}" in t["url"]][-1]
ws = websocket.create_connection(page["webSocketDebuggerUrl"], suppress_origin=True)
n = [0]; lock = threading.Lock(); results = {}; playlists = [0]
def send(method, params=None, session=None):
    with lock:
        n[0] += 1; m = {"id": n[0], "method": method, "params": params or {}}
        if session: m["sessionId"] = session
        ws.send(json.dumps(m)); return n[0]
def reader():
    while True:
        try: m = json.loads(ws.recv())
        except Exception: return
        if "id" in m: results[m["id"]] = m; continue
        meth, sid = m.get("method"), m.get("sessionId")
        if meth == "Target.attachedToTarget":
            ws2 = m["params"]["sessionId"]
            send("Network.enable", {}, ws2); send("Runtime.enable", {}, ws2); send("Runtime.runIfWaitingForDebugger", {}, ws2)
        elif meth == "Network.responseReceived" and ".m3u8" in m["params"]["response"]["url"]:
            # Le nombre de listes dit l'activite du lecteur (vaft en va chercher d'autres pendant une pub). Leur contenu
            # n'est pas lisible ici : getResponseBody rend vide dans le worker du lecteur.
            playlists[0] += 1
threading.Thread(target=reader, daemon=True).start()
send("Target.setAutoAttach", {"autoAttach": True, "waitForDebuggerOnStart": True, "flatten": True})
send("Network.enable")
send("Runtime.enable")
send("Page.enable")
if script: send("Page.addScriptToEvaluateOnNewDocument", {"source": script}); time.sleep(0.5)
send("Page.reload", {"ignoreCache": True})
time.sleep(8)
send("Runtime.evaluate", {"expression": "(()=>{const v=document.querySelector('video'); if(v){v.muted=true; return v.play()}})()", "userGesture": True, "awaitPromise": True})
ads, samples, times = 0, 0, []
probe = ("JSON.stringify({ad: !!document.querySelector('[data-a-target=\"video-ad-label\"],[data-a-target=\"video-ad-countdown\"]')"
         ",t: (document.querySelector('video')||{}).currentTime||0})")
for _ in range(31):
    i = send("Runtime.evaluate", {"expression": probe, "returnByValue": True})
    for _ in range(30):
        if i in results: break
        time.sleep(0.1)
    v = json.loads(results.get(i, {}).get("result", {}).get("result", {}).get("value", "{}") or "{}")
    samples += 1; ads += 1 if v.get("ad") else 0; times.append(round(v.get("t", 0), 1))
    time.sleep(2)
print(json.dumps({"chaine": channel, "vaft": bool(script), "listes": playlists[0], "listes_avec_pub": stitched[0],
                  "echantillons_pub_affichee": ads, "echantillons": samples, "video_t": [times[0], times[len(times)//2], times[-1]]}))
