//! Compte Echo : chiffrement de bout en bout (`crypto`), appels au service (`api`), fusion a trois voies (`merge`),
//! passe de synchronisation (`sync`) et memoire locale du compte (`store`).

pub mod api;
pub mod crypto;
pub mod merge;
pub mod store;
pub mod sync;
