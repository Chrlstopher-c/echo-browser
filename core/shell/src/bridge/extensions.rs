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
    let outcome = session::with(|s| s.extensions.install(source));
    match outcome {
        Some(Ok(id)) => {
            mark_restart_needed();
            info!(%id, "extension declaree");
            publish(&CoreEvent::Notice {
                level: echo_contract::NoticeLevel::Info,
                message: "Extension ajoutée — elle s'installe à la relance.".to_string(),
            });
        }
        Some(Err(err)) => notify_error(&format!("installation impossible : {err}")),
        None => notify_error("installation impossible : le navigateur est occupé."),
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
    let view = extensions
        .into_iter()
        .filter(|extension| !crate::extension_tabs::is_pont(&extension.id))
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
