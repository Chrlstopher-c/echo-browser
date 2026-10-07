//! Responsabilite : reperer les onglets que les extensions ouvrent dans la fenetre d'ancrage et les
//! reprendre comme onglets d'Echo. CEF ne cree pas de navigateur pour eux : on les voit par le protocole
//! de debogage local (cible « page » qui a une fenetre Chrome — les onglets d'Echo n'en ont pas).

use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Read;
use std::net::{Ipv4Addr, TcpStream};
use tracing::{debug, warn};
use tungstenite::{connect, Message};

/// Lance l'ecoute (fil a part). Sans port de debogage joignable, ne fait rien.
pub fn start() {
    std::thread::spawn(|| {
        // Le serveur de debogage met quelques secondes a accepter les connexions : on reessaie.
        for attempt in 1..=20 {
            std::thread::sleep(std::time::Duration::from_secs(2));
            match watch() {
                Ok(()) => return,
                Err(error) => debug!(attempt, %error, "ecoute des onglets d'extension : nouvel essai"),
            }
        }
        warn!("ecoute des onglets d'extension abandonnee");
    });
}

fn browser_endpoint() -> Option<String> {
    let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, crate::devtools::port())).ok()?;
    // L'adresse renvoyee est construite sur l'en-tete Host : sans le port, elle en est privee.
    let request = format!("GET /json/version HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n", crate::devtools::port());
    std::io::Write::write_all(&mut stream, request.as_bytes()).ok()?;
    stream.set_read_timeout(Some(std::time::Duration::from_secs(2))).ok()?;
    let mut raw = Vec::new();
    let mut chunk = [0u8; 4096];
    while let Ok(n) = stream.read(&mut chunk) {
        if n == 0 { break; }
        raw.extend_from_slice(&chunk[..n]);
        if raw.ends_with(b"}\n") || raw.ends_with(b"}") { break; }
    }
    let text = String::from_utf8_lossy(&raw);
    let body: Value = serde_json::from_str(text.split_once("\r\n\r\n")?.1).ok()?;
    body["webSocketDebuggerUrl"].as_str().map(str::to_string)
}

fn watch() -> anyhow::Result<()> {
    let url = browser_endpoint().ok_or_else(|| anyhow::anyhow!("point de debogage introuvable"))?;
    let (mut socket, _) = connect(url.as_str())?;
    socket.send(Message::text(json!({"id": 1, "method": "Target.setDiscoverTargets", "params": {"discover": true}}).to_string()))?;
    let mut next_id = 10;
    // Requetes getWindowForTarget en cours ; cibles deja classees ; onglets d'ancrage en attente d'adresse.
    let mut asking: HashMap<i64, String> = HashMap::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut pending: HashMap<String, String> = HashMap::new();
    let mut last_url: HashMap<String, String> = HashMap::new();
    loop {
        let Message::Text(text) = socket.read()? else { continue };
        let msg: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
        if matches!(msg["method"].as_str(), Some("Target.targetCreated" | "Target.targetInfoChanged")) {
            let info = &msg["params"]["targetInfo"];
            if info["type"] != "page" {
                continue;
            }
            let target = info["targetId"].as_str().unwrap_or_default().to_string();
            let page = info["url"].as_str().unwrap_or_default().to_string();
            last_url.insert(target.clone(), page.clone());
            if pending.contains_key(&target) {
                pending.insert(target.clone(), page);
                take_if_ready(&mut socket, &mut pending, &target, &mut next_id)?;
            } else if seen.insert(target.clone()) {
                // Premiere fois qu'on voit cette page : est-elle dans une fenetre Chrome (l'ancrage) ?
                asking.insert(next_id, target.clone());
                let ask = json!({"id": next_id, "method": "Browser.getWindowForTarget", "params": {"targetId": target}});
                socket.send(Message::text(ask.to_string()))?;
                next_id += 1;
            }
            continue;
        }
        let Some(id) = msg["id"].as_i64() else { continue };
        let Some(target) = asking.remove(&id) else { continue };
        if msg.get("result").is_some() {
            pending.insert(target.clone(), last_url.get(&target).cloned().unwrap_or_default());
            take_if_ready(&mut socket, &mut pending, &target, &mut next_id)?;
        }
    }
}

/// L'onglet d'ancrage a une adresse reelle : on le ferme et on le rouvre dans Echo.
fn take_if_ready(
    socket: &mut tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<TcpStream>>,
    pending: &mut HashMap<String, String>,
    target: &str,
    next_id: &mut i64,
) -> anyhow::Result<()> {
    let page = pending.get(target).cloned().unwrap_or_default();
    if page.is_empty() || page == "about:blank" {
        return Ok(());
    }
    pending.remove(target);
    socket.send(Message::text(json!({"id": *next_id, "method": "Target.closeTarget", "params": {"targetId": target}}).to_string()))?;
    *next_id += 1;
    debug!(%page, "onglet d'extension repris dans Echo");
    crate::containers::later(move || {
        crate::bridge::open_tab_like_active(&page);
        crate::bridge::publish_tabs();
    });
    Ok(())
}
