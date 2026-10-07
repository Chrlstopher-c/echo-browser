//! Responsabilite : les conteneurs d'onglets. Chacun a son propre contexte Chromium — cookies,
//! stockage local, cache — donc ses propres comptes : deux onglets de conteneurs differents ne
//! partagent aucune session, meme sur le meme site.

use cef::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use tracing::warn;

const MAX_ID_LEN: usize = 48;

thread_local! {
    /// Un contexte par conteneur, cree a la premiere demande et garde tant que le navigateur vit.
    static CONTEXTS: RefCell<HashMap<String, RequestContext>> = RefCell::new(HashMap::new());

    /// Les conteneurs dont le profil est pret : Chromium l'initialise de facon asynchrone, et refuse
    /// d'y ouvrir une vue avant.
    static READY: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

wrap_request_context_handler! {
    struct ReadyHandler {
        id: String,
    }

    impl RequestContextHandler {
        fn on_request_context_initialized(&self, request_context: Option<&mut RequestContext>) {
            READY.with(|set| set.borrow_mut().insert(self.id.clone()));
            if let Some(manager) = request_context.and_then(|context| context.cookie_manager(None)) {
                crate::extension_profiles::mark(&manager, &crate::profiles::space_of_context(Some(&self.id)));
            }
        }
    }
}

/// Vrai quand le contexte du conteneur peut porter une vue. Le cree au passage s'il n'existe pas.
pub fn is_ready(id: &str) -> bool {
    context_for(id);
    READY.with(|set| set.borrow().contains(id))
}

/// Un identifiant sert de nom de dossier : lettres, chiffres, tiret et soulignement seulement.
pub fn is_valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= MAX_ID_LEN
        && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Le contexte du conteneur `id` (cree et persiste dans `profile/conteneur-<id>` la premiere fois).
/// `None` pour un identifiant invalide : l'onglet retombe alors sur le contexte commun.
pub fn context_for(id: &str) -> Option<RequestContext> {
    if !is_valid_id(id) {
        warn!(%id, "identifiant de conteneur refuse");
        return None;
    }
    if let Some(known) = CONTEXTS.with(|map| map.borrow().get(id).cloned()) {
        return Some(known);
    }
    // Chromium n'accepte un profil que comme enfant direct de la racine du cache.
    let path = crate::flags::data_dir().join("profile").join(format!("conteneur-{id}"));
    let settings = RequestContextSettings {
        cache_path: path.to_string_lossy().as_ref().into(),
        persist_session_cookies: 1,
        ..Default::default()
    };
    let mut handler = ReadyHandler::new(id.to_string());
    let context = request_context_create_context(Some(&settings), Some(&mut handler))?;
    crate::assets::install_factory_in(&context);
    CONTEXTS.with(|map| map.borrow_mut().insert(id.to_string(), context.clone()));
    Some(context)
}

/// Chaque contexte deja cree, avec son identifiant.
pub fn for_each_context(mut f: impl FnMut(&str, &RequestContext)) {
    CONTEXTS.with(|map| map.borrow().iter().for_each(|(id, context)| f(id, context)));
}

/// Libere les contextes avant l'arret de Chromium.
pub fn release() {
    CONTEXTS.with(|map| map.borrow_mut().clear());
    READY.with(|set| set.borrow_mut().clear());
}

wrap_task! {
    struct LaterTask {
        job: std::sync::Arc<std::sync::Mutex<Option<Box<dyn FnOnce() + Send>>>>,
    }

    impl Task {
        fn execute(&self) {
            let job = self.job.lock().ok().and_then(|mut slot| slot.take());
            if let Some(job) = job {
                job();
            }
        }
    }
}

/// Relance `job` dans un instant : le temps que Chromium finisse d'initialiser un profil.
pub fn later<J: FnOnce() + Send + 'static>(job: J) {
    let mut task = LaterTask::new(std::sync::Arc::new(std::sync::Mutex::new(Some(Box::new(job)))));
    post_delayed_task(ThreadId::UI, Some(&mut task), RETRY_MS);
}

const RETRY_MS: i64 = 40;
