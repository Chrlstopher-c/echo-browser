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
            let script = crate::injection::page_script(&url, &self.shield)?;
            Some(crate::injection::filter::new_filter(script))
        }

        fn on_before_resource_load(
            &self,
            _browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            request: Option<&mut Request>,
            _callback: Option<&mut Callback>,
        ) -> ReturnValue {
            let Some(request) = request else { return ReturnValue::CONTINUE };
            let url = CefString::from(&request.url()).to_string();
            if est_interne(&url) {
                return ReturnValue::CONTINUE;
            }
            let source = frame
                .map(|frame| CefString::from(&frame.url()).to_string())
                .unwrap_or_default();
            let kind = resource_kind(request.resource_type());
            let method = CefString::from(&request.method()).to_string();

            crate::identity::apply(request);

            match self.shield.decide(0, &url, &source, kind, &method) {
                Verdict::Allow => ReturnValue::CONTINUE,
                Verdict::Block { .. } | Verdict::Redirect { .. } => {
                    debug!(%url, %source, %kind, "requete bloquee");
                    ReturnValue::CANCEL
                }
            }
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
