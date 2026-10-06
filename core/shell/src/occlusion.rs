//! Responsabilite : dire a Chromium quand la fenetre n'est plus visible. Wayland ne le signale pas :
//! une fenetre rangee sur un autre espace de travail reste « visible » pour la page, dont les
//! minuteries et les animations ne sont alors pas bridees. Sous Hyprland, on suit les evenements
//! de son socket et on avertit l'onglet affiche.

use cef::*;
use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Evenements qui peuvent changer la visibilite de notre fenetre.
const EVENTS: [&str; 8] = [
    "workspace>>", "workspacev2>>", "activespecial>>", "activespecialv2>>",
    "movewindow>>", "movewindowv2>>", "openwindow>>", "focusedmon>>",
];

fn hypr_dir() -> Option<PathBuf> {
    let signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
    let runtime = std::env::var("XDG_RUNTIME_DIR").ok()?;
    Some(PathBuf::from(runtime).join("hypr").join(signature))
}

/// Interroge Hyprland (`j/clients`, `j/monitors`) : une connexion par requete, comme `hyprctl`.
fn query(dir: &std::path::Path, command: &str) -> Option<Value> {
    let mut stream = UnixStream::connect(dir.join(".socket.sock")).ok()?;
    stream.write_all(command.as_bytes()).ok()?;
    let mut reply = String::new();
    stream.read_to_string(&mut reply).ok()?;
    serde_json::from_str(&reply).ok()
}

/// Vrai si une fenetre du processus est affichee sur un espace de travail actif d'un ecran.
fn is_visible(dir: &std::path::Path) -> Option<bool> {
    let pid = i64::from(std::process::id());
    let clients = query(dir, "j/clients")?;
    let monitors = query(dir, "j/monitors")?;
    let shown: Vec<i64> = monitors
        .as_array()?
        .iter()
        .flat_map(|m| [m["activeWorkspace"]["id"].as_i64(), m["specialWorkspace"]["id"].as_i64()])
        .flatten()
        .collect();
    let window = clients.as_array()?.iter().find(|c| c["pid"].as_i64() == Some(pid))?;
    let on_screen = window["mapped"].as_bool().unwrap_or(true) && !window["hidden"].as_bool().unwrap_or(false);
    Some(on_screen && window["workspace"]["id"].as_i64().is_some_and(|id| shown.contains(&id)))
}

/// Lance l'ecoute des evenements Hyprland. Sans Hyprland, ne fait rien.
pub fn watch() {
    let Some(dir) = hypr_dir() else { return };
    std::thread::spawn(move || {
        let Ok(stream) = UnixStream::connect(dir.join(".socket2.sock")) else {
            warn!("socket d'evenements Hyprland injoignable");
            return;
        };
        info!("visibilite de la fenetre suivie via Hyprland");
        let mut last = true;
        for line in BufReader::new(stream).lines().map_while(Result::ok) {
            if !EVENTS.iter().any(|event| line.starts_with(event)) {
                continue;
            }
            if let Some(visible) = is_visible(&dir) {
                if visible != last {
                    last = visible;
                    apply(visible);
                }
            }
        }
    });
}

fn apply(visible: bool) {
    debug!(visible, "fenetre visible ou masquee");
    let mut task = HiddenTask::new(visible);
    post_task(ThreadId::UI, Some(&mut task));
}

wrap_task! {
    struct HiddenTask {
        visible: bool,
    }

    impl Task {
        fn execute(&self) {
            crate::tabs::WINDOW_HIDDEN.store(!self.visible, std::sync::atomic::Ordering::Relaxed);
            crate::session::with(|s| s.tabs.refresh_visibility());
        }
    }
}
