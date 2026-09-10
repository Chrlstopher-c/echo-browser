//! Responsabilite : compter ce que le bouclier a bloque, globalement et par onglet.

use parking_lot::Mutex;
use serde::Serialize;
use std::collections::HashMap;

/// Compteurs de blocage. Un onglet est identifie par le meme entier que dans le domaine `tabs`.
#[derive(Debug, Default)]
pub struct Tally {
    per_tab: Mutex<HashMap<u32, u64>>,
    total: Mutex<u64>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct TallySnapshot {
    pub tab: u64,
    pub total: u64,
}

impl Tally {
    pub fn record_block(&self, tab_id: u32) {
        *self.per_tab.lock().entry(tab_id).or_insert(0) += 1;
        *self.total.lock() += 1;
    }

    pub fn snapshot(&self, tab_id: u32) -> TallySnapshot {
        TallySnapshot {
            tab: self.per_tab.lock().get(&tab_id).copied().unwrap_or(0),
            total: *self.total.lock(),
        }
    }

    /// A appeler quand un onglet navigue ailleurs ou se ferme.
    pub fn reset_tab(&self, tab_id: u32) {
        self.per_tab.lock().remove(&tab_id);
    }
}
