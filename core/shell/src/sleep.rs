//! Responsabilite : endormir periodiquement les onglets inactifs pour rendre leur memoire.

use cef::*;
use std::time::Duration;
use tracing::info;

const TICK_MS: i64 = 15_000;

/// Message console que la page emet a la premiere saisie de l'utilisateur.
pub const DIRTY_MARKER: &str = "echo:dirty";

/// Pose une ecoute des saisies reelles (`isTrusted`) ; la premiere previent le coeur par la console.
pub const DIRTY_WATCHER: &str = "(()=>{let sent=false;const f=e=>{if(sent||!e.isTrusted)return;sent=true;\
console.debug('echo:dirty')};addEventListener('input',f,true);addEventListener('change',f,true);\
let t=0;addEventListener('scroll',()=>{clearTimeout(t);t=setTimeout(()=>{\
const y=Math.round(document.scrollingElement?document.scrollingElement.scrollTop:0);\
console.debug('echo:scroll:'+y)},400)},{passive:true,capture:true})})()";

/// Prefixe du message console qui porte le defilement de la page.
pub const SCROLL_MARKER: &str = "echo:scroll:";

/// Rejoue le defilement d'avant la mise en veille ; renonce si l'utilisateur a deja defile.
pub fn restore_script(y: i32) -> String {
    format!(
        "(()=>{{const go=()=>{{if(scrollY<5)scrollTo(0,{y})}};go();setTimeout(go,300);setTimeout(go,1200)}})()"
    )
}

/// Delai d'inactivite avant la mise en veille, ou `None` si la veille est coupee.
/// `ECHO_SLEEP_AFTER_S` (secondes) l'emporte sur les reglages : c'est le levier des bancs de mesure.
fn idle_delay() -> Option<Duration> {
    if let Some(secs) = std::env::var("ECHO_SLEEP_AFTER_S").ok().and_then(|v| v.parse().ok()) {
        return Some(Duration::from_secs(secs));
    }
    use echo_library::settings::Value;
    let settings = crate::session::with(|s| echo_library::settings::all(&s.library))?;
    let find = |key: &str| settings.iter().find(|(k, _)| k == key).map(|(_, v)| v);
    let enabled = !matches!(find("tabs.sleepEnabled"), Some(Value::Flag(false)));
    let Some(Value::Number(minutes)) = find("tabs.sleepAfterMinutes") else { return None };
    enabled.then(|| Duration::from_secs((minutes.max(1.0) * 60.0) as u64))
}

/// Au-dela de ce nombre de pages en memoire, la veille se hate.
const PRESSURE_TABS: usize = 4;
const PRESSURE_IDLE: Duration = Duration::from_secs(60);

/// Beaucoup d'onglets ouverts : on n'attend plus le delai complet pour rendre la memoire.
fn under_pressure(idle: Duration) -> Duration {
    let live = crate::session::with(|s| s.tabs.live_count()).unwrap_or(0);
    if live > PRESSURE_TABS { idle.min(PRESSURE_IDLE) } else { idle }
}

/// Delai avant de purger la memoire JavaScript d'une page d'arriere-plan (`ECHO_TRIM_AFTER_S`, 0 = jamais).
fn trim_delay() -> Option<Duration> {
    let secs = std::env::var("ECHO_TRIM_AFTER_S").ok().and_then(|v| v.parse().ok()).unwrap_or(30);
    (secs > 0).then(|| Duration::from_secs(secs))
}

/// Arme la verification periodique.
pub fn start() {
    info!("veille des onglets armee");
    schedule_tick();
}

fn schedule_tick() {
    let mut task = SleepTask::new(0);
    post_delayed_task(ThreadId::UI, Some(&mut task), TICK_MS);
}

wrap_task! {
    struct SleepTask {
        unused: i32,
    }

    impl Task {
        fn execute(&self) {
            if let Some(idle) = trim_delay() {
                crate::bridge::trim_idle_tabs(idle);
            }
            let slept = idle_delay().map(under_pressure).map_or(0, crate::bridge::sleep_idle_tabs);
            if slept > 0 {
                info!(onglets = slept, "onglets inactifs endormis");
            }
            schedule_tick();
        }
    }
}
