//! Responsabilite : servir l'interface du navigateur sous le schema interne `echo://`.
//!
//! Pas de serveur HTTP local : un port ouvert sur la machine serait joignable par n'importe
//! quelle page affichee dans le navigateur, ce qui reviendrait a lui offrir une telecommande.
//! Le schema interne n'est pas atteignable depuis le web.

// Les macros `wrap_*` de CEF exigent les traits `Impl*` et `Wrap*` dans la portee : import global impose.
use cef::*;
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc as StdRc;
use tracing::{debug, warn};

pub const SCHEME: &str = "echo";
/// Hote reserve a l'interface. `echo://ui/index.html` est la page du navigateur.
pub const UI_HOST: &str = "ui";

/// Options du schema, valeurs de `cef_scheme_options_t` :
/// standard (1) + secure (8) + cors enabled (16) + fetch enabled (64).
const SCHEME_OPTIONS: i32 = 1 | 8 | 16 | 64;

/// Racine des fichiers construits de l'interface.
pub fn ui_root() -> PathBuf {
    if let Some(dir) = std::env::var_os("ECHO_UI_DIR") {
        return PathBuf::from(dir);
    }
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("ui")))
        .filter(|dir| dir.is_dir())
        .unwrap_or_else(|| PathBuf::from("ui/dist"))
}

/// Declare le schema aupres de Chromium. A appeler depuis `on_register_custom_schemes`.
pub fn register(registrar: Option<&mut SchemeRegistrar>) {
    let Some(registrar) = registrar else { return };
    registrar.add_custom_scheme(Some(&CefString::from(SCHEME)), SCHEME_OPTIONS);
}

/// Branche la fabrique qui repondra aux requetes `echo://`. A appeler une fois le contexte pret.
pub fn install_factory() {
    let mut factory = UiSchemeFactory::new(ui_root());
    let installed = register_scheme_handler_factory(
        Some(&CefString::from(SCHEME)),
        Some(&CefString::from(UI_HOST)),
        Some(&mut factory),
    );
    if installed == 1 {
        debug!(racine = ?ui_root(), "schema echo:// branche");
    } else {
        warn!("schema echo:// non branche — l'interface ne s'affichera pas");
    }
}

wrap_scheme_handler_factory! {
    struct UiSchemeFactory {
        root: PathBuf,
    }

    impl SchemeHandlerFactory {
        fn create(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _scheme_name: Option<&CefString>,
            request: Option<&mut Request>,
        ) -> Option<ResourceHandler> {
            let request = request?;
            let url = CefString::from(&request.url()).to_string();
            if is_ipc(&url) {
                crate::bridge::submit(&post_body(request));
                return Some(StaticResource::new(b"[]".to_vec(), "application/json".into(), StdRc::new(Cell::new(0))));
            }
            let (body, mime) = load(&self.root, &url);
            Some(StaticResource::new(body, mime, StdRc::new(Cell::new(0))))
        }
    }
}

wrap_resource_handler! {
    struct StaticResource {
        body: Vec<u8>,
        mime: String,
        cursor: StdRc<Cell<usize>>,
    }

    impl ResourceHandler {
        fn open(
            &self,
            _request: Option<&mut Request>,
            handle_request: Option<&mut std::os::raw::c_int>,
            _callback: Option<&mut Callback>,
        ) -> std::os::raw::c_int {
            if let Some(handle) = handle_request {
                *handle = 1;
            }
            1
        }

        fn response_headers(
            &self,
            response: Option<&mut Response>,
            response_length: Option<&mut i64>,
            _redirect_url: Option<&mut CefString>,
        ) {
            if let Some(response) = response {
                response.set_status(if self.body.is_empty() { 404 } else { 200 });
                response.set_mime_type(Some(&CefString::from(self.mime.as_str())));
            }
            if let Some(length) = response_length {
                *length = self.body.len() as i64;
            }
        }

        fn read(
            &self,
            data_out: *mut u8,
            bytes_to_read: std::os::raw::c_int,
            bytes_read: Option<&mut std::os::raw::c_int>,
            _callback: Option<&mut ResourceReadCallback>,
        ) -> std::os::raw::c_int {
            let start = self.cursor.get();
            let remaining = self.body.len().saturating_sub(start);
            let count = remaining.min(bytes_to_read.max(0) as usize);
            if count > 0 && !data_out.is_null() {
                // SAFETY : Chromium fournit un tampon d'au moins `bytes_to_read` octets.
                unsafe { std::ptr::copy_nonoverlapping(self.body[start..].as_ptr(), data_out, count) };
                self.cursor.set(start + count);
            }
            if let Some(read) = bytes_read {
                *read = count as std::os::raw::c_int;
            }
            i32::from(count > 0)
        }
    }
}

/// Chemin reserve aux demandes de l'interface.
fn is_ipc(url: &str) -> bool {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    path.ends_with("/ipc")
}

/// Extrait le corps d'une requete POST envoyee par l'interface.
fn post_body(request: &mut Request) -> Vec<u8> {
    let Some(post) = request.post_data() else { return Vec::new() };
    // Le tampon doit etre dimensionne avant l'appel : CEF lit sa longueur pour savoir
    // combien d'elements rendre. Un vecteur vide fait revenir zero element, et le corps
    // de la demande arrive vide.
    let count = post.element_count();
    let mut elements: Vec<Option<PostDataElement>> = (0..count).map(|_| None).collect();
    post.elements(Some(&mut elements));
    let mut collected = Vec::new();
    for element in elements.into_iter().flatten() {
        let size = element.bytes_count();
        if size == 0 {
            continue;
        }
        let mut buffer = vec![0u8; size];
        let read = element.bytes(size, buffer.as_mut_ptr());
        buffer.truncate(read);
        collected.extend_from_slice(&buffer);
    }
    collected
}

/// Lit le fichier demande sous la racine de l'interface. Toute URL inconnue retombe sur `index.html`.
fn load(root: &Path, url: &str) -> (Vec<u8>, String) {
    let path = resolve(root, url);
    match std::fs::read(&path) {
        Ok(body) => {
            debug!(%url, octets = body.len(), "servi");
            (body, mime_of(&path).to_string())
        }
        Err(_) => match std::fs::read(root.join("index.html")) {
            Ok(body) => (body, "text/html".to_string()),
            Err(err) => {
                warn!(?root, %err, "interface introuvable");
                (Vec::new(), "text/plain".to_string())
            }
        },
    }
}

/// Traduit une URL en chemin, en refusant toute remontee hors de la racine.
fn resolve(root: &Path, url: &str) -> PathBuf {
    let after_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    let path = after_scheme.split(['?', '#']).next().unwrap_or("");
    let path = path.split_once('/').map(|(_, rest)| rest).unwrap_or("");
    let safe: Vec<&str> = path
        .split('/')
        .filter(|part| !part.is_empty() && *part != "." && *part != "..")
        .collect();
    if safe.is_empty() {
        return root.join("index.html");
    }
    root.join(safe.join("/"))
}

fn mime_of(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html",
        Some("js" | "mjs") => "text/javascript",
        Some("css") => "text/css",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        _ => "application/octet-stream",
    }
}
