//! Responsabilite : memoire de structure — chaque page calcule l'empreinte de son gabarit (squelette des balises et
//! classes stables, repetitions ecrasees) ; « Masquer cet element » retient un selecteur pour ce gabarit, applique a
//! toute page qui a le meme, sur le site ou ailleurs.

use std::collections::HashMap;
use std::sync::OnceLock;

use cef::*;
use parking_lot::Mutex;

pub const FINGERPRINT_MARKER: &str = "echo:gabarit:";
pub const HIDE_MARKER: &str = "echo:masquer:";

/// Fonctions partagees : empreinte du gabarit et selecteur stable d'un element (sans identifiants ni classes a chiffres).
const SHARED: &str = r#"const echoStable=c=>!/\d/.test(c)&&c.length<40;
const echoSig=e=>e.tagName.toLowerCase()+[...e.classList].filter(echoStable).sort().map(c=>'.'+c).join('');
const echoSkeleton=(e,d)=>{if(d>4)return '';let out='',prev='';for(const c of [...e.children].slice(0,60)){
if(['SCRIPT','STYLE','NOSCRIPT','TEMPLATE','LINK','META'].includes(c.tagName))continue;
const s=echoSig(c)+'('+echoSkeleton(c,d+1)+')';if(s!==prev)out+=s;prev=s}return out};
const echoPrint=()=>{const t=echoSkeleton(document.body,0);let h=2166136261;for(let i=0;i<t.length;i++){
h^=t.charCodeAt(i);h=Math.imul(h,16777619)>>>0}return h.toString(16)};
const echoSelector=e=>{const parts=[];for(let n=e;n&&n!==document.body&&parts.length<6;n=n.parentElement)
parts.unshift(echoSig(n));return 'body '+parts.join(' > ')};"#;

/// Joue en fin de chargement : annonce le gabarit.
pub fn announce_script() -> String {
    format!("(()=>{{{SHARED}console.debug('{FINGERPRINT_MARKER}'+echoPrint())}})()")
}

/// Joue a la demande : masque l'element sous le point (x, y) et annonce gabarit et selecteur.
pub fn hide_script(x: f32, y: f32) -> String {
    format!(
        "(()=>{{{SHARED}const e=document.elementFromPoint({x},{y});if(!e||e===document.body)return;\
const sel=echoSelector(e);e.style.setProperty('display','none','important');\
console.debug('{HIDE_MARKER}'+JSON.stringify({{g:echoPrint(),s:sel}}))}})()"
    )
}

static LAST: OnceLock<Mutex<HashMap<i32, String>>> = OnceLock::new();

fn last() -> &'static Mutex<HashMap<i32, String>> {
    LAST.get_or_init(|| Mutex::new(HashMap::new()))
}

fn css_script(selectors: &[String]) -> String {
    let rules: String = selectors.iter().map(|s| format!("{s}{{display:none!important}}")).collect();
    format!(
        "(()=>{{let s=document.getElementById('echo-masque');if(!s){{s=document.createElement('style');\
s.id='echo-masque';document.documentElement.appendChild(s)}}s.textContent={}}})()",
        serde_json::to_string(&rules).unwrap_or_else(|_| "''".into())
    )
}

/// La page a annonce son gabarit : on y applique ce qui a ete masque pour lui.
pub fn announced(browser: &Browser, fingerprint: &str) {
    if fingerprint.len() > 16 {
        return;
    }
    last().lock().insert(browser.identifier(), fingerprint.to_string());
    let selectors = crate::session::with(|s| echo_library::hidden::selectors(&s.library, fingerprint)).unwrap_or_default();
    if let (false, Some(frame)) = (selectors.is_empty(), browser.main_frame()) {
        frame.execute_java_script(Some(&CefString::from(css_script(&selectors).as_str())), None, 0);
    }
}

/// L'utilisateur a masque un element : retenu pour le gabarit de la page.
pub fn hidden(browser: &Browser, message: &str) {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(message) else { return };
    let (Some(fingerprint), Some(selector)) = (value["g"].as_str(), value["s"].as_str()) else { return };
    if fingerprint.len() > 16 || selector.len() > 400 || selector.contains(['{', '}', '<']) {
        return;
    }
    let page = browser.main_frame().map(|f| CefString::from(&f.url()).to_string()).unwrap_or_default();
    let site = echo_network::site::site_of(&page).unwrap_or_default();
    last().lock().insert(browser.identifier(), fingerprint.to_string());
    crate::session::with(|s| echo_library::hidden::add(&s.library, fingerprint, selector, &site));
}

/// La page active a-t-elle des elements masques (pour proposer de les reafficher) ?
pub fn active_has_hidden() -> bool {
    let Some(browser) = crate::session::with(|s| s.tabs.active().and_then(|t| t.browser())).flatten() else {
        return false;
    };
    let Some(fingerprint) = last().lock().get(&browser.identifier()).cloned() else { return false };
    crate::session::with(|s| !echo_library::hidden::selectors(&s.library, &fingerprint).is_empty()).unwrap_or(false)
}

/// Reaffiche tout ce qui etait masque sur le gabarit de la page active.
pub fn unhide_active() {
    let Some(browser) = crate::session::with(|s| s.tabs.active().and_then(|t| t.browser())).flatten() else { return };
    let Some(fingerprint) = last().lock().get(&browser.identifier()).cloned() else { return };
    crate::session::with(|s| echo_library::hidden::clear(&s.library, &fingerprint));
    if let Some(frame) = browser.main_frame() {
        frame.execute_java_script(Some(&CefString::from(css_script(&[]).as_str())), None, 0);
    }
    browser.reload();
}
