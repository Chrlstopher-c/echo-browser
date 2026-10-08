"""Outils communs des tests : socket de pilotage, evaluation dans l'interface ou une page, attente."""
import json, os, socket, time, urllib.request

import websocket

PORT, DT = os.environ["PORT"], os.environ["ECHO_DEVTOOLS_PORT"]
BASE = f"http://localhost:{PORT}"
UI = "echo://ui/index.html"


def _connect():
    s = socket.socket(socket.AF_UNIX)
    s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
    return s.makefile("rw")


_f = _connect()


def call(**request):
    _f.write(json.dumps(request) + "\n"); _f.flush()
    return json.loads(_f.readline())


def ui(request):
    """Envoie une requete de l'interface au coeur (comme un clic dans la barre)."""
    return call(op="ui", request=request)


def setting(key, kind, value):
    return ui({"kind": "updateSetting", "key": key, "value": {"type": kind, "value": value}})


def targets():
    return json.load(urllib.request.urlopen(f"http://127.0.0.1:{DT}/json/list"))


def ev(js, where=UI):
    """Evalue `js` dans la premiere cible dont l'adresse commence par `where`."""
    t = [t for t in targets() if t["url"].startswith(where)][0]
    ws = websocket.create_connection(t["webSocketDebuggerUrl"], suppress_origin=True)
    ws.send(json.dumps({"id": 1, "method": "Runtime.evaluate",
                        "params": {"expression": js, "returnByValue": True, "awaitPromise": True}}))
    while (m := json.loads(ws.recv())).get("id") != 1:
        pass
    ws.close()
    return m["result"]["result"].get("value")


def until(cond, label, tries=30, pause=0.5):
    last = None
    for _ in range(tries):
        try:
            if cond():
                return
        except Exception as error:  # la cible peut ne pas exister encore
            last = error
        time.sleep(pause)
    raise AssertionError(f"{label}{f' ({last})' if last else ''}")


def tabs():
    return call(op="tabs")["tabs"]


def active():
    return [t for t in tabs() if t["active"]][0]


def page(name, body, title=None):
    """Ecrit une page de test dans le dossier servi et rend son adresse."""
    with open(os.path.join(os.environ["W"], name), "w") as out:
        out.write(f"<!doctype html><meta charset=utf-8><title>{title or name}</title>{body}")
    return f"{BASE}/{name}"


ADDRESS = "document.querySelector('input[aria-label=\"Adresse\"]')"


def type_address(text):
    """Tape dans l'adresse. Le banc n'a pas de clavier : le focus systeme est simule."""
    ev(f"(()=>{{const i={ADDRESS};i.focus();if(!document.hasFocus())i.dispatchEvent(new FocusEvent('focusin',"
       f"{{bubbles:true}}));const set=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set;"
       f"set.call(i,{json.dumps(text)});i.dispatchEvent(new Event('input',{{bubbles:true}}));return 1}})()")


def press_in_address(key, shift=False):
    ev(f"(()=>{{{ADDRESS}.dispatchEvent(new KeyboardEvent('keydown',{{key:{json.dumps(key)},shiftKey:{str(shift).lower()},"
       f"bubbles:true}}));return 1}})()")


def options():
    return ev("[...document.querySelectorAll('[role=option]')].map(o=>o.dataset.kind+'|'+o.getAttribute('aria-selected')"
              "+'|'+o.textContent)")


def open_page(name):
    """Ouvre une page d'Echo (reglages, aide, bibliotheque) dans un onglet et attend sa cible."""
    call(op="open", url=f"echo://ui/{name}.html")
    until(lambda: any(t["url"].startswith(f"echo://ui/{name}") for t in targets()), f"page {name} absente")
    time.sleep(1)
