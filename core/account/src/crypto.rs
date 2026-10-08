//! Responsabilite : le chiffrement de bout en bout. Le mot de passe ne quitte jamais la machine : PBKDF2 en tire une
//! cle maitresse, HKDF la separe en cle d'acces (envoyee au service, qui n'en garde qu'une empreinte) et cle de
//! chiffrement (jamais envoyee). Les donnees partent en AES-256-GCM, liees a leur type (on ne peut pas echanger les
//! favoris contre les reglages sur le serveur sans que l'ouverture echoue).

use std::num::NonZeroU32;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM, NONCE_LEN};
use ring::rand::{SecureRandom, SystemRandom};
use ring::{hkdf, pbkdf2};

/// Cout de derivation : ~0,5 s sur une machine modeste, a chaque connexion seulement.
pub const ITERATIONS: u32 = 600_000;
pub const SALT_LEN: usize = 16;

pub struct Keys {
    pub access: [u8; 32],
    pub encryption: [u8; 32],
}

pub fn derive(password: &str, salt: &[u8], iterations: u32) -> Keys {
    let mut master = [0u8; 32];
    let rounds = NonZeroU32::new(iterations.max(1)).unwrap_or(NonZeroU32::MIN);
    pbkdf2::derive(pbkdf2::PBKDF2_HMAC_SHA256, rounds, salt, password.as_bytes(), &mut master);
    let prk = hkdf::Salt::new(hkdf::HKDF_SHA256, b"echo-compte").extract(&master);
    let expand = |info: &[u8]| {
        let mut out = [0u8; 32];
        let infos = [info];
        if let Ok(okm) = prk.expand(&infos, hkdf::HKDF_SHA256) {
            let _ = okm.fill(&mut out);
        }
        out
    };
    Keys { access: expand(b"acces"), encryption: expand(b"chiffrement") }
}

pub fn random_bytes<const N: usize>() -> anyhow::Result<[u8; N]> {
    let mut out = [0u8; N];
    SystemRandom::new().fill(&mut out).map_err(|_| anyhow::anyhow!("aleatoire indisponible"))?;
    Ok(out)
}

fn key(encryption: &[u8; 32]) -> anyhow::Result<LessSafeKey> {
    let unbound = UnboundKey::new(&AES_256_GCM, encryption).map_err(|_| anyhow::anyhow!("cle invalide"))?;
    Ok(LessSafeKey::new(unbound))
}

/// Chiffre `plain` pour le type `kind` : base64(nonce || chiffre || etiquette).
pub fn seal(encryption: &[u8; 32], kind: &str, plain: &[u8]) -> anyhow::Result<String> {
    let nonce_bytes = random_bytes::<NONCE_LEN>()?;
    let mut buffer = plain.to_vec();
    key(encryption)?
        .seal_in_place_append_tag(Nonce::assume_unique_for_key(nonce_bytes), Aad::from(kind.as_bytes()), &mut buffer)
        .map_err(|_| anyhow::anyhow!("chiffrement impossible"))?;
    let mut out = nonce_bytes.to_vec();
    out.extend_from_slice(&buffer);
    Ok(STANDARD.encode(out))
}

pub fn open(encryption: &[u8; 32], kind: &str, sealed: &str) -> anyhow::Result<Vec<u8>> {
    let raw = STANDARD.decode(sealed)?;
    anyhow::ensure!(raw.len() > NONCE_LEN, "donnees tronquees");
    let (nonce, rest) = raw.split_at(NONCE_LEN);
    let nonce = Nonce::try_assume_unique_for_key(nonce).map_err(|_| anyhow::anyhow!("nonce invalide"))?;
    let mut buffer = rest.to_vec();
    let plain = key(encryption)?
        .open_in_place(nonce, Aad::from(kind.as_bytes()), &mut buffer)
        .map_err(|_| anyhow::anyhow!("donnees illisibles (autre cle ou alterees)"))?;
    Ok(plain.to_vec())
}

pub fn b64(bytes: &[u8]) -> String {
    STANDARD.encode(bytes)
}

pub fn from_b64(text: &str) -> anyhow::Result<Vec<u8>> {
    Ok(STANDARD.decode(text)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_deux_cles_different_et_dependent_du_mot_de_passe() {
        let salt = [7u8; SALT_LEN];
        let a = derive("mot de passe", &salt, 1000);
        let b = derive("mot de passe", &salt, 1000);
        let c = derive("autre", &salt, 1000);
        assert_eq!(a.access, b.access);
        assert_ne!(a.access, a.encryption);
        assert_ne!(a.access, c.access);
    }

    #[test]
    fn aller_retour_et_refus_d_un_autre_type_ou_d_une_autre_cle() {
        let keys = derive("x", &[1u8; SALT_LEN], 1000);
        let sealed = seal(&keys.encryption, "favoris", b"bonjour").unwrap();
        assert_eq!(open(&keys.encryption, "favoris", &sealed).unwrap(), b"bonjour");
        assert!(open(&keys.encryption, "reglages", &sealed).is_err(), "type echange");
        assert!(open(&keys.access, "favoris", &sealed).is_err(), "autre cle");
    }
}
