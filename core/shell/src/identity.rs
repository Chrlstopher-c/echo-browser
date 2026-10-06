//! Responsabilite : ce que le navigateur declare de lui-meme aux sites qu'il visite.
//!
//! Chromium embarque s'annonce « Chromium » et reclame ses pages en anglais. Les deux
//! detonnent : les services grand public traitent cette combinaison comme du trafic
//! automatise et repondent par un captcha. Mesure le 2026-09-10 sur Google.

use cef::{CefString, ImplRequest, Request};

/// Version de Chromium embarquee. A tenir alignee avec les binaires CEF (`tools/fetch-cef.sh`) : un en-tete
/// qui annonce 152 pendant que le moteur dit 154 est un signal d'automate.
const CHROME_VERSION: &str = "154";
const CHROME_FULL_VERSION: &str = "154.0.8037.94";

/// Marque aleatoire que Chrome ajoute a sa liste, telle que le Chrome 154 de bureau la declare.
const GREASE_BRAND: &str = "Not A(Brand";
const GREASE_VERSION: &str = "99";

/// Langues demandees, sans ponderation : Chromium ajoute les siennes. En fournir
/// deja ponderees produit un en-tete a doublons, visible de loin.
/// Une adresse francaise qui reclame ses pages en anglais est un signal d'automate.
pub const ACCEPT_LANGUAGE: &str = "fr-FR,fr,en-US,en";

/// Aligne les en-tetes d'identification sur ceux d'un Chrome de bureau.
pub fn apply(request: &Request) {
    let brands = format!(
        r#""Chromium";v="{CHROME_VERSION}", "Google Chrome";v="{CHROME_VERSION}", "{GREASE_BRAND}";v="{GREASE_VERSION}""#
    );
    let full_versions = format!(
        r#""Chromium";v="{CHROME_FULL_VERSION}", "Google Chrome";v="{CHROME_FULL_VERSION}", "{GREASE_BRAND}";v="{GREASE_VERSION}.0.0.0""#
    );
    set(request, "sec-ch-ua", &brands);
    set(request, "sec-ch-ua-full-version-list", &full_versions);
    set(request, "sec-ch-ua-full-version", &format!("\"{CHROME_FULL_VERSION}\""));
    set(request, "sec-ch-ua-arch", "\"x86\"");
    set(request, "sec-ch-ua-bitness", "\"64\"");
    set(request, "sec-ch-ua-model", "\"\"");
    set(request, "sec-ch-ua-platform", "\"Linux\"");
    set(request, "sec-ch-ua-platform-version", "\"\"");
    set(request, "sec-ch-ua-wow64", "?0");
}

fn set(request: &Request, name: &str, value: &str) {
    request.set_header_by_name(
        Some(&CefString::from(name)),
        Some(&CefString::from(value)),
        1,
    );
}

/// Aligne aussi ce que les pages lisent en JavaScript (`navigator.userAgentData`) sur les en-tetes : sinon les
/// deux se contredisent (marque « Google Chrome » dans l'un, absente de l'autre). Passe par le protocole de
/// debogage de Chromium, comme le font les outils de test.
pub fn emulate(browser: &cef::Browser) {
    use cef::{ImplBrowser, ImplBrowserHost};
    let message = serde_json::json!({
        "id": 1,
        "method": "Emulation.setUserAgentOverride",
        "params": {
            "userAgent": format!("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{CHROME_VERSION}.0.0.0 Safari/537.36"),
            "acceptLanguage": ACCEPT_LANGUAGE,
            "platform": "Linux x86_64",
            "userAgentMetadata": {
                "brands": [
                    {"brand": "Chromium", "version": CHROME_VERSION},
                    {"brand": "Google Chrome", "version": CHROME_VERSION},
                    {"brand": GREASE_BRAND, "version": GREASE_VERSION},
                ],
                "fullVersionList": [
                    {"brand": "Chromium", "version": CHROME_FULL_VERSION},
                    {"brand": "Google Chrome", "version": CHROME_FULL_VERSION},
                    {"brand": GREASE_BRAND, "version": format!("{GREASE_VERSION}.0.0.0")},
                ],
                "platform": "Linux",
                "platformVersion": "",
                "architecture": "x86",
                "model": "",
                "mobile": false,
                "bitness": "64",
                "wow64": false,
            },
        },
    });
    if let Some(host) = browser.host() {
        host.send_dev_tools_message(Some(message.to_string().as_bytes()));
    }
}
