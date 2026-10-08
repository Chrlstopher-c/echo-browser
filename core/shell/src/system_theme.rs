//! Responsabilite : le theme du bureau (clair ou sombre), lu par le portail XDG, pour le choix « Système » d'Echo.

use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;

/// 0 inconnu, 1 sombre, 2 clair (valeurs du portail `org.freedesktop.appearance color-scheme`).
static SCHEME: AtomicU8 = AtomicU8::new(0);

const POLL: Duration = Duration::from_secs(15);

fn read_portal() -> u8 {
    let output = std::process::Command::new("gdbus")
        .args([
            "call", "--session", "--dest", "org.freedesktop.portal.Desktop",
            "--object-path", "/org/freedesktop/portal/desktop",
            "--method", "org.freedesktop.portal.Settings.Read", "org.freedesktop.appearance", "color-scheme",
        ])
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            if text.contains("uint32 1") { 1 } else if text.contains("uint32 2") { 2 } else { 0 }
        }
        _ => 0,
    }
}

/// Sombre, clair, ou `None` si le bureau n'exprime pas de preference.
pub fn current() -> Option<bool> {
    match SCHEME.load(Ordering::Relaxed) {
        1 => Some(true),
        2 => Some(false),
        _ => None,
    }
}

pub fn publish() {
    crate::bridge::publish(&echo_contract::CoreEvent::SystemScheme { dark: current() });
}

/// Surveille le bureau ; un changement est republie a l'interface.
pub fn watch() {
    SCHEME.store(read_portal(), Ordering::Relaxed);
    std::thread::spawn(|| loop {
        std::thread::sleep(POLL);
        let now = read_portal();
        if SCHEME.swap(now, Ordering::Relaxed) != now {
            crate::containers::later(publish);
        }
    });
}
