//! Responsabilite : installer une extension depuis le Chrome Web Store sans quitter Echo. La boutique
//! reconnait qu'elle ne parle pas au vrai Chrome et renvoie vers lui ; on pose donc sur ses fiches un
//! bouton « Ajouter à Echo » qui confie l'installation au gestionnaire d'Echo.

/// Hote du catalogue.
pub const HOST: &str = "chromewebstore.google.com";

/// Message console du bouton : `echo:install:<adresse de la fiche>`.
pub const INSTALL_MARKER: &str = "echo:install:";

/// Script pose sur le catalogue (site a page unique : le bouton suit la navigation interne).
pub const BUTTON_SCRIPT: &str = r#"(()=>{if(window.__echoStore)return;window.__echoStore=1;
const id='__echo_add';const sync=()=>{const fiche=/\/detail\//.test(location.pathname);let b=document.getElementById(id);
if(!fiche){if(b)b.remove();return}if(b)return;b=document.createElement('button');b.id=id;b.textContent='Ajouter à Echo';
b.style.cssText='position:fixed;right:28px;bottom:28px;z-index:2147483647;padding:14px 22px;border:0;border-radius:999px;'+
'font:600 15px system-ui,sans-serif;color:#fff;background:#3d7a5f;box-shadow:0 8px 24px rgba(0,0,0,.35);cursor:pointer';
b.onclick=()=>{console.debug('echo:install:'+location.href);b.textContent='Ajout en cours…';b.disabled=true};
document.documentElement.appendChild(b)};sync();setInterval(sync,800)})()"#;

/// Vrai si l'adresse est une page du catalogue.
pub fn is_store(url: &str) -> bool {
    url.split("://").nth(1).and_then(|rest| rest.split('/').next()) == Some(HOST)
}
