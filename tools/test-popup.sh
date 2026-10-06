#!/usr/bin/env bash
# Popup (window.open) : elle s'ouvre en onglet, parle a sa page d'origine (window.opener), puis se ferme
# seule (window.close) sans fermer Echo. Instance isolee, une seule fenetre attendue.
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
setTimeout(() => { window.opener.postMessage('connecte', '*'); window.close() }, 2500)
</script>
H
trap 'cp "$ECHO_RUN_DIR/browser.log" /tmp/claude-1000/popup.log 2>/dev/null; ./stop.sh >/dev/null 2>&1 || true' EXIT
./start.sh release >/dev/null; sleep 14
python3 - <<'PY'
import json, os, socket, subprocess, time
sock = socket.socket(socket.AF_UNIX)
sock.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = sock.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
o = call(op="open", url="file://" + os.environ["ECHO_RUN_DIR"] + "/opener.html")["id"]
time.sleep(2)
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
pids = set(tree(pid))
wins = [c for c in json.loads(subprocess.check_output(["hyprctl", "clients", "-j"])) if c["pid"] in pids]
assert len(wins) == 1, f"{len(wins)} fenetres : la popup est sortie du navigateur"
time.sleep(4)
tabs = call(op="tabs")["tabs"]
assert not any("popup" in t["title"] for t in tabs), "la popup ne s'est pas refermee"
assert "recu:connecte" in call(op="read", id=o)["text"], "le message n'est pas revenu a la page d'origine"
print("OK : popup en onglet, opener garde, fermeture propre")
PY
