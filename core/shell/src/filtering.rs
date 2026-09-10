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
        fn on_before_resource_load(
            &self,
            _browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            request: Option<&mut Request>,
            _callback: Option<&mut Callback>,
        ) -> ReturnValue {
            let Some(request) = request else { return ReturnValue::CONTINUE };
            let url = CefString::from(&request.url()).to_string();
            if url.starts_with("echo://") {
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
