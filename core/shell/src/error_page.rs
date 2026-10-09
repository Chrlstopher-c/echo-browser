//! Responsabilite : les pages d'erreur reseau en francais, dans le style d'Echo. Chromium affiche les siennes (qui
//! parlent de « Chrome ») : on les remplace a la fin de leur chargement. La page de certificat refuse, elle, reste
//! celle de Chromium : elle porte la logique de securite (revenir en lieu sur).

use cef::{CefString, Frame, ImplFrame};
use parking_lot::Mutex;

/// Erreur en attente d'affichage, par navigateur : (code, adresse).
static PENDING: Mutex<Vec<(i32, i32, String)>> = Mutex::new(Vec::new());

/// Note l'echec d'une page principale (hors certificat, hors navigation remplacee).
pub fn note(browser_id: i32, code: i32, url: &str) {
    let mut pending = PENDING.lock();
    pending.retain(|(id, _, _)| *id != browser_id);
    pending.push((browser_id, code, url.to_string()));
}

/// Titre et explication d'une erreur, en langage courant.
fn wording(code: i32) -> (&'static str, &'static str) {
    match code {
        -105 | -137 => ("Adresse introuvable", "Ce site n’existe pas, ou son adresse contient une faute."),
        -106 => ("Pas de connexion Internet", "L’ordinateur n’est relié à aucun réseau : vérifiez le Wi-Fi."),
        -312 => ("Adresse bloquée", "Ce port est réservé à d’autres usages : Echo ne s’y connecte pas."),
        -102 => {
            ("Le site refuse la connexion", "Le serveur est joignable mais n’accepte pas de visite pour l’instant.")
        }
        -7 | -118 => ("Le site ne répond pas", "Il met trop de temps à répondre. Il est peut-être surchargé."),
        -21 => ("Le réseau a changé", "La connexion a changé pendant le chargement. Réessayez."),
        -324 | -100 | -101 => ("Connexion interrompue", "Le site a coupé la connexion avant la fin."),
        _ => ("Page non chargée", "Le site n’a pas pu être affiché."),
    }
}

/// La page d'erreur de Chromium vient de se charger : si une erreur attend pour ce navigateur, on la remplace.
pub fn loaded(browser_id: i32, frame: &Frame, status: i32) {
    // Une vraie page (statut HTTP) s'est chargee : l'erreur notee n'est plus d'actualite.
    if status >= 100 {
        PENDING.lock().retain(|(id, _, _)| *id != browser_id);
        return;
    }
    let found = {
        let mut pending = PENDING.lock();
        pending.iter().position(|(id, _, _)| *id == browser_id).map(|at| pending.remove(at))
    };
    let Some((_, code, url)) = found else { return };
    let (title, detail) = wording(code);
    let host = url.split('/').nth(2).unwrap_or(&url).to_string();
    let data = serde_json::json!({"title": title, "detail": detail, "host": host, "code": code}).to_string();
    let script = format!("({SCRIPT})({data})");
    frame.execute_java_script(Some(&CefString::from(script.as_str())), Some(&CefString::from("echo://erreur")), 0);
}

const SCRIPT: &str = r#"(e) => {
  const dark = matchMedia('(prefers-color-scheme: dark)').matches;
  const c = dark
    ? {bg: '#222326', card: '#26272b', ink: '#ebeced', muted: '#babcbf', hi: 'rgba(255,255,255,.06)',
       lo: 'rgba(0,0,0,.6)', accent: '#55b98d'}
    : {bg: '#e3e4e8', card: '#e3e4e8', ink: '#1d1f23', muted: '#3c4047', hi: 'rgba(255,255,255,.85)',
       lo: 'rgba(70,74,90,.28)', accent: '#2b7656'};
  const relief = (n) => `-${n}px -${n}px ${n * 2.5}px ${c.hi}, ${n}px ${n}px ${n * 3}px ${c.lo}`;
  document.title = e.title;
  document.documentElement.innerHTML = '<head><meta charset="utf-8"></head><body></body>';
  document.body.style.cssText = `margin:0;min-height:100vh;display:grid;place-items:center;background:${c.bg};`
    + `color:${c.ink};font:15px/1.55 system-ui,sans-serif`;
  const make = (tag, text, css) => { const n = document.createElement(tag); n.textContent = text; n.style.cssText = css;
    return n; };
  const card = make('main', '', `max-width:460px;margin:24px;padding:36px 40px;border-radius:22px;`
    + `background:${c.card};box-shadow:${relief(6)}`);
  const button = make('button', 'Réessayer', `border:0;border-radius:999px;padding:9px 18px;font:inherit;`
    + `font-weight:600;color:${c.accent};background:${c.card};box-shadow:${relief(4)};cursor:pointer`);
  button.onclick = () => location.reload();
  card.append(make('h1', e.title, 'margin:0 0 8px;font-size:22px'),
    make('p', e.detail, `margin:0 0 6px;color:${c.muted}`),
    make('p', `${e.host} · code ${e.code}`, `margin:0 0 22px;font-size:12.5px;color:${c.muted}`), button);
  document.body.append(card);
}"#;
