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

    fn closed(&self) -> bool {
        let mut count = self.0.lock();
        *count = count.saturating_sub(1);
        info!(vivants = *count, "vue navigateur fermee");
        *count == 0
    }
}

wrap_client! {
    pub struct EchoClient {
        live: LiveBrowsers,
    }

    impl Client {
        fn life_span_handler(&self) -> Option<LifeSpanHandler> {
            Some(EchoLifeSpanHandler::new(self.live.clone()))
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
            if self.live.closed() {
                quit_message_loop();
            }
        }
    }
}
