//! Rapport d'aptitude du bouclier : telecharge les listes, construit le moteur,
//! puis mesure ce qu'il bloque, ce qu'il laisse passer, et ce qu'il injecte.
//!
//! Lancer avec : cargo run -p echo-shield --example rapport --release

use echo_shield::{verdict::Verdict, Shield};
use std::time::Instant;

/// (url demandee, page qui la demande, type, doit-etre-bloquee)
const REQUESTS: &[(&str, &str, &str, bool)] = &[
    ("https://securepubads.g.doubleclick.net/tag/js/gpt.js", "https://www.lemonde.fr/", "script", true),
    ("https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js", "https://www.20minutes.fr/", "script", true),
    ("https://www.google-analytics.com/analytics.js", "https://www.lemonde.fr/", "script", true),
    ("https://connect.facebook.net/en_US/fbevents.js", "https://www.fnac.com/", "script", true),
    ("https://static.criteo.net/js/ld/publishertag.js", "https://www.leboncoin.fr/", "script", true),
    ("https://cdn.taboola.com/libtrc/tb_loader.js", "https://www.01net.com/", "script", true),
    ("https://sb.scorecardresearch.com/beacon.js", "https://www.tf1.fr/", "script", true),
    ("https://www.googletagmanager.com/gtm.js?id=GTM-XXXX", "https://www.cdiscount.com/", "script", true),
    ("https://prebid.adnxs.com/pbs/v1/openrtb2/auction", "https://www.lequipe.fr/", "xhr", true),
    ("https://ads.pubmatic.com/AdServer/js/pwt/x/y.js", "https://www.futura-sciences.com/", "script", true),
    ("https://analytics.tiktok.com/i18n/pixel/events.js", "https://www.zalando.fr/", "script", true),
    ("https://static.hotjar.com/c/hotjar-123.js", "https://www.boursorama.com/", "script", true),
    ("https://www.lemonde.fr/assets/main.js", "https://www.lemonde.fr/", "script", false),
    ("https://fonts.googleapis.com/css2?family=Inter", "https://www.lemonde.fr/", "stylesheet", false),
    ("https://www.youtube.com/s/player/base.js", "https://www.youtube.com/", "script", false),
    ("https://github.com/assets/app.js", "https://github.com/", "script", false),
    ("https://api.stripe.com/v1/tokens", "https://boutique.example.com/", "xhr", false),
    // Connexion « Continuer avec Google » (GSI) : le bouton et le relais de la popup sont des cadres.
    ("https://accounts.google.com/gsi/client", "https://www.linkedin.com/login/fr", "script", false),
    ("https://accounts.google.com/gsi/button?type=standard&client_id=x", "https://www.linkedin.com/login/fr", "sub_frame", false),
    ("https://accounts.google.com/gsi/iframe/select?client_id=x&ux_mode=popup", "https://www.linkedin.com/login/fr", "sub_frame", false),
];

const PAGES: &[&str] = &[
    "https://www.youtube.com/watch?v=abc",
    "https://www.lemonde.fr/",
    "https://www.leboncoin.fr/",
    "https://www.01net.com/",
    "https://www.jeuxvideo.com/",
    "https://www.ladepeche.fr/",
    "https://www.lejdd.fr/",
    "https://www.geo.fr/",
    "https://mashable.com/",
];

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let shield = Shield::new("data");

    println!("\n== LISTES ==");
    let refreshed = shield.refresh_lists(false)?;
    println!("listes telechargees ou rafraichies : {refreshed}");

    let started = Instant::now();
    shield.load()?;
    println!("moteur pret en {:?}", started.elapsed());

    println!("\n== BLOCAGE RESEAU ==");
    let mut correct = 0usize;
    for (url, source, kind, expected) in REQUESTS {
        let verdict = shield.decide(1, url, source, kind, "GET");
        let blocked = verdict != Verdict::Allow;
        if blocked == *expected {
            correct += 1;
        }
        let host = url.split('/').nth(2).unwrap_or(url);
        let state = match &verdict {
            Verdict::Allow => "passe   ",
            Verdict::Block { .. } => "BLOQUE  ",
            Verdict::Redirect { .. } => "REMPLACE",
        };
        println!("  {state} {:5}  {host}", if blocked == *expected { "ok" } else { "ECHEC" });
    }
    println!("  -> {correct}/{} corrects", REQUESTS.len());

    println!("\n== TRAITEMENT DES PAGES ==");
    for page in PAGES {
        let treatment = shield.treat_page(page);
        let host = page.split('/').nth(2).unwrap_or(page);
        println!(
            "  {host:26} masquage:{:4}  procedural:{:3}  scriptlets:{:6} o",
            treatment.hide_selectors.len(),
            treatment.procedural_actions.len(),
            treatment.injected_script.len()
        );
    }

    println!("\n== DEBIT ==");
    let requests: Vec<_> = (0..300).map(|i| REQUESTS[i % REQUESTS.len()]).collect();
    for (u, s, k, _) in &requests {
        let _ = shield.decide(1, u, s, k, "GET");
    }
    let started = Instant::now();
    for (u, s, k, _) in &requests {
        let _ = shield.decide(1, u, s, k, "GET");
    }
    let elapsed = started.elapsed();
    println!("  300 requetes en {elapsed:?} ({:?} par requete)", elapsed / 300);
    println!("  compteur onglet 1 : {:?}", shield.tally(1));
    Ok(())
}
