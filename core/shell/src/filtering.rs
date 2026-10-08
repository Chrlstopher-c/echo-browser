//! Responsabilite : soumettre chaque requete du navigateur au bouclier, et appliquer sa decision.

// Les macros `wrap_*` de CEF exigent les traits `Impl*` et `Wrap*` dans la portee : import global impose.
use cef::*;
use echo_shield::verdict::Verdict;
use echo_shield::Shield;
use std::sync::Arc;
use tracing::debug;

wrap_request_handler! {
    pub struct FilteringRequestHandler {
        shield: Arc<Shield>,
    }

    impl RequestHandler {
        fn resource_request_handler(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _request: Option<&mut Request>,
            _is_navigation: i32,
            _is_download: i32,
            _request_initiator: Option<&CefString>,
            _disable_default_handling: Option<&mut i32>,
        ) -> Option<ResourceRequestHandler> {
            Some(ShieldResourceHandler::new(self.shield.clone()))
        }
    }
}

wrap_resource_request_handler! {
    struct ShieldResourceHandler {
        shield: Arc<Shield>,
    }

    impl ResourceRequestHandler {
        /// Glisse le traitement du bouclier dans le document lui-meme, seul moment
        /// ou il precede vraiment les scripts du site.
        fn resource_response_filter(
            &self,
            _browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            request: Option<&mut Request>,
            response: Option<&mut Response>,
        ) -> Option<ResponseFilter> {
            let request = request?;
            // Le type de ressource n'est pas encore renseigne a ce stade : la page
            // principale s'y annonce « autre ». On se fie a la frame, qui, elle, est sure.
            if frame.map(|f| f.is_main()) != Some(1) {
                return None;
            }
            let response = response?;
            let mime = CefString::from(&response.mime_type()).to_string();
            if !mime.eq_ignore_ascii_case("text/html") {
                return None;
            }
            let url = CefString::from(&request.url()).to_string();
            // Les pages du navigateur (DevTools, echo://) ne recoivent rien : leurs fichiers internes servis
            // en text/html (traductions des DevTools) etaient sinon corrompus par l'injection.
            if est_interne(&url) {
                return None;
            }
            let script = crate::injection::page_script(&url, &self.shield)?;
            Some(crate::injection::filter::new_filter(script))
        }

        fn on_before_resource_load(
            &self,
            browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            request: Option<&mut Request>,
            _callback: Option<&mut Callback>,
        ) -> ReturnValue {
            let Some(request) = request else { return ReturnValue::CONTINUE };
            let url = CefString::from(&request.url()).to_string();
            if est_interne(&url) {
                return ReturnValue::CONTINUE;
            }
            let browser_id = browser.as_ref().map_or(0, |b| b.identifier());
            let top = browser
                .and_then(|b| b.main_frame())
                .map(|f| CefString::from(&f.url()).to_string())
                .unwrap_or_default();
            let source = frame
                .map(|frame| CefString::from(&frame.url()).to_string())
                .unwrap_or_default();
            let main_frame = request.resource_type() == ResourceType::MAIN_FRAME;
            let kind = resource_kind(request.resource_type());
            let method = CefString::from(&request.method()).to_string();
            let page = if main_frame { url.as_str() } else { top.as_str() };
            // Le compteur « bloqués sur cette page » repart de zero a chaque nouvelle page de l'onglet.
            let tab_key = u32::try_from(browser_id).unwrap_or(0);
            if main_frame {
                self.shield.reset_tab(tab_key);
            }
            let blocked = self.verdict(request, (&url, page, &source), kind, &method, tab_key);
            if blocked.is_none() && DO_NOT_TRACK.load(std::sync::atomic::Ordering::Relaxed) {
                for name in ["DNT", "Sec-GPC"] {
                    request.set_header_by_name(Some(&CefString::from(name)), Some(&CefString::from("1")), 1);
                }
            }
            crate::network::started(crate::network::Started {
                browser: browser_id,
                id: request.identifier(),
                url: &url,
                page,
                kind,
                method: &method,
                blocked,
                main_frame,
            });
            if blocked == Some("bouclier") {
                crate::signals::note_blocked(&url);
            }
            if blocked.is_some() {
                debug!(%url, %source, %kind, ?blocked, "requete bloquee");
                return ReturnValue::CANCEL;
            }
            ReturnValue::CONTINUE
        }

        fn on_resource_load_complete(
            &self,
            browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            request: Option<&mut Request>,
            response: Option<&mut Response>,
            _status: UrlrequestStatus,
            received_content_length: i64,
        ) {
            let (Some(browser), Some(request)) = (browser, request) else { return };
            let status = response.map_or(0, |r| u16::try_from(r.status()).unwrap_or(0));
            crate::network::completed(browser.identifier(), request.identifier(), status,
                u64::try_from(received_content_length).unwrap_or(0));
        }
    }
}

