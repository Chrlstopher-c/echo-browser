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

        fn download_handler(&self) -> Option<DownloadHandler> {
            Some(crate::transfers::Transfers::new(()))
        }

        fn permission_handler(&self) -> Option<PermissionHandler> {
            Some(crate::permissions::EchoPermissions::new(()))
        }

        fn keyboard_handler(&self) -> Option<KeyboardHandler> {
            Some(crate::shortcuts::BrowserShortcuts::new(()))
        }

        fn context_menu_handler(&self) -> Option<ContextMenuHandler> {
            Some(EchoContextMenu::new(()))
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
            browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            _transition: TransitionType,
        ) {
            let Some(frame) = frame else { return };
            let url = CefString::from(&frame.url()).to_string();
            if frame.is_main() == 1 {
                if let Some(browser) = browser {
                    crate::bridge::set_tab_dirty(browser.identifier(), false);
                    crate::bridge::reset_tab_scroll(browser.identifier());
                }
                if url.starts_with("http") {
                    frame.execute_java_script(
                        Some(&CefString::from(crate::sleep::DIRTY_WATCHER)),
                        Some(&CefString::from("echo://sleep")),
                        0,
                    );
                }
            }
            if frame.is_main() == 1 && !is_interface_page(&url) && crate::roundness::enabled() {
                frame.execute_java_script(
                    Some(&CefString::from(crate::roundness::install_script(crate::window::accent_now()).as_str())),
                    Some(&CefString::from("echo://roundness")),
                    0,
                );
            }
            if is_interface_page(&url) {
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
        /// Une page qui echoue le dit ici, et nulle part ailleurs : sans ce rappel, un
        /// `ERR_BLOCKED_BY_CLIENT` ne se lit qu'a l'ecran.
        fn on_load_error(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            error_code: Errorcode,
            error_text: Option<&CefString>,
            failed_url: Option<&CefString>,
        ) {
            let url = failed_url.map(CefString::to_string).unwrap_or_default();
            let texte = error_text.map(CefString::to_string).unwrap_or_default();
            tracing::warn!(%url, %texte, code = error_code.get_raw(), "chargement en echec");
        }

        fn on_load_end(&self, browser: Option<&mut Browser>, frame: Option<&mut Frame>, _status: i32) {
            let Some(frame) = frame else { return };
            let url = CefString::from(&frame.url()).to_string();
            if frame.is_main() == 1 && url.starts_with("http") {
                let pending = browser.and_then(|b| crate::bridge::take_pending_scroll(b.identifier()));
                if let Some(y) = pending {
                    let script = crate::sleep::restore_script(y);
                    frame.execute_java_script(Some(&CefString::from(script.as_str())), Some(&CefString::from("echo://sleep")), 0);
                }
            }
            if !is_interface_page(&url) {
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
            if is_interface_page(&url) {
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
        /// L'icone du site : la premiere adresse annoncee par la page, que l'onglet affiche.
        fn on_favicon_urlchange(&self, browser: Option<&mut Browser>, icon_urls: Option<&mut CefStringList>) {
            let (Some(browser), Some(icon_urls)) = (browser, icon_urls) else { return };
            if let Some(icon) = first_icon(icon_urls) {
                crate::bridge::set_tab_favicon(browser.identifier(), &icon);
            }
        }

        /// Le vrai titre de la page, celui que l'onglet et l'historique affichent.
        fn on_title_change(&self, browser: Option<&mut Browser>, title: Option<&CefString>) {
            let Some(browser) = browser else { return };
            let title = title.map(CefString::to_string).unwrap_or_default();
            crate::bridge::set_tab_title(browser.identifier(), &title);
        }

        /// La page passe en plein ecran, ou en sort : l'interface doit s'effacer.
        fn on_fullscreen_mode_change(&self, _browser: Option<&mut Browser>, fullscreen: i32) {
            crate::bridge::set_fullscreen(fullscreen == 1);
        }

        /// Remonte la console de la page dans le journal : sans elle, une erreur de script
        /// se traduit par une fenetre blanche et aucune trace.
        fn on_console_message(
            &self,
            browser: Option<&mut Browser>,
            level: LogSeverity,
            message: Option<&CefString>,
            source: Option<&CefString>,
            line: i32,
        ) -> i32 {
            let message = message.map(CefString::to_string).unwrap_or_default();
            if let Some(y) = message.strip_prefix(crate::sleep::SCROLL_MARKER).and_then(|v| v.parse::<i32>().ok()) {
                if let Some(browser) = browser {
                    crate::bridge::set_tab_scroll(browser.identifier(), y);
                }
                return 1;
            }
            let source_text = source.map(CefString::to_string).unwrap_or_default();
            if source_text.contains("/gsi/") && message.contains("reading 'postMessage'") {
                if let Some(browser) = browser.as_deref() {
                    crate::bridge::finish_orphan_signin(browser.identifier());
                }
            }
            if message == crate::sleep::PRESS_MARKER {
                if crate::overlay::menu_open() {
                    crate::overlay::close_menu();
                }
                return 1;
            }
            if message == crate::sleep::DIRTY_MARKER {
                if let Some(browser) = browser {
                    crate::bridge::set_tab_dirty(browser.identifier(), true);
                }
                return 1;
            }
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
        fn on_after_created(&self, browser: Option<&mut Browser>) {
            if let Some(browser) = browser.as_deref() {
                crate::identity::emulate(browser);
                crate::scheme::apply(browser);
            }
            self.live.opened();
        }

        /// Sans ce rappel, une page qui appelle `window.close()` ferme la fenetre entiere d'Echo.
        fn do_close(&self, browser: Option<&mut Browser>) -> i32 {
            browser.map_or(0, |b| i32::from(crate::bridge::page_asks_close(b.identifier())))
        }

        fn on_before_close(&self, _browser: Option<&mut Browser>) {
            // Ne jamais quitter ici : fermer un onglet detruit plusieurs vues en cascade
            // et le compteur passe par zero alors que la fenetre est toujours la.
            // La sortie appartient a la fenetre — voir `on_window_destroyed`.
            self.live.closed();
        }
    }
}

// Le clic droit du navigateur. Chromium propose quatre entrees en anglais, sans rapport
// avec la cible : on prend la main et on sert le notre.
wrap_context_menu_handler! {
    pub struct EchoContextMenu {
        marker: (),
    }

    impl ContextMenuHandler {
        /// Laisse le modele de Chromium intact : `run_context_menu` le remplace par notre menu.
        fn on_before_context_menu(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _params: Option<&mut ContextMenuParams>,
            model: Option<&mut MenuModel>,
        ) {
            // Ne pas vider le modele : sans entree, Chromium n'appelle plus `run_context_menu`.
            let _ = model;
        }

        /// Prend la main sur l'affichage. Rendre 1 dit a Chromium que le menu est a nous.
        fn run_context_menu(
            &self,
            _browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            params: Option<&mut ContextMenuParams>,
            _model: Option<&mut MenuModel>,
            callback: Option<&mut RunContextMenuCallback>,
        ) -> i32 {
            let Some(params) = params else { return 0 };
            // La barre a ses propres menus (page) : celui des sites n'a rien a y faire.
            let url = frame.map(|f| CefString::from(&f.url()).to_string()).unwrap_or_default();
            if is_interface_page(&url) {
                if let Some(callback) = callback {
                    callback.cancel();
                }
                return 1;
            }
            let click = read_click(params);
            let (x, y) = (params.xcoord(), params.ycoord());
            // Le rappel se referme tout de suite : notre menu ne passe pas par lui, et
            // le laisser ouvert bloquerait la page sur un menu invisible.
            if let Some(callback) = callback {
                callback.cancel();
            }
            crate::bridge::context::open(click, x, y);
            1
        }

        fn on_context_menu_dismissed(&self, _browser: Option<&mut Browser>, _frame: Option<&mut Frame>) {
            crate::bridge::context::close();
        }
    }
}

/// Traduit ce que Chromium rapporte du clic en ce dont le menu a besoin.
fn read_click(params: &ContextMenuParams) -> crate::menu::Click {
    let text = |value: CefStringUserfree| CefString::from(&value).to_string();
    let browser = crate::session::with(|s| s.tabs.active().and_then(|tab| tab.browser())).flatten();
    crate::menu::Click {
        link: text(params.link_url()),
        image: if params.has_image_contents() == 1 { text(params.source_url()) } else { String::new() },
        selection: text(params.selection_text()),
        page: text(params.page_url()),
        editable: params.is_editable() == 1,
        can_go_back: browser.as_ref().map(|b| b.can_go_back() == 1).unwrap_or(false),
        can_go_forward: browser.as_ref().map(|b| b.can_go_forward() == 1).unwrap_or(false),
    }
}

/// Vrai pour les pages de l'interface elle-meme ; la page d'accueil, servie par le meme
/// schema, est une page d'onglet comme une autre (chargement, titre, veille).
fn is_interface_page(url: &str) -> bool {
    url.starts_with("echo://ui/") && url != crate::search::HOME && url != crate::terminal::PAGE
}

/// La premiere adresse d'icone de la liste que Chromium prete pour le rappel.
fn first_icon(list: &mut CefStringList) -> Option<String> {
    let raw: *mut cef::sys::_cef_string_list_t = list.into();
    if raw.is_null() {
        return None;
    }
    // SAFETY : `raw` designe la liste que Chromium garde vivante pendant ce rappel ; on ne fait que la lire.
    unsafe {
        if cef::sys::cef_string_list_size(raw) == 0 {
            return None;
        }
        let mut value = std::mem::zeroed();
        (cef::sys::cef_string_list_value(raw, 0, &mut value) > 0)
            .then(|| CefString::from(std::ptr::from_ref(&value)).to_string())
    }
}
