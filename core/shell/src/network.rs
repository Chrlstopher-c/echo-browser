//! Responsabilite : le reseau des onglets dans le navigateur — chaque requete est notee dans le journal de son onglet
//! (fils de Chromium), les regles de l'utilisateur (domaines bloques par site, isolement strict) sont appliquees, et le
//! panneau Reseau recoit l'onglet actif deux fois par seconde tant qu'il est ouvert (rien n'est diffuse sinon).

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::OnceLock;

use cef::*;
use echo_contract::{CoreEvent, JournalEntryView, NetDomainView, NetRequestView, NetWeightView, NetworkView};
use echo_network::log::Outgoing;
use echo_network::{Rules, TabLog};
use parking_lot::{Mutex, RwLock};
use tracing::warn;

const PUBLISH_MS: i64 = 500;
const DETAIL: usize = 150;
const JOURNAL: usize = 100;
/// Onglets suivis au plus (les navigateurs fermes sortent par la gauche).
const MAX_TABS: usize = 200;

static LOGS: OnceLock<Mutex<HashMap<i32, TabLog>>> = OnceLock::new();
static RULES: OnceLock<RwLock<Rules>> = OnceLock::new();
static WATCHING: AtomicBool = AtomicBool::new(false);
static DIRTY: AtomicBool = AtomicBool::new(false);
static SHOWN: AtomicI32 = AtomicI32::new(-1);
static FOCUS: Mutex<Option<String>> = Mutex::new(None);
/// (site de la page, hote tiers) deja notes au journal : evite de solliciter la base a chaque requete.
static SEEN_THIRD: OnceLock<Mutex<std::collections::HashSet<(String, String)>>> = OnceLock::new();

fn logs() -> &'static Mutex<HashMap<i32, TabLog>> {
    LOGS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn rules_path() -> std::path::PathBuf {
    crate::flags::data_dir().join("reseau.json")
}

fn rules() -> &'static RwLock<Rules> {
    RULES.get_or_init(|| RwLock::new(Rules::load(&rules_path())))
}

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

/// La regle de l'utilisateur qui bloque `url` sur la page `page`, s'il y en a une.
pub fn rule_verdict(url: &str, page: &str) -> Option<&'static str> {
    rules().read().verdict(url, page)
}

/// Une requete part (ou est bloquee) : notee dans le journal de son onglet.
pub struct Started<'a> {
    pub browser: i32,
    pub id: u64,
    pub url: &'a str,
    pub page: &'a str,
    pub kind: &'a str,
    pub method: &'a str,
    pub blocked: Option<&'a str>,
    pub main_frame: bool,
}

pub fn started(s: Started<'_>) {
    let mut all = logs().lock();
    if all.len() >= MAX_TABS
        && !all.contains_key(&s.browser)
        && let Some(oldest) = all.keys().min().copied()
    {
        all.remove(&oldest);
    }
    let log = all.entry(s.browser).or_default();
    if s.main_frame {
        log.navigated(s.url);
    }
    let page = if s.main_frame { s.url } else { s.page };
    let now_ms = now_ms();
    log.start(Outgoing { id: s.id, url: s.url, page, kind: s.kind, method: s.method, blocked: s.blocked, now_ms });
    DIRTY.store(true, Ordering::Relaxed);
    if s.blocked.is_none() && echo_network::site::is_third_party(s.url, page) {
        note_third_party(page, s.url);
    }
}

fn note_third_party(page: &str, url: &str) {
    let (Some(site), Some(host)) = (echo_network::site::site_of(page), echo_network::site::host_of(url)) else { return };
    let key = (site, host.to_ascii_lowercase());
    let seen = SEEN_THIRD.get_or_init(|| Mutex::new(std::collections::HashSet::new()));
    if seen.lock().insert(key.clone()) {
        crate::containers::later(move || {
            crate::session::with(|s| echo_library::journal::record(&s.library, &key.0, "tiers", &key.1, true));
        });
    }
}

/// Note un acces sensible au journal du site de `url` (fil de l'interface).
pub fn journal(url: &str, kind: &str, detail: &str) {
    let Some(site) = echo_network::site::site_of(url) else { return };
    crate::session::with(|s| echo_library::journal::record(&s.library, &site, kind, detail, false));
    DIRTY.store(true, Ordering::Relaxed);
}

pub fn completed(browser: i32, id: u64, status: u16, bytes: u64) {
    if let Some(log) = logs().lock().get_mut(&browser) {
        log.complete(id, status, bytes, now_ms());
        DIRTY.store(true, Ordering::Relaxed);
    }
}

