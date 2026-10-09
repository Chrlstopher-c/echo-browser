//! Verifie que la bibliotheque tient ses promesses sur une base reelle, en memoire.

use echo_library::{bookmarks, downloads, history, settings, Library};

fn library() -> Library {
    Library::in_memory().expect("base en memoire")
}

#[test]
fn un_favori_sajoute_se_retrouve_et_se_retire() {
    let lib = library();
    assert!(bookmarks::add(&lib, "https://exemple.fr", "Exemple", None, "graphite"));
    assert!(bookmarks::contains(&lib, "https://exemple.fr"));
    assert_eq!(bookmarks::list(&lib).len(), 1);

    assert!(bookmarks::remove(&lib, "https://exemple.fr"));
    assert!(!bookmarks::contains(&lib, "https://exemple.fr"));
}

#[test]
fn ajouter_deux_fois_la_meme_adresse_ne_cree_pas_de_doublon() {
    let lib = library();
    bookmarks::add(&lib, "https://exemple.fr", "Ancien titre", None, "graphite");
    bookmarks::add(&lib, "https://exemple.fr", "Nouveau titre", None, "graphite");

    let all = bookmarks::list(&lib);
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].title, "Nouveau titre");
}

#[test]
fn les_favoris_gardent_l_ordre_choisi() {
    let lib = library();
    for name in ["a", "b", "c"] {
        bookmarks::add(&lib, &format!("https://{name}.fr"), name, None, "graphite");
    }
    assert!(bookmarks::move_to(&lib, "https://c.fr", 0, "graphite"));

    let urls: Vec<String> = bookmarks::list(&lib).into_iter().map(|b| b.url).collect();
    assert_eq!(urls, ["https://c.fr", "https://a.fr", "https://b.fr"]);
}

#[test]
fn l_historique_se_cherche_par_titre_et_par_adresse() {
    let lib = library();
    history::record(&lib, "https://recettes.fr/tarte", "Tarte aux pommes", None, "graphite");
    history::record(&lib, "https://meteo.fr", "Meteo du jour", None, "graphite");

    let (trouve, total) = history::search(&lib, "tarte", "graphite");
    assert_eq!(trouve.len(), 1);
    assert_eq!(trouve[0].url, "https://recettes.fr/tarte");
    assert_eq!(total, 2);

    let (par_titre, _) = history::search(&lib, "Meteo", "graphite");
    assert_eq!(par_titre.len(), 1);
}

#[test]
fn les_pages_internes_ne_vont_pas_dans_l_historique() {
    let lib = library();
    assert!(!history::record(&lib, "echo://ui/index.html", "Interface", None, "graphite"));
    assert!(!history::record(&lib, "chrome://extensions", "Extensions", None, "graphite"));

    let (entries, total) = history::search(&lib, "", "graphite");
    assert!(entries.is_empty());
    assert_eq!(total, 0);
}

