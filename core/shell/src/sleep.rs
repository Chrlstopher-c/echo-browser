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
console.debug('echo:scroll:'+y)},400)},{passive:true,capture:true});\
addEventListener('mousedown',e=>{if(e.button!==2)console.debug('echo:press')},true);\
let last='';const m=()=>{const all=[...document.querySelectorAll('video,audio')];\
const on=all.filter(x=>!x.paused&&!x.ended&&x.readyState>2);\
const s=(on.length?'1':'0')+(on.some(x=>!x.muted&&x.volume>0)?'1':'0');\
if(s!==last){last=s;console.debug('echo:media:'+s)}};\
for(const ev of ['play','playing','pause','ended','volumechange','emptied'])addEventListener(ev,m,true);\
setInterval(m,5000)})()";

/// Message console de lecture : `echo:media:<lecture><son>` (1/0 chacun).
pub const MEDIA_MARKER: &str = "echo:media:";

/// Message console d'un clic gauche ou milieu dans la page : il referme le menu contextuel ouvert.
pub const PRESS_MARKER: &str = "echo:press";

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

/// Quand la memoire de l'ordinateur vient a manquer, la veille n'attend plus le delai regle.
const PRESSURE_IDLE: Duration = Duration::from_secs(60);
/// Seuils de manque : moins de 12 % de la memoire disponible, ou moins de 1,5 Go.
const PRESSURE_RATIO: f64 = 0.12;
const PRESSURE_FLOOR_KB: u64 = 1_536 * 1024;

/// Memoire (totale, disponible) en Ko, lue dans /proc/meminfo.
fn meminfo() -> Option<(u64, u64)> {
    let text = std::fs::read_to_string("/proc/meminfo").ok()?;
    let field = |name: &str| {
        text.lines().find(|l| l.starts_with(name))?.split_whitespace().nth(1)?.parse::<u64>().ok()
    };
    Some((field("MemTotal:")?, field("MemAvailable:")?))
}

/// La memoire vient-elle a manquer ? `ECHO_PRESSURE=1` le simule (bancs).
fn memory_short() -> bool {
    if std::env::var("ECHO_PRESSURE").is_ok_and(|v| v == "1") {
        return true;
    }
    meminfo().is_some_and(|(total, available)| {
        available < PRESSURE_FLOOR_KB || (available as f64) < total as f64 * PRESSURE_RATIO
    })
}

/// Le delai regle, sauf si la memoire manque : alors on rend la memoire des onglets inactifs plus tot. Le nombre
/// d'onglets ouverts n'entre plus en compte (audit du 08/10 : 60 s des 5 onglets, contre 5 min affichees).
fn under_pressure(idle: Duration) -> Duration {
    if memory_short() { idle.min(PRESSURE_IDLE) } else { idle }
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
            for id in crate::session::with(|s| s.tabs.kept_awake_asleep()).unwrap_or_default() {
                crate::bridge::warm_tab(id);
            }
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
