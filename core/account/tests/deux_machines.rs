//! Deux machines, un compte : ce que l'une ajoute ou change apparait sur l'autre, rien n'est perdu quand les deux
//! modifient en meme temps. Contre un service reel ou local : `COMPTE_URL=http://127.0.0.1:8788 cargo test -p
//! echo-account -- --ignored`.

use std::collections::BTreeMap;

use echo_account::api::Client;
use echo_account::store::Stored;
use echo_account::sync;
use serde_json::{json, Value};

fn locals(reglages: Value, favoris: Value, extensions: Value) -> BTreeMap<String, Value> {
    BTreeMap::from([
        ("reglages".to_string(), reglages),
        ("favoris".to_string(), favoris),
        ("extensions".to_string(), extensions),
    ])
}

#[test]
#[ignore = "demande un service de compte (COMPTE_URL)"]
fn ce_qu_une_machine_change_arrive_sur_l_autre() {
    let url = std::env::var("COMPTE_URL").expect("COMPTE_URL");
    let client = Client::new(&url);
    let email = format!("essai-machines-{}@exemple.org", std::process::id());
    let session_a = client.register(&email, "un mot de passe solide").unwrap();
    let session_b = client.login(&email, "un mot de passe solide").unwrap();
    assert!(client.login(&email, "mauvais").is_err());
    let (mut a, mut b) = (Stored::from_session(&session_a), Stored::from_session(&session_b));

    let favori = json!({"url": "https://exemple.org/", "title": "Exemple"});
    let local_a = locals(json!({"search.engine": "google"}), json!([favori]), json!({"graphite": ["p"]}));
    let out = sync::run(&client, &session_a, &mut a, &local_a).unwrap();
    assert!(out.writes.is_empty() && out.postponed.is_empty());

    let local_b = locals(json!({"search.engine": "duckduckgo"}), json!([]), json!({}));
    let out = sync::run(&client, &session_b, &mut b, &local_b).unwrap();
    assert_eq!(out.writes["favoris"], json!([favori]), "le favori de A arrive sur B");
    assert_eq!(out.writes["extensions"], json!({"graphite": ["p"]}));
    // Premiere synchro de B : sa valeur locale l'emporte sur un reglage qu'il a deja.
    assert!(!out.writes.contains_key("reglages"));

    let out = sync::run(&client, &session_a, &mut a, &local_a).unwrap();
    assert_eq!(out.writes["reglages"], json!({"search.engine": "duckduckgo"}), "le reglage de B arrive sur A");
    client.logout(&session_a.token).unwrap();
}
