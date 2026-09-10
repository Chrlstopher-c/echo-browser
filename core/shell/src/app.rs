//! Responsabilite : le point de contact avec Chromium — drapeaux au demarrage, creation de la fenetre.

use crate::{client::EchoClient, flags, window};
// Les macros `wrap_*` de CEF exigent les traits `Impl*` et `Wrap*` dans la portee : import global impose.
use cef::*;
use std::cell::RefCell;
use tracing::info;

/// Page servie au demarrage tant que l'interface n'est pas empaquetee.
const STARTUP_URL: &str = "https://www.qwant.com/";

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
            let url = startup_url();
            info!(%url, "contexte Chromium pret, ouverture de la fenetre");
            *self.client.borrow_mut() = Some(EchoClient::new(Default::default()));
            let mut client = self.client.borrow().clone();
            let chrome_view = window::create_chrome_view(client.as_mut(), &url);
            let mut delegate = window::BrowserWindowDelegate::new(
                RefCell::new(chrome_view),
                RuntimeStyle::ALLOY,
                ShowState::NORMAL,
            );
            cef::window_create_top_level(Some(&mut delegate));
        }

        fn default_client(&self) -> Option<Client> {
            self.client.borrow().clone()
        }
    }
}

/// URL d'ouverture : `--url=` si fournie, sinon la page de demarrage.
fn startup_url() -> String {
    let Some(command_line) = command_line_get_global() else {
        return STARTUP_URL.to_string();
    };
    let value = CefString::from(&command_line.switch_value(Some(&CefString::from("url")))).to_string();
    if value.is_empty() { STARTUP_URL.to_string() } else { value }
}
