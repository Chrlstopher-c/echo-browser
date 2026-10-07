//! Responsabilite : le code injecte dans la page d'interface pour lui donner `window.echo`.

/// Amorce du pont, executee avant les scripts de l'interface.
///
/// L'interface envoie ses demandes par une requete au schema interne ; le coeur repond
/// par la liste des evenements a diffuser. Les evenements spontanes arrivent par
/// `__echoDeliver`, appele depuis le coeur.
pub const BOOTSTRAP: &str = r#"
(() => {
  if (window.echo) return;
  const listeners = new Set();
  // Evenements arrives avant que l'interface s'abonne : gardes, puis rejoues au premier abonne.
  let early = [];
  const deliver = (event) => {
    if (listeners.size === 0 && early !== null) { early.push(event); return; }
    for (const l of listeners) { try { l(event) } catch (e) { console.error(e) } }
  };
  window.__echoDeliver = deliver;
  window.echo = {
    send(request) {
      fetch('echo://ui/ipc', {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify(request),
      })
        .then((r) => r.json())
        .then((events) => { if (Array.isArray(events)) events.forEach(deliver) })
        .catch((e) => console.error('pont indisponible', e));
    },
    subscribe(listener) {
      listeners.add(listener);
      if (early !== null) { const pending = early; early = null; pending.forEach(deliver); }
      return () => listeners.delete(listener);
    },
  };
})();
"#;
