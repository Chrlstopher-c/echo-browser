//! Responsabilite : les drapeaux Chromium imposes au demarrage, et pourquoi chacun est la.

use cef::{CefString, CommandLine, ImplCommandLine};
use std::path::{Path, PathBuf};

/// Vulkan et la plateforme Wayland sont incompatibles dans Chromium : sans ce drapeau,
/// le processus navigateur s'arrete avec le code 28 avant d'avoir ouvert une fenetre.
/// Mesure le 2026-09-10 sur RTX 3060 + Wayland.
const DISABLED_FEATURES: &str = "Vulkan";

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

    let loadable = installed_extensions(&extensions_dir());
    if !loadable.is_empty() {
        switch_with_value(command_line, "load-extension", &loadable.join(","));
        tracing::info!(nombre = loadable.len(), "extensions chargees au demarrage");
    }
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

/// Les dossiers d'extension prets a charger — ceux qui portent un `manifest.json`.
pub fn installed_extensions(dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.join("manifest.json").is_file())
        .filter_map(|path| path.to_str().map(str::to_owned))
        .collect()
}
