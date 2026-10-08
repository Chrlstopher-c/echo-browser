"""Serveur de fichiers pour les essais : comme `http.server`, avec les requetes partielles (Range) que les medias
exigent pour se positionner. Usage : python3 tools/serveur-test.py <port> <dossier>"""
import os
import re
import sys
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer


class Handler(SimpleHTTPRequestHandler):
    def send_head(self):
        path = self.translate_path(self.path)
        match = re.match(r"bytes=(\d+)-(\d*)", self.headers.get("Range", ""))
        if not match or not os.path.isfile(path):
            return super().send_head()
        size = os.path.getsize(path)
        start = int(match.group(1))
        end = int(match.group(2)) if match.group(2) else size - 1
        f = open(path, "rb")
        f.seek(start)
        self.send_response(206)
        self.send_header("Content-Type", self.guess_type(path))
        self.send_header("Content-Range", f"bytes {start}-{end}/{size}")
        self.send_header("Content-Length", str(end - start + 1))
        self.send_header("Accept-Ranges", "bytes")
        self.end_headers()
        return f

    def end_headers(self):
        self.send_header("Accept-Ranges", "bytes")
        super().end_headers()


if __name__ == "__main__":
    port, folder = int(sys.argv[1]), sys.argv[2]
    ThreadingHTTPServer(("", port), partial(Handler, directory=folder)).serve_forever()
