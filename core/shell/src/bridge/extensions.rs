//! Responsabilite : les demandes de l'interface qui touchent aux extensions —
//! installation, inventaire, fenetres et pages de reglages.

use super::publish::publish_tabs;
use super::{
    mark_restart_needed, notify_error, open_tab, publish, session, RESTART_NEEDED,
};
use echo_contract::CoreEvent;
use tracing::info;

/// Installe une extension sans quitter le navigateur.
///
/// Passer par la fiche du catalogue ne marche pas : la page reconnait qu'elle ne parle
/// pas au vrai Chrome et renvoie l'utilisateur vers l'application installee sur la
/// machine — une autre fenetre, un autre navigateur, une extension qui atterrit ailleurs.
/// On telecharge donc le paquet et on le depaquette nous-memes. Il devient une extension
/// a nous, chargee au demarrage, que notre gestionnaire pilote entierement.
pub(super) fn install_extension(source: &str) {
    let Some(id) = echo_extensions::catalog::extract_id(source) else {
        notify_error(&format!("installation impossible : aucun identifiant d'extension dans « {source} »"));
        return;
    };
    let installed = session::with(|s| s.extensions.list().iter().any(|e| e.id == id)).unwrap_or(false);
    if !installed {
        match session::with(|s| s.extensions.install(source)) {
            Some(Ok(_)) => {
                mark_restart_needed();
                info!(%id, "extension declaree");
            }
            Some(Err(err)) => return notify_error(&format!("installation impossible : {err}")),
            None => return notify_error("installation impossible : le navigateur est occupé."),
        }
    }
    let mut registry = crate::extension_profiles::registry();
    let space = session::with(|s| s.tabs.space()).unwrap_or_else(|| crate::profiles::DEFAULT.to_string());
    let added = echo_extensions::profiles::add(&mut registry, &space, &id);
    crate::extension_profiles::save(&registry);
    let message = match (installed, added) {
        (false, _) => "Extension ajoutée à ce profil — elle s'installe à la relance.",
        (true, true) => "Extension ajoutée à ce profil.",
        (true, false) => "Cette extension est déjà dans ce profil.",
    };
    publish(&CoreEvent::notice(echo_contract::NoticeLevel::Info, message));
    publish_extensions();
}

/// Retire une extension du catalogue du profil affiche. Plus aucun profil ne la garde : sa declaration est retiree et
/// Chromium la desinstalle a la relance.
pub(super) fn remove_from_profile(id: &str) {
    let mut registry = crate::extension_profiles::registry();
    let space = session::with(|s| s.tabs.space()).unwrap_or_else(|| crate::profiles::DEFAULT.to_string());
    let kept_elsewhere = echo_extensions::profiles::remove(&mut registry, &space, id);
    crate::extension_profiles::save(&registry);
    if !kept_elsewhere {
        let profile_root = echo_extensions::profile::default_profile(&crate::flags::data_dir());
        let withdrawn = profile_root.parent().map(|root| echo_extensions::external::withdraw(root, id));
        if let Some(Err(err)) = withdrawn {
            notify_error(&format!("désinstallation incomplète : {err}"));
        }
        mark_restart_needed();
    }
    publish_extensions();
}

/// Ouvre le catalogue dans un onglet, pour y chercher une extension a installer.
pub(super) fn open_store(source: &str) {
    let target = echo_extensions::catalog::extract_id(source)
        .map(|id| echo_extensions::Extensions::store_page(&id))
        .unwrap_or_else(|| CATALOG_HOME.to_string());
    info!(%target, "ouverture du catalogue");
    open_tab(&target);
    publish_tabs();
}

/// Page d'accueil du catalogue.
const CATALOG_HOME: &str = "https://chromewebstore.google.com/";

/// Diffuse l'inventaire des extensions.
pub fn publish_extensions() {
    let pending = RESTART_NEEDED.load(std::sync::atomic::Ordering::SeqCst);
    let Some(extensions) = session::with(|s| s.extensions.list()) else { return };
    // Un profil ne montre que ses extensions ; nos propres paquets (ligne de commande) restent visibles partout.
    let mine = crate::extension_profiles::current_ids();
    let view = extensions
        .into_iter()
        .filter(|extension| !crate::extension_tabs::is_pont(&extension.id))
        .filter(|extension| extension.from_command_line || mine.contains(&extension.id))
        .map(|extension| {
            let url = |path: &str| echo_extensions::action::resource_url(&extension.id, path);
            echo_contract::ExtensionView {
                name: extension.name.clone(),
                version: extension.version.clone(),
                enabled: extension.enabled,
                // Toute bascule attend la relance : Chromium ne sait pas desactiver une
                // extension a chaud, il sait n'en charger qu'une liste au demarrage.
                // Une extension sans version est declaree mais pas encore installee.
                pending: pending || extension.version.is_empty(),
                removable: true,
                // L'icone passe par notre schema : celui de l'extension ne se lit pas
                // depuis une page interne (voir assets::ICON_HOST).
                icon: extension
                    .action
                    .icon
                    .as_ref()
                    .map(|_| format!("echo://{}/{}", crate::assets::ICON_HOST, extension.id)),
                popup: extension.action.popup.as_deref().map(&url),
                options: extension.action.options.as_deref().map(&url),
                description: extension.description.clone(),
                permissions: extension.permissions.clone(),
                id: extension.id.clone(),
            }
        })
        .collect();
    publish(&CoreEvent::ExtensionsChanged { extensions: view, restart_pending: pending });
}

/// Ouvre la fenetre d'une extension sous son icone, ou la referme si c'est deja elle.
///
/// L'adresse de la fenetre vient du manifeste, jamais de l'interface : une adresse
/// choisie par la page afficherait n'importe quoi au-dessus du contenu.
pub(super) fn open_extension_popup(id: &str, anchor: echo_contract::AnchorRect) {
    let found = session::with(|s| {
        s.extensions.list().into_iter().find(|extension| extension.id == id)
    })
    .flatten();
    let Some(extension) = found else {
        tracing::warn!(%id, "fenetre d'extension : extension inconnue");
        return;
    };
    let Some(path) = extension.action.popup.as_deref() else {
        tracing::warn!(%id, "cette extension ne declare pas de fenetre");
        return;
    };
    let url = echo_extensions::action::resource_url(id, path);

    // L'appel a Chromium se fait hors de tout acces a l'etat : il rappelle le programme
    // pendant la creation de la vue.
    let Some(chrome) = session::with(|s| s.chrome.clone()).flatten() else { return };
    let rect = cef::Rect { x: anchor.x, y: anchor.y, width: anchor.width, height: anchor.height };
    crate::overlay::toggle_extension_popup(id, &url, rect, &chrome);
    publish(&CoreEvent::ExtensionPopupChanged { id: crate::overlay::open_popup_id() });
}

/// Ouvre la page de reglages d'une extension dans un onglet.
pub(super) fn open_extension_options(id: &str) {
    let page = session::with(|s| {
        s.extensions
            .list()
            .into_iter()
            .find(|extension| extension.id == id)
            .and_then(|extension| extension.action.options.clone())
    })
    .flatten();
    let Some(page) = page else {
        notify_error("Cette extension n'a pas de page de réglages.");
        return;
    };
    open_tab(&echo_extensions::action::resource_url(id, &page));
}
