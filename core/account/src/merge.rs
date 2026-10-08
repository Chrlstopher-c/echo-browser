//! Responsabilite : fusionner a trois voies (derniere version synchronisee, version locale, version du service), pour
//! qu'une modification faite sur une machine ne soit jamais ecrasee par une autre faite ailleurs.

use serde_json::{Map, Value};

/// Comment fusionner un type de donnees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// Objet cle → valeur (reglages) : par cle, la valeur modifiee localement l'emporte, sinon celle du service.
    Map,
    /// Objet cle → liste d'identifiants (extensions par profil) : ensembles fusionnes cle par cle.
    SetMap,
    /// Liste d'objets identifies par un champ (favoris par `url`) : ajouts et retraits des deux cotes.
    KeyedList(&'static str),
    /// Objet adresse → `{"t", "v"}` (visite) ou `{"d"}` (effacement), `*` effacant tout ce qui precede : par cle, la
    /// date la plus recente l'emporte ; seules les `max` cles les plus recentes sont gardees (historique).
    Latest { max: usize },
}

pub fn empty(shape: Shape) -> Value {
    match shape {
        Shape::Map | Shape::SetMap | Shape::Latest { .. } => Value::Object(Map::new()),
        Shape::KeyedList(_) => Value::Array(Vec::new()),
    }
}

pub fn merge(shape: Shape, base: &Value, local: &Value, remote: &Value) -> Value {
    match shape {
        Shape::Map => merge_map(base, local, remote, |b, l, r| if l != b { l.clone() } else { r.clone() }),
        Shape::SetMap => merge_map(base, local, remote, |b, l, r| merge_set(b, l, r)),
        Shape::KeyedList(key) => merge_list(key, base, local, remote),
        Shape::Latest { max } => merge_latest(max, local, remote),
    }
}

/// Date d'une entree d'historique : sa visite ou son effacement.
pub fn stamp(entry: &Value) -> i64 {
    entry["v"].as_i64().or_else(|| entry["d"].as_i64()).unwrap_or(0)
}

fn merge_latest(max: usize, local: &Value, remote: &Value) -> Value {
    let mut out = obj(remote);
    for (key, mine) in obj(local) {
        let keep_mine = match out.get(&key) {
            None => true,
            Some(theirs) => stamp(&mine) > stamp(theirs) || (stamp(&mine) == stamp(theirs) && mine.get("d").is_some()),
        };
        if keep_mine {
            out.insert(key, mine);
        }
    }
    let cleared = out.get("*").map(stamp).unwrap_or(i64::MIN);
    let mut entries: Vec<(String, Value)> = out.into_iter().filter(|(k, v)| k == "*" || stamp(v) > cleared).collect();
    entries.sort_by_key(|(k, v)| std::cmp::Reverse(if k == "*" { i64::MAX } else { stamp(v) }));
    entries.truncate(max + 1);
    Value::Object(entries.into_iter().collect())
}

fn obj(value: &Value) -> Map<String, Value> {
    value.as_object().cloned().unwrap_or_default()
}

fn merge_map(base: &Value, local: &Value, remote: &Value, pick: impl Fn(&Value, &Value, &Value) -> Value) -> Value {
    let (base, local, remote) = (obj(base), obj(local), obj(remote));
    let mut keys: Vec<&String> = remote.keys().chain(local.keys()).chain(base.keys()).collect();
    keys.dedup();
    let mut out = Map::new();
    let null = Value::Null;
    for key in keys {
        if out.contains_key(key) {
            continue;
        }
        let (b, l, r) = (base.get(key).unwrap_or(&null), local.get(key).unwrap_or(&null), remote.get(key).unwrap_or(&null));
        let value = pick(b, l, r);
        if !value.is_null() {
            out.insert(key.clone(), value);
        }
    }
    Value::Object(out)
}

/// Ensembles (listes de chaines) : le service, plus les ajouts locaux, moins les retraits locaux.
fn merge_set(base: &Value, local: &Value, remote: &Value) -> Value {
    let list = |v: &Value| -> Vec<Value> { v.as_array().cloned().unwrap_or_default() };
    let (base, local, remote) = (list(base), list(local), list(remote));
    let mut out: Vec<Value> = remote.into_iter().filter(|x| !(base.contains(x) && !local.contains(x))).collect();
    for item in local {
        if !base.contains(&item) && !out.contains(&item) {
            out.push(item);
        }
    }
    if out.is_empty() { Value::Null } else { Value::Array(out) }
}

