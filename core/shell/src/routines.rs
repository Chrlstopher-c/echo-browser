//! Responsabilite : routines dans le navigateur — suit les sites ouverts a la suite (seance = visites a moins de 15 min
//! d'intervalle), propose une routine quand une suite revient, et l'ouvre d'un geste une fois acceptee.

use std::time::{Duration, Instant};

use echo_contract::{CoreEvent, RoutineProposalView, RoutineView};
use echo_library::routines;
use parking_lot::Mutex;

/// Au-dela de ce silence, une nouvelle seance commence.
const SESSION_GAP: Duration = Duration::from_secs(15 * 60);
const KEPT: usize = 6;

static RECENT: Mutex<Vec<(String, String, Instant)>> = Mutex::new(Vec::new());

/// Fenetre entre deux comptes d'une meme suite (`ECHO_ROUTINE_GAP_S` pour les essais).
fn gap_s() -> i64 {
    std::env::var("ECHO_ROUTINE_GAP_S").ok().and_then(|v| v.parse().ok()).unwrap_or(routines::DEFAULT_GAP_S)
}

/// Une page atteinte (fil de l'interface).
pub fn visited(url: &str) {
    let Some(site) = echo_network::site::site_of(url) else { return };
    let recent: Vec<(String, String)> = {
        let mut all = RECENT.lock();
        let now = Instant::now();
        if all.last().is_some_and(|(_, _, at)| now.duration_since(*at) > SESSION_GAP) {
            all.clear();
        }
        if all.last().is_some_and(|(last, _, _)| *last == site) {
            return;
        }
        all.push((site, url.to_string(), now));
        if all.len() > KEPT {
            all.remove(0);
        }
        all.iter().map(|(s, u, _)| (s.clone(), u.clone())).collect()
    };
    let proposal = crate::session::with(|s| routines::observe(&s.library, &recent, gap_s())).flatten();
    if let Some(p) = proposal {
        let proposal = RoutineProposalView { fingerprint: p.fingerprint, sites: p.sites, urls: p.urls };
        crate::bridge::publish(&CoreEvent::RoutineProposed { proposal });
    }
}

pub fn publish() {
    let list = crate::session::with(|s| routines::list(&s.library)).unwrap_or_default();
    let routines = list.into_iter().map(|r| RoutineView { id: r.id, name: r.name, urls: r.urls }).collect();
    crate::bridge::publish(&CoreEvent::RoutinesChanged { routines });
}

pub fn accept(fingerprint: &str, name: &str) {
    let name = name.trim();
    let name = if name.is_empty() { "Routine" } else { name };
    crate::session::with(|s| routines::adopt(&s.library, fingerprint, name));
    publish();
}

pub fn dismiss(fingerprint: &str) {
    crate::session::with(|s| routines::dismiss(&s.library, fingerprint));
}

/// Ouvre chaque page de la routine dans un nouvel onglet.
pub fn open(id: i64) {
    let urls = crate::session::with(|s| routines::list(&s.library).into_iter().find(|r| r.id == id).map(|r| r.urls))
        .flatten()
        .unwrap_or_default();
    for url in urls.iter().filter(|u| u.starts_with("http")) {
        crate::bridge::open_tab(url);
    }
}

pub fn remove(id: i64) {
    crate::session::with(|s| routines::remove(&s.library, id));
    publish();
}
