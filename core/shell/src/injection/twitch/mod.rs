//! Pubs Twitch : chargees par le lecteur lui-meme, hors de portee des listes de blocage. vaft (TwitchAdSolutions,
//! licence MIT, `LICENSE-vaft`, version 37.0.0 du depot archive le 05/03/2026) intercepte le worker du lecteur et
//! remplace le flux publicitaire par un flux sans pub. Mesure du 08/10/2026 sur profils neufs : sans lui, une pub
//! d'environ 16 s a chaque arrivee sur une chaine ; avec lui, aucune pub, la lecture continue (bandeau « Blocking ads »).
//! Applique seulement quand le bouclier est actif pour la page.

use echo_shield::Shield;

const VAFT: &str = include_str!("vaft.js");

/// Le script a poser au tout debut d'une page Twitch, `None` ailleurs ou bouclier coupe.
pub fn script_for(url: &str, shield: &Shield) -> Option<&'static str> {
    (is_twitch(url) && shield.is_active_for(url)).then_some(VAFT)
}

fn is_twitch(url: &str) -> bool {
    let host = url.split("://").nth(1).and_then(|rest| rest.split(['/', '?', '#', ':']).next());
    matches!(host, Some("twitch.tv" | "www.twitch.tv" | "m.twitch.tv"))
}

#[cfg(test)]
mod tests {
    use super::is_twitch;

    #[test]
    fn seulement_les_pages_twitch() {
        assert!(is_twitch("https://www.twitch.tv/squeezie"));
        assert!(is_twitch("https://m.twitch.tv/"));
        assert!(!is_twitch("https://twitch.tv.example.org/"));
        assert!(!is_twitch("https://example.org/?u=https://www.twitch.tv/"));
        assert!(!is_twitch("https://clips.twitch.tv/x"));
    }
}
