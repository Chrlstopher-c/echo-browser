//! Verifie que la bibliotheque tient ses promesses sur une base reelle, en memoire.

use echo_library::{bookmarks, downloads, history, settings, Library};

fn library() -> Library {
    Library::in_memory().expect("base en memoire")
}

#[test]
fn un_favori_sajoute_se_retrouve_et_se_retire() {
    let lib = library();
    assert!(bookmarks::add(&lib, "https://exemple.fr", "Exemple", None));
    assert!(bookmarks::contains(&lib, "https://exemple.fr"));
    assert_eq!(bookmarks::list(&lib).len(), 1);

    assert!(bookmarks::remove(&lib, "https://exemple.fr"));
    assert!(!bookmarks::contains(&lib, "https://exemple.fr"));
}

#[test]
fn ajouter_deux_fois_la_meme_adresse_ne_cree_pas_de_doublon() {
    let lib = library();
    bookmarks::add(&lib, "https://exemple.fr", "Ancien titre", None);
    bookmarks::add(&lib, "https://exemple.fr", "Nouveau titre", None);

    let all = bookmarks::list(&lib);
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].title, "Nouveau titre");
}

#[test]
fn les_favoris_gardent_l_ordre_choisi() {
    let lib = library();
    for name in ["a", "b", "c"] {
        bookmarks::add(&lib, &format!("https://{name}.fr"), name, None);
    }
    assert!(bookmarks::move_to(&lib, "https://c.fr", 0));

    let urls: Vec<String> = bookmarks::list(&lib).into_iter().map(|b| b.url).collect();
    assert_eq!(urls, ["https://c.fr", "https://a.fr", "https://b.fr"]);
}

#[test]
fn l_historique_se_cherche_par_titre_et_par_adresse() {
    let lib = library();
    history::record(&lib, "https://recettes.fr/tarte", "Tarte aux pommes", None);
    history::record(&lib, "https://meteo.fr", "Meteo du jour", None);

    let (trouve, total) = history::search(&lib, "tarte");
    assert_eq!(trouve.len(), 1);
    assert_eq!(trouve[0].url, "https://recettes.fr/tarte");
    assert_eq!(total, 2);

    let (par_titre, _) = history::search(&lib, "Meteo");
    assert_eq!(par_titre.len(), 1);
}

#[test]
fn les_pages_internes_ne_vont_pas_dans_l_historique() {
    let lib = library();
    assert!(!history::record(&lib, "echo://ui/index.html", "Interface", None));
    assert!(!history::record(&lib, "chrome://extensions", "Extensions", None));

    let (entries, total) = history::search(&lib, "");
    assert!(entries.is_empty());
    assert_eq!(total, 0);
}

#[test]
fn l_historique_s_efface_entierement() {
    let lib = library();
    history::record(&lib, "https://exemple.fr", "Exemple", None);
    assert!(history::clear(&lib));
    assert_eq!(history::search(&lib, "").1, 0);
}

#[test]
fn un_reglage_inconnu_est_refuse_avec_la_liste_des_valeurs_admises() {
    let lib = library();
    let err = settings::set(&lib, "reglage.invente", &settings::Value::Flag(true))
        .expect_err("une cle inconnue doit etre refusee");
    assert!(err.contains("reglage.invente"));
    assert!(err.contains("shield.enabled"), "le refus doit citer les cles admises");
}

#[test]
fn un_reglage_connu_se_relit_apres_ecriture() {
    let lib = library();
    settings::set(&lib, "shield.strict", &settings::Value::Flag(true)).expect("ecriture");

    let value = settings::all(&lib)
        .into_iter()
        .find(|(key, _)| key == "shield.strict")
        .map(|(_, value)| value);
    assert_eq!(value, Some(settings::Value::Flag(true)));
}

#[test]
fn les_reglages_absents_prennent_leur_valeur_par_defaut() {
    let lib = library();
    let all = settings::all(&lib);
    assert_eq!(all.len(), settings::defaults().len());
    let engine = all.iter().find(|(key, _)| key == "search.engine").map(|(_, v)| v.clone());
    assert_eq!(engine, Some(settings::Value::Text("google".into())));
}

#[test]
fn un_telechargement_se_suit_puis_s_oublie() {
    let lib = library();
    let mut download = downloads::started(1, "rapport.pdf", "https://exemple.fr/r.pdf", Some(1000));
    assert!(downloads::upsert(&lib, &download));

    download.received = 1000;
    download.state = downloads::State::Complete;
    download.path = Some("/home/essai/rapport.pdf".into());
    assert!(downloads::upsert(&lib, &download));

    let listed = downloads::list(&lib);
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].state, downloads::State::Complete);
    assert_eq!(listed[0].received, 1000);

    assert!(downloads::forget(&lib, 1));
    assert!(downloads::list(&lib).is_empty());
}
