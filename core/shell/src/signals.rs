//! Responsabilite : signaux anonymes (reglage `signals.share`, coupe par defaut). Seulement s'il est active : les
//! domaines visites et les hotes bloques par le bouclier sont comptes par jour, en memoire puis dans la bibliotheque ;
//! chaque jour fini part en un lot sans compte, avec un jeton aleatoire neuf (aucun identifiant stable), puis est
//! efface.
//! Desactive : rien n'est compte, et ce qui attendait est efface.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

use cef::*;
use echo_library::settings::Value as Setting;
use echo_library::signals;
use parking_lot::Mutex;
use serde_json::{json, Map, Value};
use tracing::{info, warn};

pub const SETTING: &str = "signals.share";
const TICK_MS: i64 = 10 * 60 * 1000;
const FIRST_MS: i64 = 60 * 1000;
const TOP: usize = 200;
/// Un jour plus vieux n'est plus envoye (le service le refuserait).
const MAX_AGE_DAYS: i64 = 7;

static ENABLED: AtomicBool = AtomicBool::new(false);
static PENDING: Mutex<Option<HashMap<(&'static str, String), u32>>> = Mutex::new(None);

fn unix_day() -> i64 {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
    now.map(|d| d.as_secs() as i64 / 86_400).unwrap_or(0)
}

/// `AAAA-MM-JJ` d'un jour Unix (calendrier civil, UTC).
fn date_of(day: i64) -> String {
    let z = day + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Jour sous lequel les comptes sont ranges (`ECHO_SIGNALS_SHIFT_DAYS` le recule, pour les essais).
fn record_day() -> String {
    let shift = std::env::var("ECHO_SIGNALS_SHIFT_DAYS").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    date_of(unix_day() - shift)
}

fn read_enabled() -> bool {
    crate::session::with(|s| {
        echo_library::settings::all(&s.library).into_iter().any(|(k, v)| k == SETTING && v == Setting::Flag(true))
    })
    .unwrap_or(false)
}

fn count(kind: &'static str, key: String) {
    if !ENABLED.load(Ordering::Relaxed) || key.is_empty() || key.len() > 253 {
        return;
    }
    *PENDING.lock().get_or_insert_with(HashMap::new).entry((kind, key)).or_default() += 1;
}

/// Une page visitee : son domaine seulement.
pub fn note_visit(url: &str) {
    if let Some(site) = echo_network::site::site_of(url) {
        count("sites", site);
    }
}

/// Une requete bloquee par le bouclier : son hote (fils de Chromium).
pub fn note_blocked(url: &str) {
    if let Some(host) = echo_network::site::host_of(url) {
        count("bloques", host.to_ascii_lowercase());
    }
}

/// Le reglage a change : coupe, tout ce qui attendait est efface.
pub fn setting_changed(key: &str) {
    if key != SETTING {
        return;
    }
    let on = read_enabled();
    ENABLED.store(on, Ordering::SeqCst);
    if on {
        schedule(first_delay());
    } else {
        *PENDING.lock() = None;
        crate::session::with(|s| signals::clear(&s.library));
    }
    info!(partage = on, "signaux anonymes");
}

fn flush() {
    let Some(pending) = PENDING.lock().take() else { return };
    let rows: Vec<(String, String, u32)> = pending.into_iter().map(|((k, key), n)| (k.to_string(), key, n)).collect();
    let day = record_day();
    crate::session::with(|s| signals::add(&s.library, &day, &rows));
}

fn lot(day: &str) -> Option<Value> {
    let token: String = echo_account::crypto::random_bytes::<16>().ok()?.iter().map(|b| format!("{b:02x}")).collect();
    let section = |kind: &str| -> Value {
        let top = crate::session::with(|s| signals::top(&s.library, day, kind, TOP)).unwrap_or_default();
        Value::Object(top.into_iter().map(|(k, n)| (k, json!(n))).collect::<Map<String, Value>>())
    };
    Some(json!({
        "jour": day, "jeton": token, "version": env!("CARGO_PKG_VERSION"),
        "sites": section("sites"), "bloques": section("bloques"),
    }))
}

/// Envoie chaque jour fini en attente ; efface ceux qui sont partis (ou trop vieux).
fn send() {
    let Some(url) = crate::account::service_url() else { return };
    let today = date_of(unix_day());
    let oldest = date_of(unix_day() - MAX_AGE_DAYS);
    for day in crate::session::with(|s| signals::days(&s.library)).unwrap_or_default() {
        if day.as_str() < oldest.as_str() {
            crate::session::with(|s| signals::forget_day(&s.library, &day));
            continue;
        }
        if day >= today {
            continue;
        }
        let Some(body) = lot(&day) else { continue };
        let url = format!("{}/v1/signaux", url.trim_end_matches('/'));
        std::thread::spawn(move || match ureq::post(&url).send_json(&body) {
            Ok(_) => crate::containers::later(move || {
                crate::session::with(|s| signals::forget_day(&s.library, &day));
                info!(%day, "signaux anonymes envoyes");
            }),
            Err(err) => warn!(%err, %day, "signaux anonymes non envoyes"),
        });
    }
}

/// Premier envoi apres le lancement ou l'activation (`ECHO_SIGNALS_SEND_S` pour les essais).
fn first_delay() -> i64 {
    std::env::var("ECHO_SIGNALS_SEND_S").ok().and_then(|v| v.parse::<i64>().ok()).map_or(FIRST_MS, |s| s * 1000)
}

pub fn start() {
    ENABLED.store(read_enabled(), Ordering::SeqCst);
    schedule(first_delay());
}

fn schedule(delay_ms: i64) {
    let mut task = SignalsTask::new(0);
    post_delayed_task(ThreadId::UI, Some(&mut task), delay_ms);
}

wrap_task! {
    struct SignalsTask {
        unused: i32,
    }

    impl Task {
        fn execute(&self) {
            if ENABLED.load(Ordering::Relaxed) {
                flush();
                send();
            }
            schedule(TICK_MS);
        }
    }
}
