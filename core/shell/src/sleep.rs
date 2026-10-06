//! Responsabilite : endormir periodiquement les onglets inactifs pour rendre leur memoire.

use cef::*;
use std::time::Duration;
use tracing::info;

/// Delai d'inactivite avant la mise en veille, sauf `ECHO_SLEEP_AFTER_S`.
const DEFAULT_IDLE_SECS: u64 = 300;
const TICK_MS: i64 = 15_000;

/// Message console que la page emet a la premiere saisie de l'utilisateur.
pub const DIRTY_MARKER: &str = "echo:dirty";

/// Pose une ecoute des saisies reelles (`isTrusted`) ; la premiere previent le coeur par la console.
pub const DIRTY_WATCHER: &str = "(()=>{let sent=false;const f=e=>{if(sent||!e.isTrusted)return;sent=true;\
console.debug('echo:dirty')};addEventListener('input',f,true);addEventListener('change',f,true)})()";

fn idle_delay() -> Duration {
    let secs = std::env::var("ECHO_SLEEP_AFTER_S")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_IDLE_SECS);
    Duration::from_secs(secs)
}

/// Arme la verification periodique.
pub fn start() {
    info!(apres_s = idle_delay().as_secs(), "veille des onglets armee");
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
            let slept = crate::bridge::sleep_idle_tabs(idle_delay());
            if slept > 0 {
                info!(onglets = slept, "onglets inactifs endormis");
            }
            schedule_tick();
        }
    }
}