#[test]
fn l_historique_s_efface_entierement() {
    let lib = library();
    history::record(&lib, "https://exemple.fr", "Exemple", None, "graphite");
    assert!(history::clear(&lib));
    assert_eq!(history::search(&lib, "", "graphite").1, 0);
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

#[test]
fn une_visite_effacee_ne_revient_pas_par_une_autre_machine() {
    let lib = library();
    let t = echo_library::now();
    history::import(&lib, "https://a.fr", "A", t - 100, "graphite");
    assert!(history::remove(&lib, "https://a.fr", t - 100));
    assert!(history::import(&lib, "https://a.fr", "A", t - 100, "graphite"));
    assert_eq!(history::search(&lib, "", "graphite").1, 0);
    history::import(&lib, "https://a.fr", "A", t, "graphite");
    assert_eq!(history::latest(&lib, 10)[0].visited_at, t);
    assert!(history::forgotten(&lib).iter().any(|(url, at)| url == "https://a.fr" && *at == t - 100));
}

#[test]
fn tout_effacer_est_transmis_et_bloque_les_visites_anterieures() {
    let lib = library();
    history::record(&lib, "https://b.fr", "B", None, "graphite");
    assert!(history::clear(&lib));
    assert!(history::forgotten(&lib).iter().any(|(url, _)| url == history::ALL));
    history::import(&lib, "https://c.fr", "C", 50, "graphite");
    assert_eq!(history::search(&lib, "", "graphite").1, 0);
}

#[test]
fn journal_d_acces_par_site_sans_doublon_de_tiers() {
    use echo_library::journal;
    let lib = library();
    assert!(journal::record(&lib, "example.com", "tiers", "cdn.tracker.net", true));
    assert!(!journal::record(&lib, "example.com", "tiers", "cdn.tracker.net", true));
    assert!(journal::record(&lib, "example.com", "permission", "position : refusée", false));
    let entries = journal::list(&lib, "example.com", 10);
    assert_eq!(entries.len(), 2);
    assert!(journal::list(&lib, "autre.fr", 10).is_empty());
    for i in 0..250 {
        journal::record(&lib, "plein.fr", "tiers", &format!("h{i}"), true);
    }
    assert_eq!(journal::list(&lib, "plein.fr", 500).len(), 200);
}

#[test]
fn une_suite_qui_revient_est_proposee_une_fois_puis_devient_routine() {
    use echo_library::routines;
    let lib = library();
    let visit = |s: &str| (s.to_string(), format!("https://{s}/"));
    let suite = vec![visit("github.com"), visit("linear.app"), visit("slack.com")];
    let mut proposal = None;
    for _ in 0..3 {
        for end in 1..=suite.len() {
            if let Some(p) = routines::observe(&lib, &suite[..end], 0) {
                proposal = Some(p);
            }
        }
    }
    let proposal = proposal.expect("suite non proposee apres 3 passages");
    assert_eq!(proposal.sites, ["github.com", "linear.app", "slack.com"]);
    assert!(routines::observe(&lib, &suite, 0).is_none(), "proposee deux fois");
    let id = routines::adopt(&lib, &proposal.fingerprint, "Matin").expect("routine non creee");
    let all = routines::list(&lib);
    assert_eq!((all[0].id, all[0].name.as_str(), all[0].urls.len()), (id, "Matin", 3));
    assert!(routines::remove(&lib, id));
    assert!(routines::list(&lib).is_empty());
}

#[test]
fn une_suite_n_est_comptee_qu_une_fois_par_fenetre() {
    use echo_library::routines;
    let lib = library();
    let suite = vec![("a.fr".to_string(), "https://a.fr/".to_string()), ("b.fr".to_string(), "https://b.fr/".to_string())];
    for _ in 0..10 {
        assert!(routines::observe(&lib, &suite, 600).is_none());
    }
}

#[test]
fn un_element_masque_l_est_pour_tout_le_gabarit() {
    use echo_library::hidden;
    let lib = library();
    assert!(hidden::add(&lib, "g1", "div.banniere", "journal.fr"));
    hidden::add(&lib, "g1", "div.banniere", "journal.fr");
    assert_eq!(hidden::selectors(&lib, "g1", "journal.fr"), ["div.banniere"]);
    assert!(hidden::selectors(&lib, "g1", "autre.fr").is_empty(), "un autre site ne voit pas les regles");
    assert!(hidden::selectors(&lib, "g2", "journal.fr").is_empty());
    assert!(hidden::clear(&lib, "g1", "journal.fr"));
    assert!(hidden::selectors(&lib, "g1", "journal.fr").is_empty());
}

#[test]
fn signaux_en_attente_par_jour() {
    use echo_library::signals;
    let lib = library();
    let c = |k: &str, key: &str, n: u32| (k.to_string(), key.to_string(), n);
    signals::add(&lib, "2026-10-07", &[c("sites", "a.fr", 2), c("bloques", "pub.net", 5)]);
    signals::add(&lib, "2026-10-07", &[c("sites", "a.fr", 1)]);
    signals::add(&lib, "2026-10-08", &[c("sites", "b.fr", 1)]);
    assert_eq!(signals::days(&lib), ["2026-10-07", "2026-10-08"]);
    assert_eq!(signals::top(&lib, "2026-10-07", "sites", 10), [("a.fr".to_string(), 3)]);
    signals::forget_day(&lib, "2026-10-07");
    assert_eq!(signals::days(&lib), ["2026-10-08"]);
}

#[test]
fn page_surveillee_lignes_ajoutees_et_retirees() {
    use echo_library::watched;
    let lib = library();
    let url = "https://exemple.fr/prix";
    assert!(watched::visited(&lib, url, "a").is_none(), "non surveillee : rien");
    watched::watch(&lib, url);
    assert!(watched::is_watched(&lib, url));
    assert!(watched::visited(&lib, url, "Titre\nPrix : 20 €\nStock : 3").is_none(), "premiere visite : reference");
    assert!(watched::visited(&lib, url, "Titre\nPrix : 20 €\nStock : 3").is_none(), "identique");
    let change = watched::visited(&lib, url, "Titre\nPrix : 18 €\nStock : 3").expect("changement");
    assert_eq!(change.added, ["Prix : 18 €"]);
    assert_eq!(change.removed, ["Prix : 20 €"]);
    watched::unwatch(&lib, url);
    assert!(!watched::is_watched(&lib, url));
}

#[test]
fn historique_et_favoris_propres_a_chaque_profil() {
    let lib = Library::in_memory().unwrap();
    history::record(&lib, "https://perso.fr", "Perso", None, "graphite");
    history::record(&lib, "https://travail.fr", "Travail", None, "sable");
    bookmarks::add(&lib, "https://banque.fr", "Banque", None, "graphite");
    bookmarks::add(&lib, "https://outil.fr", "Outil", None, "sable");
    let urls = |space: &str| history::search(&lib, "", space).0.into_iter().map(|e| e.url).collect::<Vec<_>>();
    assert_eq!(urls("graphite"), vec!["https://perso.fr"]);
    assert_eq!(urls("sable"), vec!["https://travail.fr"]);
    assert_eq!(history::search(&lib, "trav", "graphite").0.len(), 0);
    assert_eq!(bookmarks::list_in(&lib, "sable").len(), 1);
    assert_eq!(bookmarks::list(&lib).len(), 2, "la synchronisation voit tous les profils");
    assert_eq!(history::latest(&lib, 10).len(), 2);
}
