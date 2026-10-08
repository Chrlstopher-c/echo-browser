//! Extensions propres a chaque profil (regle de Chris : un profil est une identite, rien ne se partage). Chromium
//! installe une extension declaree dans tous ses profils ; le pont interne, present dans chacun, desactive celles qui n'y
//! appartiennent pas. Il les lit dans une marque posee ici dans chaque contexte : le cookie `extensions`.

use cef::*;
use echo_extensions::profiles::{self, Registry};
use tracing::warn;

const MARK_URL: &str = "https://echo-profil.invalid/";
/// Secondes entre 1601 (origine des dates de Chromium) et 1970.
const WINDOWS_EPOCH_OFFSET_S: i64 = 11_644_473_600;
const MARK_LIFETIME_S: i64 = 10 * 365 * 24 * 3600;

pub fn registry() -> Registry {
    crate::session::with(|s| s.extensions.registry(crate::profiles::DEFAULT)).unwrap_or_default()
}

/// Les extensions du profil affiche.
pub fn current_ids() -> Vec<String> {
    let space = crate::session::with(|s| s.tabs.space()).unwrap_or_else(|| crate::profiles::DEFAULT.to_string());
    profiles::ids_for(&registry(), &space).to_vec()
}

/// Enregistre le registre, en donne la copie au pont et lui fait reappliquer la regle dans chaque profil.
pub fn save(registry: &Registry) {
    crate::account::schedule::touch();
    if let Err(err) = crate::session::with(|s| s.extensions.save_registry(registry)).unwrap_or(Ok(())) {
        warn!(%err, "registre des extensions par profil non enregistre");
    }
    crate::containers::for_each_context(|id, context| {
        if let Some(manager) = context.cookie_manager(None) {
            mark(&manager, &crate::profiles::space_of_context(Some(id)));
        }
    });
    if let Some(manager) = cookie_manager_get_global_manager(None) {
        mark(&manager, crate::profiles::DEFAULT);
    }
    let space = crate::session::with(|s| s.tabs.space()).unwrap_or_else(|| crate::profiles::DEFAULT.to_string());
    apply_now(&space);
}

/// Delai laisse a la page du pont pour appliquer la regle avant d'etre refermee.
const APPLY_MS: i64 = 3000;

thread_local! {
    static APPLYING: std::cell::RefCell<Vec<crate::overlay::Overlay>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Fait appliquer la regle tout de suite dans chaque contexte du profil `space` : la page du pont s'y ouvre un instant,
/// sans rien afficher. Sans cela, le pont ne la reapplique qu'au prochain demarrage du profil.
fn apply_now(space: &str) {
    let Some(chrome) = crate::session::with(|s| s.chrome.clone()).flatten() else { return };
    let mut contexts: Vec<Option<RequestContext>> = Vec::new();
    if space == crate::profiles::DEFAULT {
        contexts.push(None);
    }
    crate::containers::for_each_context(|id, context| {
        if crate::profiles::space_of_context(Some(id)) == space {
            contexts.push(Some(context.clone()));
        }
    });
    let rect = Rect { x: 0, y: 0, width: 1, height: 1 };
    for mut context in contexts {
        let page = crate::extension_tabs::apply_page();
        if let Some(overlay) = crate::overlay::Overlay::open_in(&chrome, &page, rect.clone(), context.as_mut()) {
            overlay.set_visible(false);
            APPLYING.with(|list| list.borrow_mut().push(overlay));
        }
    }
    let mut task = CloseApplying::new(0);
    post_delayed_task(ThreadId::UI, Some(&mut task), APPLY_MS);
}

wrap_task! {
    struct CloseApplying {
        unused: i32,
    }

    impl Task {
        fn execute(&self) {
            APPLYING.with(|list| list.borrow_mut().drain(..).for_each(crate::overlay::Overlay::close));
        }
    }
}

/// Au demarrage : premiere ecriture du registre (migration) et marque du contexte commun.
pub fn start() {
    let registry = registry();
    if let Err(err) = crate::session::with(|s| s.extensions.save_registry(&registry)).unwrap_or(Ok(())) {
        warn!(%err, "registre des extensions par profil non enregistre");
    }
    match cookie_manager_get_global_manager(None) {
        Some(manager) => mark(&manager, crate::profiles::DEFAULT),
        None => warn!("pas de gestionnaire de cookies commun : marque du profil principal non posee"),
    }
}

/// Pose dans un contexte la liste des extensions de son profil : le pont y reapplique la regle.
pub fn mark(manager: &CookieManager, space: &str) {
    let ids = profiles::ids_for(&registry(), space).join(",");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let expires = Basetime { val: (now + WINDOWS_EPOCH_OFFSET_S + MARK_LIFETIME_S) * 1_000_000 };
    for (name, value) in [("profil", space.to_string()), ("extensions", ids)] {
        let cookie = Cookie {
            name: CefString::from(name),
            value: CefString::from(value.as_str()),
            path: CefString::from("/"),
            has_expires: 1,
            expires: expires.clone(),
            ..Default::default()
        };
        let mut done = MarkDone::new(space.to_string());
        if manager.set_cookie(Some(&CefString::from(MARK_URL)), Some(&cookie), Some(&mut done)) != 1 {
            warn!(%space, "marque de profil refusee");
        }
    }
}

wrap_set_cookie_callback! {
    struct MarkDone {
        space: String,
    }

    impl SetCookieCallback {
        fn on_complete(&self, success: ::std::os::raw::c_int) {
            if success == 1 {
                tracing::debug!(space = %self.space, "marque de profil posee");
            } else {
                warn!(space = %self.space, "marque de profil non posee : le pont ne separera pas les extensions");
            }
        }
    }
}
