//! Responsabilite : les angles arrondis de la page. Une vue web est un rectangle natif que Chromium
//! ne sait pas rogner ; on rogne donc le document lui-meme (`clip-path` sur la racine) et le fond de
//! la vue prend la teinte de la fenetre, qui apparait dans les coins.

/// Actif par defaut ; `ECHO_ROUND=0` le coupe, pour comparer.
pub fn enabled() -> bool {
    std::env::var("ECHO_ROUND").map_or(true, |v| v != "0")
}

/// Rogne la racine, et redonne le fond blanc par defaut aux pages qui n'en declarent pas : la teinte de la
/// vue, elle, ne doit jamais remplacer ce blanc.
pub const SCRIPT: &str = "(()=>{const r=document.documentElement;\
r.style.setProperty('clip-path','inset(0 round 12px)','important');\
addEventListener('DOMContentLoaded',()=>{const t=c=>c==='rgba(0, 0, 0, 0)'||c==='transparent';\
const h=getComputedStyle(r).backgroundColor;\
const b=document.body?getComputedStyle(document.body).backgroundColor:'rgba(0, 0, 0, 0)';\
if(t(h)&&t(b))r.style.setProperty('background-color','#fff');});})()";
