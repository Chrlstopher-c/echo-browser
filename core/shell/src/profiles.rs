//! Responsabilite : les profils (espaces facon Arc). Chacun a sa liste d'onglets et, sauf le profil par
//! defaut, ses propres comptes : un conteneur `profil-<id>` (cookies et stockage a part).

use cef::{ImplCookieManager, ImplRequestContext};

/// Le profil d'origine : il garde le contexte commun, donc les connexions deja faites.
pub const DEFAULT: &str = "graphite";

/// Le conteneur des onglets d'un profil, `None` pour le profil par defaut.
/// Les quatre profils d'origine (teintes) dont un dossier existe deja : ils ont servi, on les garde visibles.
pub fn legacy_with_data() -> Vec<String> {
    let dir = crate::flags::data_dir().join("profile");
    ["sable", "rose", "foret", "ardoise"]
        .into_iter()
        .filter(|id| dir.join(format!("conteneur-profil-{id}")).is_dir())
        .map(str::to_string)
        .collect()
}

pub fn container_for(space: &str) -> Option<String> {
    (space != DEFAULT && crate::containers::is_valid_id(space)).then(|| format!("profil-{space}"))
}

/// Separe, dans l'identifiant d'un contexte, le profil du conteneur choisi dans ce profil.
const SCOPE: &str = "--";

/// Le contexte d'un conteneur choisi par l'utilisateur dans le profil `space`. Un conteneur appartient a son profil :
/// « Compte 2 » d'un profil ne partage pas ses cookies avec « Compte 2 » d'un autre. Le profil par defaut garde
/// l'identifiant nu (contextes deja existants).
pub fn scope_container(space: &str, container: &str) -> String {
    match container_for(space) {
        Some(base) if !container.starts_with("profil-") => format!("{base}{SCOPE}{container}"),
        _ => container.to_string(),
    }
}

/// Le profil auquel appartient un contexte (`None` : le contexte commun, profil par defaut).
pub fn space_of_context(context: Option<&str>) -> String {
    context
        .and_then(|id| id.strip_prefix("profil-"))
        .map(|rest| rest.split(SCOPE).next().unwrap_or(rest).to_string())
        .unwrap_or_else(|| DEFAULT.to_string())
}

/// Passe au profil `id` : on y retrouve son dernier onglet, ou une page d'accueil s'il est vide.
pub fn switch(id: &str) {
    if !crate::containers::is_valid_id(id) {
        return;
    }
    let same = crate::session::with(|s| s.tabs.space() == id).unwrap_or(false);
    crate::session::with(|s| s.tabs.set_space(id));
    let target = crate::session::with(|s| s.tabs.last_in_space(id)).flatten();
    match target {
        Some(tab) => crate::bridge::select_tab(tab),
        None if !same => crate::bridge::open_tab(crate::search::HOME),
        None => {}
    }
    crate::bridge::publish_tabs();
    crate::bridge::publish_extensions();
    if !same {
        // Favoris et historique sont ceux du profil : la Bibliotheque et la barre suivent.
        crate::bridge::publish_bookmarks();
        crate::bridge::publish_history("");
    }
}

/// Dossiers de profils a effacer au prochain lancement (un contexte ouvert ne peut pas l'etre a chaud).
fn pending_wipe_path() -> std::path::PathBuf {
    crate::flags::data_dir().join("profils-a-effacer.json")
}

/// Au lancement, avant toute creation de contexte : efface les dossiers des profils supprimes ou reinitialises.
pub fn wipe_pending() {
    let path = pending_wipe_path();
    let Ok(raw) = std::fs::read(&path) else { return };
    let ids: Vec<String> = serde_json::from_slice(&raw).unwrap_or_default();
    let root = crate::flags::data_dir().join("profile");
    let Ok(entries) = std::fs::read_dir(&root) else { return };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let matches = ids.iter().any(|id| {
            let base = format!("conteneur-profil-{id}");
            name == base || name.starts_with(&format!("{base}{SCOPE}"))
        });
        if matches && let Err(err) = std::fs::remove_dir_all(entry.path()) {
            tracing::warn!(%name, %err, "dossier de profil non efface");
        }
    }
    let _ = std::fs::remove_file(path);
    tracing::info!(profils = ids.len(), "donnees de profils effacees");
}

/// Le contexte (ou un conteneur) appartient-il au profil `id` ?
fn belongs_to(context: &str, id: &str) -> bool {
    let base = format!("profil-{id}");
    context == base || context.starts_with(&format!("{base}{SCOPE}"))
}

/// Supprime (`delete`) ou reinitialise un profil : ses onglets se ferment, ses cookies sont effaces tout de suite (plus
/// aucune session ouverte), le reste de ses donnees au prochain lancement. Le profil principal ne se supprime pas.
pub fn forget(id: &str, delete: bool) {
    if id == DEFAULT || !crate::containers::is_valid_id(id) {
        return;
    }
    if crate::session::with(|s| s.tabs.space() == id).unwrap_or(false) {
        switch(DEFAULT);
    }
    let tabs: Vec<echo_contract::TabId> =
        crate::session::with(|s| s.tabs.iter().filter(|t| t.space == id).map(|t| t.id).collect()).unwrap_or_default();
    for tab in tabs {
        crate::bridge::close_tab(tab);
    }
    crate::containers::for_each_context(|context, request_context| {
        if belongs_to(context, id)
            && let Some(manager) = request_context.cookie_manager(None)
        {
            manager.delete_cookies(None, None, None);
        }
    });
    let path = pending_wipe_path();
    let mut ids: Vec<String> = std::fs::read(&path).ok().and_then(|r| serde_json::from_slice(&r).ok()).unwrap_or_default();
    if !ids.iter().any(|known| known == id) {
        ids.push(id.to_string());
    }
    if let Err(err) = std::fs::write(&path, serde_json::to_vec(&ids).unwrap_or_default()) {
        tracing::warn!(%err, "effacement du profil non programme");
    }
    if delete {
        let mut registry = crate::extension_profiles::registry();
        if registry.remove(id).is_some() {
            crate::extension_profiles::save(&registry);
        }
    }
    tracing::info!(%id, delete, "profil oublie");
    crate::bridge::publish_tabs();
}
