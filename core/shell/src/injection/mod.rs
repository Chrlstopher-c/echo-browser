//! Responsabilite : appliquer dans la page ce que le blocage reseau ne peut pas faire —
//! masquer les emplacements publicitaires et neutraliser les detecteurs de bloqueur.
//!
//! Le moment compte autant que le contenu : injecte apres le premier script du site,
//! le traitement arrive trop tard et la regie a deja gagne.

pub mod filter;
mod twitch;

use cef::{CefString, Frame, ImplFrame};
use echo_shield::verdict::PageTreatment;
use echo_shield::Shield;
use tracing::debug;

/// Banc : `ECHO_BENCH_NO_INJECT=1` coupe masquage et scriptlets pour mesurer leur cout memoire.
fn bench_sans_injection() -> bool {
    std::env::var_os("ECHO_BENCH_NO_INJECT").is_some()
}

/// Le code a poser dans le document d'une page, ou `None` s'il n'y a rien a y faire.
pub fn page_script(url: &str, shield: &Shield) -> Option<String> {
    if url.is_empty() || url.starts_with("echo://") || url.starts_with("about:") || url.starts_with("chrome-extension://") || bench_sans_injection() {
        return None;
    }
    let treatment = shield.treat_page(url);
    let codecs = crate::codecs::shim();
    let twitch = twitch::script_for(url, shield);
    if treatment.is_empty() && codecs.is_none() && twitch.is_none() {
        return None;
    }
    debug!(
        %url,
        masquage = treatment.hide_selectors.len(),
        scriptlets = treatment.injected_script.len(),
        "traitement prepare"
    );
    let mut script = codecs.map(|c| format!("try {{ {c} }} catch (e) {{}}\n")).unwrap_or_default();
    if let Some(vaft) = twitch {
        script.push_str(vaft);
        script.push('\n');
    }
    script.push_str(&build(&treatment));
    Some(script)
}

/// Applique le traitement du bouclier a une page qui commence a charger.
///
/// Filet de securite pour les pages dont le flux HTML n'a pas pu etre filtre : le
/// traitement arrive alors apres les premiers scripts du site — trop tard pour un
/// anti-bloqueur, a temps pour le masquage.
pub fn treat_page(frame: &Frame, shield: &Shield) {
    let url = CefString::from(&frame.url()).to_string();
    if url.is_empty() || url.starts_with("echo://") || url.starts_with("about:") || url.starts_with("chrome-extension://") || bench_sans_injection() {
        return;
    }
    // Filet : sans le flux HTML filtre, vaft arrive apres les premiers scripts ; il agit des la chaine suivante.
    if let Some(vaft) = twitch::script_for(&url, shield) {
        run(frame, vaft);
    }
    let treatment = shield.treat_page(&url);
    if treatment.is_empty() {
        return;
    }
    debug!(
        %url,
        masquage = treatment.hide_selectors.len(),
        scriptlets = treatment.injected_script.len(),
        "traitement de la page"
    );
    run(frame, &build(&treatment));
}

/// Assemble le code a executer : le masquage d'abord, les scriptlets ensuite.
fn build(treatment: &PageTreatment) -> String {
    let mut script = String::with_capacity(treatment.injected_script.len() + 4096);
    if let Some(css) = treatment.hiding_stylesheet() {
        script.push_str(&hide_snippet(&css));
    }
    if !treatment.injected_script.is_empty() {
        script.push_str("try {\n");
        script.push_str(&treatment.injected_script);
        script.push_str("\n} catch (e) {}\n");
    }
    script
}

/// Pose la feuille de masquage. A ce stade `document.head` n'existe pas toujours :
/// on s'accroche a la racine, qui est toujours la.
fn hide_snippet(css: &str) -> String {
    format!(
        "try {{\n  const s = document.createElement('style');\n  \
         s.textContent = {};\n  \
         (document.head || document.documentElement).appendChild(s);\n\
         }} catch (e) {{}}\n",
        json_string(css)
    )
}

/// Encode une chaine en litteral JavaScript sur.
fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '<' => out.push_str("\\u003C"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn run(frame: &Frame, script: &str) {
    frame.execute_java_script(
        Some(&CefString::from(script)),
        Some(&CefString::from("echo://shield")),
        0,
    );
}
