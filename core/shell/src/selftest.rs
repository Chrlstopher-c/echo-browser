//! Responsabilite : rejouer sans main les manipulations d'onglets, pour verifier
//! qu'elles ne figent pas le navigateur.
//!
//! Active par `ECHO_SELFTEST=1`. Passe par les memes fonctions que les clics de
//! l'interface : ce qui casse ici casse aussi a la main.

// Les macros `wrap_*` de CEF exigent les traits `Impl*` et `Wrap*` dans la portee : import global impose.
use cef::*;
use tracing::info;

/// Programme la sequence si la variable d'environnement le demande.
pub fn schedule() {
    schedule_bench();
    schedule_wake_check();
    if std::env::var_os("ECHO_SELFTEST").is_none() {
        return;
    }
    info!("autotest arme : ouverture puis fermeture d'onglet");
    plan(6_000, Step::Open);
    plan(11_000, Step::Close);
    plan(13_000, Step::Browse);
    plan(15_000, Step::Report);
    if std::env::var_os("ECHO_SELFTEST_RESTART").is_some() {
        plan(18_000, Step::Restart);
    }
}

/// Banc memoire : `ECHO_BENCH_URLS` (adresses separees par des espaces) ouvre un onglet toutes les 5 s.
fn schedule_bench() {
    let Some(urls) = std::env::var("ECHO_BENCH_URLS").ok() else {
        return;
    };
    for (i, url) in urls.split_whitespace().enumerate() {
        info!(url, "banc : onglet programme");
        let mut task = BenchTask::new(url.to_string());
        post_delayed_task(ThreadId::UI, Some(&mut task), 6_000 + 5_000 * i as i64);
    }
}

/// `ECHO_BENCH_WAKE_AT_S` : reveille a cette date le premier onglet endormi, puis rapporte l'etat 10 s apres.
fn schedule_wake_check() {
    let Some(at) = std::env::var("ECHO_BENCH_WAKE_AT_S").ok().and_then(|v| v.parse::<i64>().ok()) else {
        return;
    };
    for (phase, delay) in [(0, at * 1000), (1, at * 1000 + 10_000)] {
        let mut task = BenchWakeTask::new(phase);
        post_delayed_task(ThreadId::UI, Some(&mut task), delay);
    }
}

wrap_task! {
    struct BenchWakeTask {
        phase: i32,
    }

    impl Task {
        fn execute(&self) {
            if self.phase == 0 {
                let target = crate::session::with(|s| s.tabs.snapshot().iter().find(|t| t.asleep).map(|t| t.id)).flatten();
                info!(?target, "banc : reveil de l'onglet endormi");
                if let Some(id) = target {
                    crate::bridge::select_tab(id);
                }
            } else {
                let tabs = crate::session::with(|s| s.tabs.snapshot()).unwrap_or_default();
                for t in &tabs {
                    info!(id = t.id, asleep = t.asleep, loading = t.loading, title = %t.title, "banc : etat final");
                }
            }
        }
    }
}

wrap_task! {
    struct BenchTask {
        url: String,
    }

    impl Task {
        fn execute(&self) {
            crate::bridge::open_tab(&self.url);
            crate::bridge::publish_tabs();
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Step {
    Open,
    Close,
    Report,
    Restart,
    Browse = 4,
}

fn plan(delay_ms: i64, step: Step) {
    let mut task = SelfTestTask::new(step as i32);
    post_delayed_task(ThreadId::UI, Some(&mut task), delay_ms);
}

wrap_task! {
    struct SelfTestTask {
        step: i32,
    }

    impl Task {
        fn execute(&self) {
            match self.step {
                0 => {
                    info!("autotest : ouverture d'un onglet");
                    crate::bridge::open_tab("https://www.qwant.com/");
                    crate::bridge::publish_tabs();
                }
                1 => {
                    let victim = crate::session::with(|s| s.tabs.active_id()).flatten();
                    info!(?victim, "autotest : fermeture de l'onglet actif");
                    if let Some(id) = victim {
                        crate::bridge::close_tab(id);
                    }
                }
                4 => {
                    info!("autotest : navigation pour remplir le fil");
                    crate::bridge::submit(br#"{"kind":"navigate","id":0,"input":"example.com"}"#);
                }
                2 => {
                    let count = crate::session::with(|s| s.tabs.snapshot().len()).unwrap_or(0);
                    info!(onglets_restants = count, "autotest termine, navigateur toujours vivant");
                }
                _ => {
                    info!("autotest : relance du navigateur");
                    crate::bridge::restart_browser();
                }
            }
        }
    }
}
