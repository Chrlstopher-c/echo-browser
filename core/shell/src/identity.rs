//! Responsabilite : ce que le navigateur declare de lui-meme aux sites qu'il visite.
//!
//! Chromium embarque s'annonce « Chromium » et reclame ses pages en anglais. Les deux
//! detonnent : les services grand public traitent cette combinaison comme du trafic
//! automatise et repondent par un captcha. Mesure le 2026-09-10 sur Google.

use cef::{CefString, ImplRequest, Request};

/// Version de Chromium embarquee. A tenir alignee avec le crate `cef`.
const CHROME_VERSION: &str = "152";
const CHROME_FULL_VERSION: &str = "152.0.7977.54";

/// Langues demandees, sans ponderation : Chromium ajoute les siennes. En fournir
/// deja ponderees produit un en-tete a doublons, visible de loin.
/// Une adresse francaise qui reclame ses pages en anglais est un signal d'automate.
pub const ACCEPT_LANGUAGE: &str = "fr-FR,fr,en-US,en";

/// Aligne les en-tetes d'identification sur ceux d'un Chrome de bureau.
pub fn apply(request: &Request) {
    let brands = format!(
        r#""Not?A_Brand";v="24", "Chromium";v="{CHROME_VERSION}", "Google Chrome";v="{CHROME_VERSION}""#
    );
    let full_versions = format!(
        r#""Not?A_Brand";v="24.0.0.0", "Chromium";v="{CHROME_FULL_VERSION}", "Google Chrome";v="{CHROME_FULL_VERSION}""#
    );
    set(request, "sec-ch-ua", &brands);
    set(request, "sec-ch-ua-full-version-list", &full_versions);
    set(request, "sec-ch-ua-full-version", &format!("\"{CHROME_FULL_VERSION}\""));
    set(request, "sec-ch-ua-arch", "\"x86\"");
    set(request, "sec-ch-ua-bitness", "\"64\"");
    set(request, "sec-ch-ua-model", "\"\"");
    set(request, "sec-ch-ua-platform", "\"Linux\"");
    set(request, "sec-ch-ua-platform-version", "\"6.1.0\"");
    set(request, "sec-ch-ua-wow64", "?0");
}

fn set(request: &Request, name: &str, value: &str) {
    request.set_header_by_name(
        Some(&CefString::from(name)),
        Some(&CefString::from(value)),
        1,
    );
}
