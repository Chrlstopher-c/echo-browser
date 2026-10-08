//! Responsabilite : reprise exacte d'une page — ce que l'utilisateur a saisi (jamais les mots de passe ni les donnees de
//! carte) et la position des videos et sons. La page l'emet (console, `echo:etat:`) ; l'etat suit l'onglet, la session
//! sauvegardee, et il est rejoue au chargement apres un reveil ou une relance (sans ecraser une saisie deja la).

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use parking_lot::Mutex;

use echo_contract::TabId;

pub const MARKER: &str = "echo:etat:";
/// Au-dela, l'etat n'est pas garde (une page qui saisit des volumes n'est pas un formulaire).
const MAX_LEN: usize = 16 * 1024;

/// Note l'etat en direct : champs (cle stable → valeur), media principal (temps en secondes).
pub const WATCHER: &str = r#"(()=>{if(window.__echoEtat)return;window.__echoEtat=1;
const SKIP=/pass|card|carte|cvv|cvc|iban|otp|secret|token|ssn/i;
const ok=e=>{if(e.isContentEditable)return false;const t=(e.type||'').toLowerCase();
if(e.tagName==='TEXTAREA'||e.tagName==='SELECT')return !SKIP.test(e.name+e.id+(e.autocomplete||''));
return ['text','search','email','url','tel','number',''].includes(t)&&!SKIP.test(e.name+e.id+(e.autocomplete||''))};
const key=(e,i)=>e.id?'#'+e.id:e.name?'@'+e.name+':'+[...document.getElementsByName(e.name)].indexOf(e):'%'+i;
const fields=()=>[...document.querySelectorAll('input,textarea,select')];
const media=()=>[...document.querySelectorAll('video,audio')].find(m=>m.duration>20);
let t=0;const send=()=>{clearTimeout(t);t=setTimeout(()=>{const f={};fields().forEach((e,i)=>{if(ok(e)&&e.value)
f[key(e,i)]=e.value.slice(0,4000)});const m=media();const s={f};if(m&&m.currentTime>3)s.m=Math.floor(m.currentTime);
console.debug('echo:etat:'+JSON.stringify(s))},600)};
addEventListener('input',send,true);addEventListener('change',send,true);
let last=0;addEventListener('timeupdate',()=>{const n=Date.now();if(n-last>5000){last=n;send()}},true)})()"#;

static STATES: OnceLock<Mutex<HashMap<TabId, String>>> = OnceLock::new();
static ARMED: OnceLock<Mutex<HashSet<TabId>>> = OnceLock::new();

fn states() -> &'static Mutex<HashMap<TabId, String>> {
    STATES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn armed() -> &'static Mutex<HashSet<TabId>> {
    ARMED.get_or_init(|| Mutex::new(HashSet::new()))
}

fn tab_of(browser: i32) -> Option<TabId> {
    crate::session::with(|s| s.tabs.by_browser(browser).map(|t| t.id)).flatten()
}

/// L'etat emis par la page (fil de l'interface).
pub fn record(browser: i32, json: &str) {
    let Some(tab) = tab_of(browser) else { return };
    if json.len() > MAX_LEN || serde_json::from_str::<serde_json::Value>(json).is_err() {
        return;
    }
    let changed = states().lock().insert(tab, json.to_string()).as_deref() != Some(json);
    if changed {
        crate::persist::schedule();
    }
}

pub fn get(tab: TabId) -> Option<String> {
    states().lock().get(&tab).cloned()
}

/// Un onglet restitue (relance) avec son etat : rejoue au prochain chargement.
pub fn adopt(tab: TabId, state: Option<String>) {
    if let Some(state) = state {
        states().lock().insert(tab, state);
        armed().lock().insert(tab);
    }
}

/// Onglet reveille : son etat sera rejoue au chargement.
pub fn arm(tab: TabId) {
    if states().lock().contains_key(&tab) {
        armed().lock().insert(tab);
    }
}

/// Nouvelle page dans l'onglet (pas un reveil) : l'ancien etat ne la concerne plus.
pub fn navigated(browser: i32) {
    if let Some(tab) = tab_of(browser)
        && !armed().lock().contains(&tab)
    {
        states().lock().remove(&tab);
    }
}

/// Script a jouer a la fin du chargement de la page principale, si un etat attend.
pub fn restore_script(browser: i32) -> Option<String> {
    let tab = tab_of(browser)?;
    if !armed().lock().remove(&tab) {
        return None;
    }
    let state = get(tab)?;
    Some(format!(
        r#"(()=>{{const s={state};const fields=[...document.querySelectorAll('input,textarea,select')];
const find=(k)=>{{if(k[0]==='#')return document.getElementById(k.slice(1));
if(k[0]==='@'){{const [n,i]=k.slice(1).split(/:(?=\d+$)/);return document.getElementsByName(n)[+i]}}
return fields[+k.slice(1)]}};
const go=()=>{{for(const [k,v] of Object.entries(s.f||{{}})){{const e=find(k);if(e&&!e.value){{e.value=v;
e.dispatchEvent(new Event('input',{{bubbles:true}}))}}}}
const m=[...document.querySelectorAll('video,audio')].find(x=>x.duration>20||x.readyState<1);
if(m&&s.m&&m.currentTime<1){{const set=()=>{{if(m.currentTime<1)m.currentTime=s.m}};
m.readyState>0?set():m.addEventListener('loadedmetadata',set,{{once:true}})}}}};
go();setTimeout(go,400);setTimeout(go,1500);setTimeout(go,4000)}})()"#
    ))
}
