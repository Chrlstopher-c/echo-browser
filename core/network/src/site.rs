//! Responsabilite : le « site » d'une adresse (domaine enregistrable, ex. `news.bbc.co.uk` → `bbc.co.uk`), et dire si
//! une requete est tierce par rapport a la page qui l'emet.

/// L'hote d'une adresse http(s), sans port ni identifiants.
pub fn host_of(url: &str) -> Option<&str> {
    let rest = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://"))
        .or_else(|| url.strip_prefix("wss://")).or_else(|| url.strip_prefix("ws://"))?;
    let authority = rest.split(['/', '?', '#']).next()?;
    let host = authority.rsplit('@').next()?;
    let host = if host.starts_with('[') { host.split(']').next().map(|h| &h[1..])? } else { host.split(':').next()? };
    (!host.is_empty()).then_some(host)
}

/// Le domaine enregistrable d'un hote (ou l'hote lui-meme : adresse IP, nom local).
pub fn site_of_host(host: &str) -> String {
    let lower = host.to_ascii_lowercase();
    let ip = lower.contains(':') || lower.bytes().all(|b| b.is_ascii_digit() || b == b'.');
    if ip {
        return lower;
    }
    psl::domain_str(&lower).map(str::to_string).unwrap_or(lower)
}

pub fn site_of(url: &str) -> Option<String> {
    host_of(url).map(site_of_host)
}

/// La requete vient-elle d'un autre site que la page ?
pub fn is_third_party(url: &str, page: &str) -> bool {
    match (site_of(url), site_of(page)) {
        (Some(a), Some(b)) => a != b,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domaine_enregistrable() {
        assert_eq!(site_of("https://news.bbc.co.uk/x").as_deref(), Some("bbc.co.uk"));
        assert_eq!(site_of("https://a.b.example.com:8443/p?q").as_deref(), Some("example.com"));
        assert_eq!(site_of("http://user:pw@192.168.1.10/").as_deref(), Some("192.168.1.10"));
        assert_eq!(site_of("echo://ui/index.html"), None);
    }

    #[test]
    fn tiers() {
        assert!(is_third_party("https://cdn.tracker.net/a.js", "https://www.example.com/"));
        assert!(!is_third_party("https://static.example.com/a.js", "https://www.example.com/"));
    }
}