fn merge_list(key: &str, base: &Value, local: &Value, remote: &Value) -> Value {
    let list = |v: &Value| -> Vec<Value> { v.as_array().cloned().unwrap_or_default() };
    let id = |v: &Value| v[key].as_str().map(str::to_string);
    let has = |items: &[Value], k: &Option<String>| items.iter().any(|x| &id(x) == k);
    let (base, local, remote) = (list(base), list(local), list(remote));
    let mut out: Vec<Value> = Vec::new();
    for item in &remote {
        let k = id(item);
        let removed_here = has(&base, &k) && !has(&local, &k);
        if !removed_here {
            // L'element modifie localement (titre…) garde sa version locale.
            let mine = local.iter().find(|x| id(x) == k);
            let theirs_base = base.iter().find(|x| id(x) == k);
            out.push(match (mine, theirs_base) {
                (Some(mine), Some(b)) if mine != b => mine.clone(),
                _ => item.clone(),
            });
        }
    }
    for item in &local {
        let k = id(item);
        let removed_there = has(&base, &k) && !has(&remote, &k);
        if !has(&out, &k) && !removed_there {
            out.push(item.clone());
        }
    }
    Value::Array(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reglages_chaque_cote_garde_ses_modifications() {
        let base = json!({"a": 1, "b": 1, "c": 1});
        let local = json!({"a": 2, "b": 1, "c": 1});
        let remote = json!({"a": 1, "b": 3, "c": 1, "d": 4});
        assert_eq!(merge(Shape::Map, &base, &local, &remote), json!({"a": 2, "b": 3, "c": 1, "d": 4}));
    }

    #[test]
    fn favoris_ajouts_et_retraits_des_deux_cotes() {
        let f = |u: &str| json!({"url": u, "title": u});
        let base = json!([f("1"), f("2"), f("3")]);
        let local = json!([f("1"), f("3"), f("4")]);
        let remote = json!([f("1"), f("2"), f("5")]);
        let urls: Vec<String> = merge(Shape::KeyedList("url"), &base, &local, &remote)
            .as_array().unwrap().iter().map(|x| x["url"].as_str().unwrap().to_string()).collect();
        assert_eq!(urls, ["1", "5", "4"], "2 retire ici, 3 retire la-bas, 4 et 5 ajoutes");
    }

    #[test]
    fn extensions_par_profil_en_ensembles() {
        let base = json!({"graphite": ["p"]});
        let local = json!({"graphite": ["p", "x"]});
        let remote = json!({"graphite": [], "sable": ["y"]});
        assert_eq!(merge(Shape::SetMap, &base, &local, &remote), json!({"graphite": ["x"], "sable": ["y"]}));
    }

    #[test]
    fn historique_la_date_la_plus_recente_l_emporte() {
        let local = json!({"a": {"t": "A", "v": 5}, "b": {"d": 9}, "c": {"t": "C", "v": 2}});
        let remote = json!({"a": {"t": "A", "v": 7}, "b": {"t": "B", "v": 8}, "d": {"t": "D", "v": 1}});
        let merged = merge(Shape::Latest { max: 10 }, &json!({}), &local, &remote);
        assert_eq!(merged, json!({"a": {"t": "A", "v": 7}, "b": {"d": 9}, "c": {"t": "C", "v": 2}, "d": {"t": "D", "v": 1}}));
    }

    #[test]
    fn historique_tout_effacer_et_plafond() {
        let local = json!({"*": {"d": 5}, "x": {"t": "X", "v": 9}});
        let remote = json!({"a": {"t": "A", "v": 3}, "b": {"t": "B", "v": 6}, "c": {"t": "C", "v": 7}});
        let merged = merge(Shape::Latest { max: 2 }, &json!({}), &local, &remote);
        assert_eq!(merged, json!({"*": {"d": 5}, "x": {"t": "X", "v": 9}, "c": {"t": "C", "v": 7}}));
    }

    #[test]
    fn premiere_synchro_sans_base_reunit_tout() {
        let local = json!([{"url": "a"}]);
        let remote = json!([{"url": "b"}]);
        let urls = merge(Shape::KeyedList("url"), &empty(Shape::KeyedList("url")), &local, &remote);
        assert_eq!(urls, json!([{"url": "b"}, {"url": "a"}]));
    }
}
