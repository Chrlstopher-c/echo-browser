//! Responsabilite : le point de contact avec Chromium — drapeaux au demarrage, creation de la fenetre.

use crate::{assets, client::EchoClient, flags, window};
// Les macros `wrap_*` de CEF exigent les traits `Impl*` et `Wrap*` dans la portee : import global impose.
use cef::*;
use std::cell::RefCell;
use tracing::info;

/// L'interface du navigateur, servie par le schema interne.
const STARTUP_URL: &str = "echo://ui/index.html";


wrap_app! {
    pub struct EchoApp;

    impl App {
        fn on_before_command_line_processing(
            &self,
            process_type: Option<&CefString>,
            command_line: Option<&mut CommandLine>,
        ) {
            let Some(command_line) = command_line else { return };
            let process_type = process_type.map(CefString::to_string).unwrap_or_default();
            flags::apply(&process_type, command_line);
        }

        fn on_register_custom_schemes(&self, registrar: Option<&mut SchemeRegistrar>) {
            assets::register(registrar);
        }

        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(EchoBrowserProcessHandler::new(RefCell::new(None)))
        }
    }
}

wrap_browser_process_handler! {
    struct EchoBrowserProcessHandler {
        client: RefCell<Option<Client>>,
    }

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            assets::install_factory();
            let url = startup_url();
            info!(%url, "contexte Chromium pret, ouverture de la fenetre");
            let shield = load_shield();
            *self.client.borrow_mut() = Some(EchoClient::new(Default::default(), shield.clone()));
            let mut client = self.client.borrow().clone();
            let library = open_library();
            if let Some((_, echo_library::settings::Value::Text(shell))) =
                echo_library::settings::all(&library).into_iter().find(|(key, _)| key == "appearance.shell")
            {
                window::restore_accent(&shell);
            }
            let chrome_view = window::create_chrome_view(client.as_mut(), &url);
            let mut tabs = crate::tabs::Tabs::default();
            let host = tabs.host();
            crate::session::install(crate::session::Session {
                chrome: chrome_view.clone(),
                client: client.clone(),
                tabs,
                extensions: echo_extensions::Extensions::new(flags::extensions_dir())
                    .with_profile(echo_extensions::profile::default_profile(&flags::data_dir())),
                shield: shield.clone(),
                library,
            });
            crate::selftest::schedule();
            crate::sleep::start();
            crate::control::start();
            crate::overlay::arm_test();

            let mut delegate = window::BrowserWindowDelegate::new(
                RefCell::new(chrome_view),
                RefCell::new(host),
                RuntimeStyle::ALLOY,
                ShowState::NORMAL,
            );
            cef::window_create_top_level(Some(&mut delegate));

            // La premiere page n'est demandee qu'une fois la fenetre debout. Navigure
            // plus tot, elle partait avant que Chromium ait fini d'etablir sa pile
            // reseau : la page d'accueil echouait a chaque lancement sur un
            // `ERR_PROXY_CONNECTION_FAILED`, puis se rechargeait — le temps perdu se
            // voyait a l'ecran.
            restore_or_open();
        }

        fn default_client(&self) -> Option<Client> {
            self.client.borrow().clone()
        }
    }
}

/// Ouvre la bibliotheque. Un echec n'empeche pas de naviguer : mieux vaut un
/// navigateur sans favoris qu'un navigateur qui refuse de demarrer.
fn open_library() -> std::sync::Arc<echo_library::Library> {
    match echo_library::Library::open(&flags::data_dir()) {
        Ok(library) => std::sync::Arc::new(library),
        Err(err) => {
            tracing::warn!(%err, "bibliotheque indisponible, repli en memoire");
            std::sync::Arc::new(
                echo_library::Library::in_memory().expect("base en memoire"),
            )
        }
    }
}

/// Le reglage « retrouver la session » : actif par defaut.
fn restore_enabled() -> bool {
    crate::session::with(|s| {
        echo_library::settings::all(&s.library)
            .into_iter()
            .find(|(key, _)| key == "session.restore")
            .map(|(_, value)| value != echo_library::settings::Value::Flag(false))
    })
    .flatten()
    .unwrap_or(true)
}

/// Reprend les onglets laisses par une relance, ou ouvre la page d'accueil.
fn restore_or_open() {
    let data_dir = flags::data_dir();
    // Une relance voulue reprend tout, en direct ; un demarrage ordinaire reprend la derniere session,
    // dont seul l'onglet actif est charge (les autres dorment : demarrage rapide, memoire sobre).
    let (snapshot, live) = match crate::restart::take(&data_dir) {
        Some(snapshot) => (snapshot, true),
        None if restore_enabled() => match crate::restart::load_last(&data_dir) {
            Some(snapshot) => (snapshot, false),
            None => return crate::bridge::open_tab(&home_url()),
        },
        None => return crate::bridge::open_tab(&home_url()),
    };
    for (index, tab) in snapshot.tabs.iter().enumerate() {
        let Some(url) = tab.current() else { continue };
        if !live && index != snapshot.active {
            crate::session::with(|s| s.tabs.adopt_asleep(tab));
            continue;
        }
        crate::bridge::open_tab(url);
        let opened = crate::session::with(|s| s.tabs.active_id()).flatten();
        if let Some(id) = opened {
            let (history, position, pinned) = (tab.history.clone(), tab.position, tab.pinned);
            crate::session::with(|s| {
                s.tabs.restore_history(id, history, position);
                if let Some(entry) = s.tabs.get_mut(id) {
                    entry.pinned = pinned;
                }
            });
        }
    }
    let restored = crate::session::with(|s| {
        s.tabs.snapshot().get(snapshot.active).map(|tab| tab.id)
    })
    .flatten();
    if let Some(id) = restored {
        crate::session::with(|s| s.tabs.select(id));
    }
    crate::bridge::publish_tabs();
}

/// Prepare le bouclier. Le chargement des listes se fait a cote du demarrage : le navigateur
/// s'ouvre tout de suite, le filtrage prend effet des que le moteur est pret.
fn load_shield() -> std::sync::Arc<echo_shield::Shield> {
    let shield = std::sync::Arc::new(echo_shield::Shield::new(flags::data_dir()));
    flags::seed_shield_data();
    let background = shield.clone();
    std::thread::spawn(move || {
        if let Err(err) = background.refresh_lists(false) {
            tracing::warn!(%err, "listes de filtrage non rafraichies");
        }
        match background.load() {
            Ok(()) => tracing::info!("bouclier operationnel"),
            Err(err) => tracing::warn!(%err, "bouclier indisponible — navigation sans filtrage"),
        }
    });
    shield
}

/// Page ouverte dans la vue contenu au demarrage.
fn home_url() -> String {
    let Some(command_line) = command_line_get_global() else {
        return crate::search::HOME.to_string();
    };
    let value = CefString::from(&command_line.switch_value(Some(&CefString::from("url")))).to_string();
    if value.is_empty() { crate::search::HOME.to_string() } else { value }
}

/// L'interface du navigateur. Toujours la meme : `--url=` designe la page a ouvrir
/// dans l'onglet, pas la surface qui affiche la barre laterale.
fn startup_url() -> String {
    STARTUP_URL.to_string()
}