impl ShieldResourceHandler {
    /// La raison de bloquer : regle de l'utilisateur d'abord (toujours appliquee), puis le bouclier si le site n'est
    /// pas en exception (une exception vaut pour le site entier, cadres externes compris).
    fn verdict(&self, request: &mut Request, (url, page, source): (&str, &str, &str), kind: &str, method: &str,
        tab_key: u32) -> Option<&'static str> {
        crate::identity::apply(request);
        if let Some(reason) = crate::network::rule_verdict(url, page) {
            return Some(reason);
        }
        if !page.is_empty() && !self.shield.is_active_for(page) {
            return None;
        }
        match self.shield.decide(tab_key, url, source, kind, method) {
            Verdict::Allow => None,
            Verdict::Block { .. } | Verdict::Redirect { .. } => Some("bouclier"),
        }
    }
}

/// Une adresse interne au navigateur ne passe jamais par le bouclier.
///
/// Les listes de filtres sont ecrites pour le web : appliquees aux pages du navigateur
/// lui-meme, elles bloquent au hasard des chemins qui ressemblent a de la publicite.
/// Mesure le 2026-09-10 : la fenetre de Dark Reader, `chrome-extension://…/ui/popup/`,
/// rendait un `ERR_BLOCKED_BY_CLIENT` — bloquee par notre propre bouclier.
fn est_interne(url: &str) -> bool {
    const SCHEMAS: [&str; 7] = [
        "echo://",
        "chrome-extension://",
        "chrome://",
        "chrome-untrusted://",
        "devtools://",
        "blob:",
        "about:",
    ];
    SCHEMAS.iter().any(|schema| url.starts_with(schema))
}

/// Traduit le type de ressource de Chromium vers le vocabulaire des listes de filtres.
fn resource_kind(kind: ResourceType) -> &'static str {
    match kind {
        ResourceType::MAIN_FRAME => "main_frame",
        ResourceType::SUB_FRAME => "sub_frame",
        ResourceType::STYLESHEET => "stylesheet",
        ResourceType::SCRIPT => "script",
        ResourceType::IMAGE => "image",
        ResourceType::FONT_RESOURCE => "font",
        ResourceType::MEDIA => "media",
        ResourceType::PING => "ping",
        ResourceType::FAVICON => "image",
        ResourceType::OBJECT => "object",
        ResourceType::WORKER | ResourceType::SHARED_WORKER | ResourceType::SERVICE_WORKER => "script",
        ResourceType::CSP_REPORT => "csp_report",
        ResourceType::XHR => "xhr",
        _ => "other",
    }
}

#[cfg(test)]
mod tests {
    use super::est_interne;

    #[test]
    fn les_pages_du_navigateur_echappent_au_bouclier() {
        assert!(est_interne("chrome-extension://abc/ui/popup/index.html"));
        assert!(est_interne("echo://ui/index.html"));
        assert!(est_interne("about:blank"));
        assert!(est_interne("devtools://devtools/bundled/inspector.html"));
    }

    #[test]
    fn le_web_reste_filtre() {
        assert!(!est_interne("https://ads.example.com/track.js"));
        assert!(!est_interne("http://www.google.com/"));
    }
}

/// Reglage « Demander à ne pas être suivi », lu ici sur le thread reseau (la session n'y est pas accessible).
static DO_NOT_TRACK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// Relit les reglages qui touchent aux requetes ; a appeler sur le thread interface au demarrage et a chaque changement.
pub fn refresh_settings() {
    use echo_library::settings::Value;
    let dnt = crate::session::with(|s| echo_library::settings::get(&s.library, "privacy.send_do_not_track")).flatten();
    DO_NOT_TRACK.store(!matches!(dnt, Some(Value::Flag(false))), std::sync::atomic::Ordering::Relaxed);
}
