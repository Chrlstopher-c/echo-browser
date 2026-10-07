//! Responsabilite : les profils (espaces facon Arc). Chacun a sa liste d'onglets et, sauf le profil par
//! defaut, ses propres comptes : un conteneur `profil-<id>` (cookies et stockage a part).

/// Le profil d'origine : il garde le contexte commun, donc les connexions deja faites.
pub const DEFAULT: &str = "graphite";

/// Le conteneur des onglets d'un profil, `None` pour le profil par defaut.
pub fn container_for(space: &str) -> Option<String> {
    (space != DEFAULT && crate::containers::is_valid_id(space)).then(|| format!("profil-{space}"))
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
}
