"""Serveur MCP d'Echo Browser : Claude liste, lit, ouvre et ferme les onglets du navigateur de Chris.

Parle a la prise Unix du navigateur ($XDG_RUNTIME_DIR/echo-browser/control.sock), jamais au reseau.
"""

import asyncio
import json
import os
import socket
from typing import Any

from loguru import logger
from mcp.server.mcpserver import MCPServer
from mcp.server.mcpserver.exceptions import ToolError

server = MCPServer(
    name="echo-browser",
    instructions=(
        "Pilote le navigateur de Chris. Lire d'abord browser_tabs ; browser_read rend le texte visible d'un onglet "
        "(l'actif par defaut). Un onglet 'asleep' doit etre active avant d'etre lu."
    ),
)

TIMEOUT_S = 15.0


def _socket_path() -> str:
    # Un client MCP ne transmet pas toujours l'environnement complet : on retombe sur le dossier standard.
    runtime = os.environ.get("XDG_RUNTIME_DIR") or f"/run/user/{os.getuid()}"
    return os.path.join(runtime, "echo-browser", "control.sock")


def _call_blocking(request: dict[str, Any]) -> dict[str, Any]:
    path = _socket_path()
    try:
        with socket.socket(socket.AF_UNIX) as sock:
            sock.settimeout(TIMEOUT_S)
            sock.connect(path)
            sock.sendall((json.dumps(request) + "\n").encode())
            reply = json.loads(sock.makefile("r", encoding="utf-8").readline())
    except (FileNotFoundError, ConnectionRefusedError) as error:
        logger.warning("navigateur injoignable : {}", error)
        raise ToolError("Echo Browser n'est pas lance (./start.sh release dans le projet echo-browser)") from error
    except (OSError, ValueError) as error:
        logger.error("echange avec le navigateur en echec : {}", error)
        raise ToolError(f"echange avec le navigateur impossible : {error}") from error
    if not reply.get("ok"):
        raise ToolError(str(reply.get("error", "erreur inconnue")))
    return reply


async def _call(**request: Any) -> dict[str, Any]:
    return await asyncio.to_thread(_call_blocking, request)


@server.tool()
async def browser_tabs() -> list[dict[str, Any]]:
    """Liste les onglets ouverts : id, titre, adresse, actif, endormi (asleep), en chargement."""
    return (await _call(op="tabs"))["tabs"]


@server.tool()
async def browser_read(id: int | None = None) -> dict[str, Any]:
    """Rend le titre, l'adresse et le texte visible d'un onglet (l'actif si id est omis)."""
    request: dict[str, Any] = {"op": "read"}
    if id is not None:
        request["id"] = id
    reply = await _call(**request)
    return {key: reply[key] for key in ("id", "title", "url", "loading", "text")}


@server.tool()
async def browser_open(url: str) -> dict[str, Any]:
    """Ouvre une adresse (ou une recherche) dans un nouvel onglet et rend son id."""
    return {"id": (await _call(op="open", url=url))["id"]}


@server.tool()
async def browser_navigate(id: int, url: str) -> str:
    """Charge une adresse dans un onglet existant. Relire ensuite avec browser_read (loading=false)."""
    await _call(op="navigate", id=id, url=url)
    return "ok"


@server.tool()
async def browser_activate(id: int) -> str:
    """Affiche un onglet, en le reveillant s'il dort."""
    await _call(op="activate", id=id)
    return "ok"


@server.tool()
async def browser_sleep(id: int) -> str:
    """Endort un onglet inactif pour liberer sa memoire (refuse pour l'onglet actif ou qui joue du son)."""
    await _call(op="sleep", id=id)
    return "ok"


@server.tool()
async def browser_close(id: int) -> str:
    """Ferme un onglet."""
    await _call(op="close", id=id)
    return "ok"


if __name__ == "__main__":
    server.run()
