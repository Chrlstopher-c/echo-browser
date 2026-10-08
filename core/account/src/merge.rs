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
}

pub fn empty(shape: Shape) -> Value {
    match shape {
        Shape::Map | Shape::SetMap => Value::Object(Map::new()),
        Shape::KeyedList(_) => Value::Array(Vec::new()),
    }
}

pub fn merge(shape: Shape, base: &Value, local: &Value, remote: &Value) -> Value {
    match shape {
        Shape::Map => merge_map(base, local, remote, |b, l, r| if l != b { l.clone() } else { r.clone() }),
        Shape::SetMap => merge_map(base, local, remote, |b, l, r| merge_set(b, l, r)),
        Shape::KeyedList(key) => merge_list(key, base, local, remote),
    }
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
    fn premiere_synchro_sans_base_reunit_tout() {
        let local = json!([{"url": "a"}]);
        let remote = json!([{"url": "b"}]);
        let urls = merge(Shape::KeyedList("url"), &empty(Shape::KeyedList("url")), &local, &remote);
        assert_eq!(urls, json!([{"url": "b"}, {"url": "a"}]));
    }
}
