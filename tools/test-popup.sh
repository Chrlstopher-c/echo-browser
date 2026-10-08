#!/usr/bin/env bash
# Popup (window.open, avec ou sans dimensions) : elle s'ouvre dans un onglet normal d'Echo (sans window.opener : la
# fenetre native serait figee sur about:blank par la pause des service workers d'extensions, choix de Chris du 08/10),
# se ferme seule (window.close) en revenant sur sa page d'origine, sans fermer Echo. Instance isolee, une seule
# fenetre attendue. Pages servies en http ; pour
# un bureau de test sway, passer SWAYSOCK (fenetres comptees dans son arbre) ; sinon Hyprland.
set -euo pipefail
cd "$(dirname "$0")/.."
export ECHO_RUN_DIR="$(mktemp -d)" ECHO_CONTROL_NAME="test-$$" ECHO_DATA_DIR="$(mktemp -d)"
cat > "$ECHO_RUN_DIR/opener.html" <<'H'
<title>origine</title><body style="margin:0;height:100vh"><p id=p>attente</p><script>
addEventListener('message', e => { p.textContent = 'recu:' + e.data })
document.addEventListener('click', () => window.open('popup.html', 'connexion', 'width=500,height=600'))
</script>
H
cat > "$ECHO_RUN_DIR/popup.html" <<'H'
<title>popup</title><p>connexion</p><script>
setTimeout(() => { try { window.opener.postMessage('connecte', '*') } catch (e) {} window.close() }, 2500)
</script>
H
mkdir -p "$ECHO_RUN_DIR/gsi"
cat > "$ECHO_RUN_DIR/orphan-opener.html" <<'H'
<title>origine2</title><body style="margin:0;height:100vh"><p id=p></p><script>
const n = Number(sessionStorage.n || 0) + 1; sessionStorage.n = n; p.textContent = 'chargements=' + n
document.addEventListener('click', () => window.open('gsi/select.html', '_blank', 'noopener'))
</script>
H
cat > "$ECHO_RUN_DIR/gsi/select.html" <<'H'
<title>gsi</title><p>connexion</p><script src="relay.js"></script>
H
echo "setTimeout(() => window.opener.postMessage('jeton', '*'), 1500)" > "$ECHO_RUN_DIR/gsi/relay.js"
cat > "$ECHO_RUN_DIR/lien.html" <<'H'
<title>lien</title><body style="margin:0"><a href="cible.html" target="_blank" style="display:block;height:100vh">ouvrir</a>
H
echo '<title>cible</title><p>cible</p>' > "$ECHO_RUN_DIR/cible.html"
cat > "$ECHO_RUN_DIR/simple-opener.html" <<'H'
<title>simple</title><body style="margin:0;height:100vh"><script>
document.addEventListener('click', () => window.open('cible.html'))
</script>
H
PORT=$((29000 + RANDOM % 900)); export PORT
python3 tools/serveur-test.py "$PORT" "$ECHO_RUN_DIR" >/dev/null 2>&1 & SERVEUR=$!
trap './stop.sh >/dev/null 2>&1 || true; kill $SERVEUR 2>/dev/null || true
      rm -rf "$ECHO_RUN_DIR" "$ECHO_DATA_DIR"' EXIT
./start.sh release >/dev/null; sleep 14
python3 - <<'PY'
import json, os, socket, subprocess, time
sock = socket.socket(socket.AF_UNIX)
sock.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = sock.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
def ready(tab, title):
    for _ in range(40):
        t = [x for x in call(op="tabs")["tabs"] if x["id"] == tab]
        if t and t[0]["title"] == title and not t[0]["loading"]:
            time.sleep(0.5); return
        time.sleep(0.25)
o = call(op="open", url=f"http://localhost:{os.environ['PORT']}/" + "opener.html")["id"]
ready(o, "origine")
call(op="click", x=300, y=300)  # geste utilisateur, comme un clic sur « Continuer avec Google »
time.sleep(1.5)
titles = [t["title"] for t in call(op="tabs")["tabs"]]
assert any("popup" in t or "popup.html" in t for t in titles), f"popup absente des onglets : {titles}"
pid = int(open(os.environ["ECHO_RUN_DIR"] + "/browser.pid").read())
def tree(p):
    out = [p]
    for c in subprocess.run(["pgrep", "-P", str(p)], capture_output=True, text=True).stdout.split():
        out += tree(int(c))
    return out
def windows():
    pids = set(tree(pid))
    if os.environ.get("SWAYSOCK"):  # bureau de test sway (sans ecran) : fenetres lues dans son arbre
        def views(node):
            found = [node] if node.get("pid") in pids and node.get("type") in ("con", "floating_con") else []
            for child in node.get("nodes", []) + node.get("floating_nodes", []):
                found += views(child)
            return found
        return views(json.loads(subprocess.check_output(["swaymsg", "-t", "get_tree"])))
    return [c for c in json.loads(subprocess.check_output(["hyprctl", "clients", "-j"])) if c["pid"] in pids]
wins = windows()
assert len(wins) == 1, f"{len(wins)} fenetres : la popup est sortie du navigateur"
time.sleep(4)
tabs = call(op="tabs")["tabs"]
assert not any("popup" in t["title"] for t in tabs), "la popup ne s'est pas refermee"
assert [t for t in tabs if t["id"] == o][0]["active"], "retour sur la page d'origine attendu apres la fermeture"
# window.open sans dimensions (« ouvrir » en JavaScript) : un onglet qui charge sa page.
cat = f"http://localhost:{os.environ['PORT']}/"
o3 = call(op="open", url=cat + "simple-opener.html")["id"]; ready(o3, "simple")
call(op="click", x=300, y=300); time.sleep(2.5)
assert "cible" in [t["title"] for t in call(op="tabs")["tabs"]], "window.open(adresse) n'a pas charge sa page"
print("OK : window.open sans dimensions charge sa page en onglet")
# Popup coupee de sa page (COOP, cas de LinkedIn + Google) : fermee, page d'origine rechargee.
o2 = call(op="open", url=f"http://localhost:{os.environ['PORT']}/" + "orphan-opener.html")["id"]
ready(o2, "origine2")
call(op="click", x=300, y=300)
time.sleep(5)
tabs = call(op="tabs")["tabs"]
assert not any(t["title"] == "gsi" for t in tabs), f"popup orpheline restee ouverte : {[t['title'] for t in tabs]}"
assert [t for t in tabs if t["id"] == o2][0]["active"], "retour sur la page d'origine attendu"
assert "chargements=2" in call(op="read", id=o2)["text"], call(op="read", id=o2)["text"]
# Lien target=_blank (pubs, « compléter une tâche »…) : nouvel onglet, jamais une fenetre.
li = call(op="open", url=f"http://localhost:{os.environ['PORT']}/" + "lien.html")["id"]; ready(li, "lien")
call(op="click", x=300, y=300); time.sleep(2.5)
titles = [t["title"] for t in call(op="tabs")["tabs"]]
assert "cible" in titles, f"lien _blank non ouvert en onglet : {titles}"
wins = windows()
assert len(wins) == 1, f"{len(wins)} fenetres apres un lien _blank"
print("OK : lien _blank en onglet")
print("OK : popup en onglet, fermeture propre avec retour a l'origine, popup orpheline refermee + origine rechargee")
PY
