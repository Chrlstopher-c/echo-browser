// Propose les fiches quand l'utilisateur entre dans un champ reconnu (nom, e-mail, adresse…). La page ne recoit
// rien : elle signale seulement « champ de formulaire », et c'est la barre d'Echo qui propose de remplir.
(() => {
  if (window.__echoOffre) return;
  window.__echoOffre = 1;
  const AUTO = /^(given-name|family-name|name|email|tel|organization|street-address|address-line1|postal-code|address-level2|country|country-name)$/;
  const HINT = /e-?mail|courriel|t[eé]l|phone|postal|zip|ville|city|pays|country|soci[eé]t|company|pr[eé]nom|first.?name|last.?name|\bnom\b|adresse|address|street/i;
  const FORBIDDEN = /pass|cc-|card|carte|cvv|cvc|iban|bic|search|recherche|query|\bq\b/i;
  document.addEventListener('focusin', (event) => {
    const el = event.target;
    if (!(el instanceof HTMLInputElement || el instanceof HTMLSelectElement || el instanceof HTMLTextAreaElement)) return;
    if (el instanceof HTMLInputElement && /^(password|hidden|checkbox|radio|submit|button|file|search)$/.test(el.type)) return;
    const auto = (el.getAttribute('autocomplete') || '').toLowerCase().split(/\s+/).pop();
    const hint = [el.name, el.id, el.placeholder, el.getAttribute('aria-label')].join(' ');
    if (FORBIDDEN.test(auto) || FORBIDDEN.test(hint)) return;
    if (AUTO.test(auto) || HINT.test(hint)) console.debug('echo:formulaire:champ');
  }, true);
})();
