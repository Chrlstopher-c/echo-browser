//! Sonde de presence — le seul port que le navigateur ouvre.
//!
//! Le centre de controle conclut « en marche » sur la reponse d'un port, jamais sur un
//! identifiant de processus : sans port, la tuile du navigateur reste eternellement grise.
//! La sonde sert donc une page de presence, et rien d'autre — elle ne lit pas la requete,
//! n'expose aucune commande, et ne porte aucun en-tete d'origine croisee : une page web
//! peut la solliciter, jamais en lire la reponse.

use std::io::Write;
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

/// Port declare dans `.echoforge.yml`. Le changer ici veut dire le changer la-bas.
pub const PORT: u16 = 4330;

/// Au-dela, l'ecoute est cassee pour de bon : mieux vaut rendre le fil que tourner a vide.
const MAX_ERREURS: u32 = 16;

pub fn ouvrir() {
    let listener = match TcpListener::bind((Ipv4Addr::LOCALHOST, PORT)) {
        Ok(listener) => listener,
        Err(erreur) => {
            warn!(port = PORT, %erreur, "sonde de presence non ouverte");
            return;
        }
    };
    info!(port = PORT, "sonde de presence ouverte");
    std::thread::spawn(move || servir(listener, depart()));
}

fn depart() -> String {
    let secondes = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let jour = secondes % 86_400;
    format!("{:02}:{:02}:{:02} UTC", jour / 3600, (jour % 3600) / 60, jour % 60)
}

fn servir(listener: TcpListener, depuis: String) {
    let page = page(&depuis);
    let mut erreurs = 0_u32;
    for flux in listener.incoming() {
        match flux {
            Ok(flux) => {
                erreurs = 0;
                repondre(flux, &page);
            }
            Err(erreur) => {
                erreurs += 1;
                warn!(%erreur, erreurs, "connexion refusee sur la sonde");
                if erreurs >= MAX_ERREURS {
                    warn!("sonde de presence abandonnee apres {MAX_ERREURS} echecs");
                    return;
                }
            }
        }
    }
}

fn repondre(mut flux: TcpStream, page: &str) {
    let entete = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\
         Content-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\n\r\n",
        page.len()
    );
    // Le centre de controle sonde chaque seconde et raccroche sans lire : le tuyau rompu
    // est le cas normal, pas une panne. L'avertir remplissait le journal d'une ligne par
    // seconde.
    if let Err(erreur) = flux.write_all(entete.as_bytes()).and_then(|_| flux.write_all(page.as_bytes())) {
        if erreur.kind() != std::io::ErrorKind::BrokenPipe {
            warn!(%erreur, "reponse de presence non ecrite");
        }
    }
}

fn page(depuis: &str) -> String {
    format!(
        "<!doctype html><meta charset=utf-8><title>Navigateur</title>\
<style>html{{height:100%}}body{{margin:0;height:100%;display:grid;place-content:center;\
justify-items:center;gap:.75rem;background:#0d0e10;color:#e8e6e3;\
font:400 14px/1.5 ui-sans-serif,system-ui,sans-serif}}\
.pastille{{width:.5rem;height:.5rem;border-radius:50%;background:#4ade80;\
box-shadow:0 0 0 .35rem rgba(74,222,128,.12)}}\
h1{{margin:0;font-size:1rem;font-weight:500;letter-spacing:.01em}}\
p{{margin:0;color:#8b8a87;font-size:.8125rem;font-variant-numeric:tabular-nums}}</style>\
<div class=pastille></div><h1>Navigateur en marche</h1><p>depuis {depuis} — sonde {PORT}</p>"
    )
}
