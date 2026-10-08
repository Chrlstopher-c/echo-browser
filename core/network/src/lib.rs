//! Le reseau des onglets, vu par l'utilisateur : ce que chaque page charge, d'ou, combien, et ce que le bouclier ou
//! ses regles bloquent. Sans CEF : le navigateur appelle `TabLog` depuis ses gestionnaires de requetes.

pub mod log;
pub mod rules;
pub mod site;

pub use log::{DomainStat, RequestEntry, TabLog};
pub use rules::Rules;
