//! Responsabilite : le theme clair ou sombre que les pages voient (`prefers-color-scheme`). Il suit celui
//! d'Echo, pas celui du systeme : une barre claire a cote d'un site sombre casse l'ensemble.

use cef::{Browser, ImplBrowser, ImplBrowserHost};
use std::sync::atomic::{AtomicBool, Ordering};

static DARK: AtomicBool = AtomicBool::new(true);

/// Impose le theme courant a une page (reste valable d'une navigation a l'autre).
pub fn apply(browser: &Browser) {
    let value = if DARK.load(Ordering::Relaxed) { "dark" } else { "light" };
    let message = serde_json::json!({
        "id": 2,
        "method": "Emulation.setEmulatedMedia",
        "params": {"features": [{"name": "prefers-color-scheme", "value": value}]},
    });
    if let Some(host) = browser.host() {
        host.send_dev_tools_message(Some(message.to_string().as_bytes()));
    }
}

/// Change le theme et l'applique aux pages ouvertes.
pub fn set(dark: bool, browsers: &[Browser]) {
    DARK.store(dark, Ordering::Relaxed);
    for browser in browsers {
        apply(browser);
    }
}
