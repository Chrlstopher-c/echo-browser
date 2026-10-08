//! Responsabilite : le journal reseau d'un onglet — les dernieres requetes et leur resume par domaine. Remis a zero
//! quand l'onglet change de page (nouvelle navigation principale).

use std::collections::{BTreeMap, HashMap, VecDeque};

use serde::Serialize;

use crate::site;

/// Requetes gardees par onglet : au-dela, les plus anciennes sortent du detail (le resume, lui, garde tout).
const KEPT: usize = 500;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestEntry {
    pub url: String,
    pub host: String,
    pub kind: String,
    pub method: String,
    pub third_party: bool,
    /// Pourquoi elle a ete bloquee (`bouclier`, `regle`, `isolement`), sinon rien.
    pub blocked: Option<String>,
    pub status: Option<u16>,
    pub bytes: u64,
    pub duration_ms: Option<u64>,
    #[serde(skip)]
    started_ms: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainStat {
    pub host: String,
    pub site: String,
    pub requests: u32,
    pub bytes: u64,
    pub blocked: u32,
    pub third_party: bool,
    /// Requetes par type (`script`, `image`…).
    pub kinds: BTreeMap<String, u32>,
}

#[derive(Debug, Default)]
pub struct TabLog {
    page: String,
    entries: VecDeque<(u64, RequestEntry)>,
    domains: HashMap<String, DomainStat>,
    total_bytes: u64,
}

/// Une requete qui part : ce que le navigateur en sait avant de l'envoyer.
pub struct Outgoing<'a> {
    pub id: u64,
    pub url: &'a str,
    pub page: &'a str,
    pub kind: &'a str,
    pub method: &'a str,
    pub blocked: Option<&'a str>,
    pub now_ms: u64,
}

impl TabLog {
    pub fn page(&self) -> &str {
        &self.page
    }

    /// Nouvelle page dans l'onglet : le journal repart de zero.
    pub fn navigated(&mut self, page: &str) {
        *self = TabLog { page: page.to_string(), ..Default::default() };
    }

    pub fn start(&mut self, out: Outgoing<'_>) {
        let Some(host) = site::host_of(out.url) else { return };
        let host = host.to_ascii_lowercase();
        let third_party = site::is_third_party(out.url, out.page);
        let stat = self.domains.entry(host.clone()).or_insert_with(|| DomainStat {
            host: host.clone(),
            site: site::site_of_host(&host),
            third_party,
            ..Default::default()
        });
        stat.requests += 1;
        stat.blocked += u32::from(out.blocked.is_some());
        *stat.kinds.entry(out.kind.to_string()).or_default() += 1;
        if self.entries.len() == KEPT {
            self.entries.pop_front();
        }
        self.entries.push_back((out.id, RequestEntry {
            url: out.url.to_string(),
            host,
            kind: out.kind.to_string(),
            method: out.method.to_string(),
            third_party,
            blocked: out.blocked.map(str::to_string),
            status: None,
            bytes: 0,
            duration_ms: None,
            started_ms: out.now_ms,
        }));
    }

    /// La reponse est arrivee (ou la requete a echoue) : statut, octets recus, duree.
    pub fn complete(&mut self, id: u64, status: u16, bytes: u64, now_ms: u64) {
        let Some((_, entry)) = self.entries.iter_mut().rev().find(|(i, _)| *i == id) else { return };
        entry.status = Some(status);
        entry.bytes = bytes;
        entry.duration_ms = Some(now_ms.saturating_sub(entry.started_ms));
        self.total_bytes += bytes;
        if let Some(stat) = self.domains.get_mut(&entry.host) {
            stat.bytes += bytes;
        }
    }

    /// Les domaines, les plus sollicites d'abord.
    pub fn summary(&self) -> Vec<DomainStat> {
        let mut all: Vec<DomainStat> = self.domains.values().cloned().collect();
        all.sort_by(|a, b| b.requests.cmp(&a.requests).then(b.bytes.cmp(&a.bytes)).then(a.host.cmp(&b.host)));
        all
    }

    /// Les dernieres requetes (d'un domaine, ou toutes), les plus recentes d'abord.
    pub fn recent(&self, host: Option<&str>, limit: usize) -> Vec<RequestEntry> {
        self.entries
            .iter()
            .rev()
            .filter(|(_, e)| host.is_none_or(|h| e.host == h))
            .take(limit)
            .map(|(_, e)| e.clone())
            .collect()
    }

    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn out<'a>(id: u64, url: &'a str, blocked: Option<&'a str>) -> Outgoing<'a> {
        Outgoing { id, url, page: "https://www.example.com/", kind: "script", method: "GET", blocked, now_ms: 1000 }
    }

    #[test]
    fn resume_par_domaine_tiers_et_blocages() {
        let mut log = TabLog::default();
        log.navigated("https://www.example.com/");
        log.start(out(1, "https://www.example.com/app.js", None));
        log.start(out(2, "https://ads.tracker.net/t.js", Some("bouclier")));
        log.start(out(3, "https://www.example.com/b.js", None));
        log.complete(1, 200, 1500, 1040);
        let summary = log.summary();
        assert_eq!(summary[0].host, "www.example.com");
        assert_eq!((summary[0].requests, summary[0].bytes, summary[0].third_party), (2, 1500, false));
        assert_eq!((summary[1].blocked, summary[1].third_party), (1, true));
        assert_eq!(log.recent(Some("www.example.com"), 10)[1].duration_ms, Some(40));
        log.navigated("https://autre.fr/");
        assert!(log.summary().is_empty());
    }
}