/// Le panneau s'ouvre ou se ferme.
pub fn watch(on: bool) {
    let was = WATCHING.swap(on, Ordering::SeqCst);
    if on && !was {
        SHOWN.store(-1, Ordering::SeqCst);
        tick();
    }
}

pub fn focus(host: Option<String>) {
    *FOCUS.lock() = host;
    SHOWN.store(-1, Ordering::SeqCst);
}

fn active_page() -> Option<String> {
    crate::session::with(|s| s.tabs.active().map(|t| t.url.clone())).flatten()
}

pub fn block_host(host: &str, blocked: bool) {
    let Some(page) = active_page() else { return };
    edit_rules(|r| r.set_blocked(&page, host, blocked));
}

pub fn set_strict(strict: bool) {
    let Some(page) = active_page() else { return };
    edit_rules(|r| r.set_strict(&page, strict));
}

fn edit_rules(change: impl FnOnce(&mut Rules)) {
    let mut rules = rules().write();
    change(&mut rules);
    if let Err(err) = rules.save(&rules_path()) {
        warn!(%err, "regles reseau non enregistrees");
    }
    SHOWN.store(-1, Ordering::SeqCst);
}

fn active_browser() -> Option<i32> {
    crate::session::with(|s| s.tabs.active().and_then(|t| t.browser()).map(|b| b.identifier())).flatten()
}

/// Diffuse l'onglet actif s'il a change (ou si le journal a bouge), puis se reprogramme tant que le panneau est ouvert.
fn tick() {
    if !WATCHING.load(Ordering::SeqCst) {
        return;
    }
    let browser = active_browser().unwrap_or(0);
    if DIRTY.swap(false, Ordering::Relaxed) || SHOWN.swap(browser, Ordering::SeqCst) != browser {
        publish(browser);
    }
    let mut task = NetworkTask::new(0);
    post_delayed_task(ThreadId::UI, Some(&mut task), PUBLISH_MS);
}

fn publish(browser: i32) {
    let focus = FOCUS.lock().clone();
    let page = active_page().unwrap_or_default();
    let (domains, requests, total_bytes, weight) = logs()
        .lock()
        .get(&browser)
        .map(|log| (log.summary(), log.recent(focus.as_deref(), DETAIL), log.total_bytes(), weight_view(log)))
        .unwrap_or_default();
    let journal = echo_network::site::site_of(&page)
        .and_then(|site| crate::session::with(|s| echo_library::journal::list(&s.library, &site, JOURNAL)))
        .unwrap_or_default()
        .into_iter()
        .map(|e| JournalEntryView { at: e.at, kind: e.kind, detail: e.detail })
        .collect();
    let rules = rules().read();
    let network = NetworkView {
        journal,
        blocked_hosts: rules.blocked_on(&page),
        strict: rules.is_strict(&page),
        page,
        total_bytes,
        domains: domains.into_iter().map(domain_view).collect(),
        requests: requests.into_iter().map(request_view).collect(),
        focus,
        weight,
    };
    crate::bridge::publish(&CoreEvent::NetworkChanged { network });
}

const RANKED: usize = 8;

fn weight_view(log: &TabLog) -> NetWeightView {
    let (by_kind, third_party_bytes) = log.weight();
    let mut by_kind: Vec<(String, u64)> = by_kind.into_iter().filter(|(_, b)| *b > 0).collect();
    by_kind.sort_by_key(|k| std::cmp::Reverse(k.1));
    NetWeightView {
        by_kind,
        third_party_bytes,
        heaviest: log.heaviest(RANKED).into_iter().map(request_view).collect(),
        slowest: log.slowest(RANKED).into_iter().map(request_view).collect(),
    }
}

fn domain_view(d: echo_network::DomainStat) -> NetDomainView {
    let mut kinds: Vec<(String, u32)> = d.kinds.into_iter().collect();
    kinds.sort_by_key(|k| std::cmp::Reverse(k.1));
    NetDomainView { host: d.host, site: d.site, requests: d.requests, bytes: d.bytes, blocked: d.blocked,
        third_party: d.third_party, kinds }
}

fn request_view(r: echo_network::RequestEntry) -> NetRequestView {
    NetRequestView { url: r.url, host: r.host, kind: r.kind, method: r.method, third_party: r.third_party,
        blocked: r.blocked, status: r.status, bytes: r.bytes, duration_ms: r.duration_ms }
}

wrap_task! {
    struct NetworkTask {
        unused: i32,
    }

    impl Task {
        fn execute(&self) {
            tick();
        }
    }
}
