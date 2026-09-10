//! Responsabilite : les drapeaux Chromium imposes au demarrage, et pourquoi chacun est la.

use cef::{CefString, CommandLine, ImplCommandLine};
use std::path::PathBuf;

/// Vulkan et la plateforme Wayland sont incompatibles dans Chromium : sans ce drapeau,
/// le processus navigateur s'arrete avec le code 28 avant d'avoir ouvert une fenetre.
/// Mesure le 2026-09-10 sur RTX 3060 + Wayland.
const DISABLED_FEATURES: &str = "Vulkan";

/// Identifiant d'application, tel que le gestionnaire de fenetres le voit.
pub const APP_ID: &str = "echo-browser";

/// Repertoire ou l'export CEF a depose libcef.so, les .pak et icudtl.dat.
pub fn cef_dir() -> PathBuf {
    std::env::var_os("CEF_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs_home().join(".local/share/cef"))
}

fn dirs_home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/"))
}

/// Repertoire de travail du navigateur : profil, listes de filtres, extensions.
pub fn data_dir() -> PathBuf {
    std::env::var_os("ECHO_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs_home().join(".local/share/echo-browser"))
}

/// Repertoire des extensions dépaquetées, chargees au demarrage.
pub fn extensions_dir() -> PathBuf {
    data_dir().join("extensions")
}

/// Applique les drapeaux au processus navigateur. `process_type` vide = processus principal.
pub fn apply(process_type: &str, command_line: &mut CommandLine) {
    if !process_type.is_empty() {
        return;
    }
    switch_with_value(command_line, "disable-features", DISABLED_FEATURES);
    // Sans identifiant d'application, le gestionnaire de fenetres ne sait pas ranger la
    // fenetre et la laisse flotter. C'est aussi ce qui porte l'icone dans la barre des taches.
    switch_with_value(command_line, "class", APP_ID);

    if aucun_proxy_declare() {
        command_line.append_switch(Some(&CefString::from("no-proxy-server")));
    }

    let inventaire = echo_extensions::Extensions::new(extensions_dir())
        .with_profile(echo_extensions::profile::default_profile(&data_dir()));
    // `--load-extension` n'est pas repris ici : mesure du 10/09/2026, une extension
    // ainsi chargee est listee, annoncee active, et toutes ses adresses repondent
    // ERR_BLOCKED_BY_CLIENT — y compris avec le mode developpeur. Les extensions
    // passent desormais par une declaration que Chromium installe lui-meme
    // (echo_extensions::external).
    // Ce que l'utilisateur a ecarte ne se desactive pas dans le profil : Chromium y
    // remet sa propre valeur. Il respecte en revanche cette liste-ci.
    if let Some(gardees) = inventaire.enabled_paths() {
        switch_with_value(command_line, "disable-extensions-except", &gardees.join(","));
        tracing::info!(nombre = gardees.len(), "extensions gardees actives");
    }
}

/// Vrai quand rien, dans l'environnement, ne declare de proxy.
///
/// Sans cette indication, Chromium interroge au demarrage un service de configuration
/// que cette machine n'a pas, et la premiere page echoue avant d'etre reprise :
/// `ERR_PROXY_CONNECTION_FAILED` a chaque lancement, sur la page d'accueil. Un proxy
/// declare par variable d'environnement reste evidemment honore.
fn aucun_proxy_declare() -> bool {
    const VARIABLES: [&str; 6] =
        ["http_proxy", "https_proxy", "all_proxy", "HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY"];
    VARIABLES.iter().all(|nom| std::env::var_os(nom).is_none())
}

fn switch_with_value(command_line: &mut CommandLine, name: &str, value: &str) {
    command_line.append_switch_with_value(Some(&CefString::from(name)), Some(&CefString::from(value)));
}

/// Copie le paquet de scriptlets livre avec l'application dans le repertoire de travail,
/// s'il n'y est pas deja. Sans lui, les filtres `+js(...)` restent inertes.
pub fn seed_shield_data() {
    let target = data_dir().join("shield-resources.json");
    if target.exists() {
        return;
    }
    let candidates = [
        std::env::current_exe().ok().and_then(|exe| exe.parent().map(|d| d.join("shield-resources.json"))),
        Some(PathBuf::from("data/shield-resources.json")),
    ];
    for source in candidates.into_iter().flatten() {
        if !source.is_file() {
            continue;
        }
        if let Some(parent) = target.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match std::fs::copy(&source, &target) {
            Ok(octets) => {
                tracing::info!(?source, octets, "paquet de scriptlets installe");
                return;
            }
            Err(err) => tracing::warn!(?source, %err, "paquet de scriptlets non copie"),
        }
    }
    tracing::warn!("aucun paquet de scriptlets trouve — lancer : bun tools/build-resources.mjs");
}
