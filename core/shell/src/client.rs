//! Responsabilite : le client CEF — ce que Chromium rappelle pendant la vie d'un navigateur.

// Les macros `wrap_*` de CEF exigent les traits `Impl*` et `Wrap*` dans la portee : import global impose.
use cef::*;
use parking_lot::Mutex;
use std::sync::Arc;
use tracing::info;

/// Nombre de vues navigateur vivantes. La boucle de messages s'arrete quand il retombe a zero.
#[derive(Clone, Default)]
pub struct LiveBrowsers(Arc<Mutex<usize>>);

impl LiveBrowsers {
    fn opened(&self) {
        let mut count = self.0.lock();
        *count += 1;
        info!(vivants = *count, "vue navigateur ouverte");
    }

    fn closed(&self) {
        let mut count = self.0.lock();
        *count = count.saturating_sub(1);
        info!(vivants = *count, "vue navigateur fermee");
    }
}

wrap_client! {
    pub struct EchoClient {
        live: LiveBrowsers,
        shield: std::sync::Arc<echo_shield::Shield>,
    }

    impl Client {
        fn request_handler(&self) -> Option<RequestHandler> {
            Some(crate::filtering::FilteringRequestHandler::new(self.shield.clone()))
        }

        fn life_span_handler(&self) -> Option<LifeSpanHandler> {
            Some(EchoLifeSpanHandler::new(self.live.clone()))
        }

        fn display_handler(&self) -> Option<DisplayHandler> {
            Some(EchoDisplayHandler::new(()))
        }

        fn keyboard_handler(&self) -> Option<KeyboardHandler> {
            Some(crate::shortcuts::BrowserShortcuts::new(()))
        }

        fn load_handler(&self) -> Option<LoadHandler> {
            Some(EchoLoadHandler::new(self.shield.clone()))
        }
    }
}

wrap_load_handler! {
    struct EchoLoadHandler {
        shield: std::sync::Arc<echo_shield::Shield>,
    }

    impl LoadHandler {
        /// Injecte le pont dans la page d'interface avant que ses scripts ne s'executent.
        fn on_load_start(
            &self,
            _browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            _transition: TransitionType,
        ) {
            let Some(frame) = frame else { return };
            let url = CefString::from(&frame.url()).to_string();
            if url.starts_with("echo://ui/") {
                frame.execute_java_script(
                    Some(&CefString::from(crate::bridge::script::BOOTSTRAP)),
                    Some(&CefString::from("echo://bridge")),
                    0,
                );
                info!("pont injecte dans l'interface");
                return;
            }
            crate::injection::treat_page(frame, &self.shield);
        }

        /// L'interface est chargee : elle a besoin de son etat de depart, sinon elle
        /// ignore jusqu'a l'existence de l'onglet et reste inerte.
        fn on_load_end(&self, _browser: Option<&mut Browser>, frame: Option<&mut Frame>, _status: i32) {
            let Some(frame) = frame else { return };
            let url = CefString::from(&frame.url()).to_string();
            if !url.starts_with("echo://ui/") {
                return;
            }
            crate::bridge::publish_initial_state();
        }

        fn on_loading_state_change(
            &self,
            browser: Option<&mut Browser>,
            is_loading: i32,
            _can_go_back: i32,
            _can_go_forward: i32,
        ) {
            let Some(browser) = browser else { return };
            let browser_id = browser.identifier();
            let Some(url) = browser.main_frame().map(|f| CefString::from(&f.url()).to_string()) else {
                return;
            };
            if url.starts_with("echo://ui/") {
                return;
            }
            crate::bridge::publish_tab(browser_id, &url, &url, is_loading == 1);
            crate::bridge::publish_shield();
        }
    }
}

wrap_display_handler! {
    struct EchoDisplayHandler {
        marker: (),
    }

    impl DisplayHandler {
        /// Remonte la console de la page dans le journal : sans elle, une erreur de script
        /// se traduit par une fenetre blanche et aucune trace.
        fn on_console_message(
            &self,
            _browser: Option<&mut Browser>,
            level: LogSeverity,
            message: Option<&CefString>,
            source: Option<&CefString>,
            line: i32,
        ) -> i32 {
            let message = message.map(CefString::to_string).unwrap_or_default();
            let source = source.map(CefString::to_string).unwrap_or_default();
            match level {
                LogSeverity::ERROR | LogSeverity::FATAL => {
                    tracing::error!(%source, %line, "console: {message}")
                }
                LogSeverity::WARNING => tracing::warn!(%source, %line, "console: {message}"),
                _ => tracing::info!(%source, %line, "console: {message}"),
            }
            0
        }
    }
}

wrap_life_span_handler! {
    struct EchoLifeSpanHandler {
        live: LiveBrowsers,
    }

    impl LifeSpanHandler {
        fn on_after_created(&self, _browser: Option<&mut Browser>) {
            self.live.opened();
        }

        fn on_before_close(&self, _browser: Option<&mut Browser>) {
            // Ne jamais quitter ici : fermer un onglet detruit plusieurs vues en cascade
            // et le compteur passe par zero alors que la fenetre est toujours la.
            // La sortie appartient a la fenetre — voir `on_window_destroyed`.
            self.live.closed();
        }
    }
}
