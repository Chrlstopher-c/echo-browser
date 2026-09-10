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
    if std::env::var_os("ECHO_SELFTEST").is_none() {
        return;
    }
    info!("autotest arme : ouverture puis fermeture d'onglet");
    plan(6_000, Step::Open);
    plan(11_000, Step::Close);
    plan(15_000, Step::Report);
    if std::env::var_os("ECHO_SELFTEST_RESTART").is_some() {
        plan(18_000, Step::Restart);
    }
}

#[derive(Clone, Copy, Debug)]
enum Step {
    Open,
    Close,
    Report,
    Restart,
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
