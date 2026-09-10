//! Echo Browser — coque du navigateur.
//!
//! Le meme executable sert de processus principal et de processus enfant : Chromium
//! se relance lui-meme avec `--type=`. Tout ce qui precede `execute_process` est donc
//! execute une fois par processus.

mod app;
mod assets;
mod bridge;
mod client;
mod filtering;
mod flags;
mod identity;
mod session;
mod tabs;
mod window;

use cef::{api_hash, args::Args, execute_process, initialize, run_message_loop, shutdown, ImplCommandLine, Settings};
use tracing::info;

fn main() -> anyhow::Result<()> {
    // CEF exige que la version de l'API soit fixee avant tout autre appel a la bibliotheque.
    let _ = api_hash(cef::sys::CEF_API_VERSION_LAST, 0);

    let args = Args::new();
    let Some(command_line) = args.as_cmd_line() else {
        anyhow::bail!("ligne de commande illisible");
    };
    let is_browser_process = command_line.has_switch(Some(&cef::CefString::from("type"))) != 1;

    // L'application est fournie a TOUS les processus : sans elle, les processus de rendu
    // ignorent le schema echo:// et l'interface se charge sans ses feuilles ni ses scripts.
    let mut app = app::EchoApp::new();
    let code = execute_process(Some(args.as_main_args()), Some(&mut app), std::ptr::null_mut());
    if !is_browser_process {
        anyhow::ensure!(code >= 0, "processus enfant non demarre (code {code})");
        return Ok(());
    }
    anyhow::ensure!(code == -1, "processus principal deja consomme (code {code})");

    init_logging();
    let settings = browser_settings();
    let started = initialize(Some(args.as_main_args()), Some(&settings), Some(&mut app), std::ptr::null_mut());
    anyhow::ensure!(started == 1, "Chromium n'a pas demarre");

    info!("boucle de messages lancee");
    run_message_loop();
    shutdown();
    Ok(())
}

fn browser_settings() -> Settings {
    let cef_dir = flags::cef_dir();
    let data_dir = flags::data_dir();
    Settings {
        no_sandbox: 1,
        locale: "fr".into(),
        // Chromium repose cet en-tete apres les gestionnaires de requete : il ne peut se
        // regler qu'ici. Une adresse francaise qui reclame ses pages en anglais est un
        // signal d'automate, et vaut un captcha.
        accept_language_list: identity::ACCEPT_LANGUAGE.into(),
        root_cache_path: data_dir.join("profile").to_string_lossy().as_ref().into(),
        resources_dir_path: cef_dir.to_string_lossy().as_ref().into(),
        locales_dir_path: cef_dir.join("locales").to_string_lossy().as_ref().into(),
        ..Default::default()
    }
}

fn init_logging() {
    let filter = std::env::var("ECHO_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt().with_env_filter(filter).with_target(false).init();
}
