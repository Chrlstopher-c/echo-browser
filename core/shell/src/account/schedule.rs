//! Quand synchroniser. Trois modes (reglage `sync.mode`, propre a la machine) :
//! - `realtime` : un lot de modifications part 5 s apres la premiere, le service est controle chaque minute ;
//! - `auto` (par defaut) : lot apres 60 s, controle toutes les 5 min ;
//! - `manual` : rien d'automatique, l'interface previent quand des modifications attendent.
//!
//! Dans tous les modes automatiques, une synchro au lancement. Le controle (`/v1/etat`) est une lecture legere : la
//! synchro complete ne part que si le service a une version que cette machine n'a pas fusionnee.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use cef::*;
use echo_account::api::Client;
use echo_account::store;
use echo_library::settings::Value as Setting;
use tracing::debug;

pub const MODE_SETTING: &str = "sync.mode";
const FIRST_SYNC_MS: i64 = 1_500;
const RETRY_MS: i64 = 30_000;
const AGAIN_MS: i64 = 5_000;

/// Des modifications locales attendent d'etre envoyees.
static DIRTY: AtomicBool = AtomicBool::new(false);
/// Un lot est deja programme (les modifications suivantes le rejoignent).
static BATCH_ARMED: AtomicBool = AtomicBool::new(false);
/// Generation du controle periodique : changer de mode annule l'ancien.
static POLL_GEN: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Realtime,
    Auto,
    Manual,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Realtime => "realtime",
            Mode::Auto => "auto",
            Mode::Manual => "manual",
        }
    }

    fn batch_ms(self) -> Option<i64> {
        match self {
            Mode::Realtime => Some(5_000),
            Mode::Auto => Some(60_000),
            Mode::Manual => None,
        }
    }

    fn poll_ms(self) -> Option<i64> {
        match self {
            Mode::Realtime => Some(60_000),
            Mode::Auto => Some(5 * 60_000),
            Mode::Manual => None,
        }
    }
}

pub fn mode() -> Mode {
    let text = crate::session::with(|s| {
        echo_library::settings::all(&s.library).into_iter().find(|(k, _)| k == MODE_SETTING).map(|(_, v)| v)
    })
    .flatten();
    match text {
        Some(Setting::Text(t)) if t == "realtime" => Mode::Realtime,
        Some(Setting::Text(t)) if t == "manual" => Mode::Manual,
        _ => Mode::Auto,
    }
}

pub fn pending() -> bool {
    DIRTY.load(Ordering::SeqCst)
}

pub fn start() {
    if mode() != Mode::Manual {
        later(FIRST_SYNC_MS, Job::Sync);
    }
    arm_poll();
}

/// Une donnee synchronisee vient de changer ici : elle part avec le prochain lot.
pub fn touch() {
    touch_after(mode().batch_ms());
}

/// Une visite ou un onglet ouvert/ferme : ca change sans cesse, ca part par lots plus espaces (2 min en temps reel,
/// 10 min sinon) pour tenir dans l'offre gratuite du service. Un lot deja programme plus tot les emmene aussi.
pub fn touch_soft() {
    touch_after(match mode() {
        Mode::Realtime => Some(2 * 60_000),
        Mode::Auto => Some(10 * 60_000),
        Mode::Manual => None,
    });
}

fn touch_after(batch_ms: Option<i64>) {
    if !signed_in() {
        return;
    }
    let was_dirty = DIRTY.swap(true, Ordering::SeqCst);
    if !was_dirty {
        super::publish();
    }
    if let Some(delay) = batch_ms
        && !BATCH_ARMED.swap(true, Ordering::SeqCst)
    {
        later(delay, Job::Batch);
    }
}

/// La synchro a lu les donnees locales : ce qui change ensuite fera partie du lot suivant.
pub fn snapshot_taken() {
    DIRTY.store(false, Ordering::SeqCst);
}

pub fn sync_done(again: bool) {
    if again && mode() != Mode::Manual {
        later(AGAIN_MS, Job::Sync);
    }
}

pub fn sync_failed() {
    DIRTY.store(true, Ordering::SeqCst);
    if mode() != Mode::Manual {
        later(RETRY_MS, Job::Sync);
    }
}

pub fn mode_changed() {
    arm_poll();
    if DIRTY.load(Ordering::SeqCst) && mode() != Mode::Manual {
        later(AGAIN_MS, Job::Sync);
    }
}

fn signed_in() -> bool {
    store::load(&super::stored_path()).is_some()
}

fn arm_poll() {
    let generation = POLL_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    if let Some(delay) = mode().poll_ms() {
        later(delay, Job::Poll(generation));
    }
}

#[derive(Clone, Copy)]
enum Job {
    Sync,
    Batch,
    Poll(u64),
}

fn later(delay_ms: i64, job: Job) {
    let (kind, generation) = match job {
        Job::Sync => (0, 0),
        Job::Batch => (1, 0),
        Job::Poll(g) => (2, g),
    };
    let mut task = ScheduleTask::new(kind, generation);
    post_delayed_task(ThreadId::UI, Some(&mut task), delay_ms);
}

fn run(job: Job) {
    match job {
        Job::Sync => {
            if !super::sync_now() && signed_in() {
                later(AGAIN_MS, Job::Sync);
            }
        }
        Job::Batch => {
            BATCH_ARMED.store(false, Ordering::SeqCst);
            if DIRTY.load(Ordering::SeqCst) && mode() != Mode::Manual && !super::sync_now() {
                later(AGAIN_MS, Job::Batch);
                BATCH_ARMED.store(true, Ordering::SeqCst);
            }
        }
        Job::Poll(generation) if generation == POLL_GEN.load(Ordering::SeqCst) => {
            check_service();
            if let Some(delay) = mode().poll_ms() {
                later(delay, Job::Poll(generation));
            }
        }
        Job::Poll(_) => {}
    }
}

/// Controle leger hors du fil de l'interface ; synchro complete seulement si le service a du nouveau.
fn check_service() {
    let (Some(url), Some(stored)) = (super::service_url(), store::load(&super::stored_path())) else { return };
    let history = super::history::enabled();
    std::thread::spawn(move || {
        let result = Client::new(&url).state(&stored.token);
        crate::containers::later(move || match result {
            Ok((mut versions, admin)) => {
                versions.retain(|kind, _| kind != "historique" || history);
                if admin != stored.admin {
                    let mut updated = stored.clone();
                    updated.admin = admin;
                    let _ = store::save(&super::stored_path(), &updated);
                    super::publish();
                }
                if stored.behind(&versions) || DIRTY.load(Ordering::SeqCst) {
                    debug!("service en avance ou modifications locales : synchro");
                    super::sync_now();
                }
            }
            Err(err) => {
                *super::LAST_ERROR.lock() = Some(err.to_string());
                super::publish();
            }
        });
    });
}

wrap_task! {
    struct ScheduleTask {
        kind: u8,
        generation: u64,
    }

    impl Task {
        fn execute(&self) {
            run(match self.kind {
                0 => Job::Sync,
                1 => Job::Batch,
                _ => Job::Poll(self.generation),
            });
        }
    }
}
