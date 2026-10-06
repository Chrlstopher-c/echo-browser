//! Responsabilite : sauvegarder les onglets au fil de l'eau, pour les retrouver apres une fermeture
//! ou un arret brutal (derniere session).

use cef::*;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::debug;

/// Delai de regroupement : une rafale de changements n'ecrit qu'une fois.
const DEBOUNCE_MS: i64 = 1_200;

static PENDING: AtomicBool = AtomicBool::new(false);

/// Programme une sauvegarde, sauf s'il y en a deja une en route. A appeler a chaque changement d'onglets.
pub fn schedule() {
    if PENDING.swap(true, Ordering::Relaxed) {
        return;
    }
    let mut task = SaveTask::new(());
    post_delayed_task(ThreadId::UI, Some(&mut task), DEBOUNCE_MS);
}

wrap_task! {
    struct SaveTask {
        marker: (),
    }

    impl Task {
        fn execute(&self) {
            PENDING.store(false, Ordering::Relaxed);
            if let Some(snapshot) = crate::session::with(|s| s.tabs.to_snapshot()) {
                debug!(onglets = snapshot.tabs.len(), "session enregistree");
                crate::restart::save_last(&crate::flags::data_dir(), &snapshot);
            }
        }
    }
}
