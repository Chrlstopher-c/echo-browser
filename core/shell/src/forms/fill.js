// Remplit les champs vides du formulaire avec une fiche. Jamais un mot de passe, une carte ou un IBAN.
((card) => {
  const KEYS = {
    'given-name': 'givenName', 'family-name': 'familyName', name: 'fullName', email: 'email', tel: 'tel',
    organization: 'organization', 'street-address': 'street', 'address-line1': 'street', 'postal-code': 'postalCode',
    'address-level2': 'city', country: 'country', 'country-name': 'country',
  };
  const GUESS = [
    [/e-?mail|courriel/, 'email'], [/t[eé]l|phone|mobile|portable/, 'tel'], [/postal|zip|\bcp\b/, 'postalCode'],
    [/ville|city|commune|localit/, 'city'], [/pays|country/, 'country'],
    [/soci[eé]t[eé]|entreprise|company|organi/, 'organization'], [/pr[eé]nom|first.?name|given/, 'givenName'],
    [/last.?name|surname|family|\bnom\b|^nom|_nom|nom_/, 'familyName'], [/full.?name|^name$|nom complet/, 'fullName'],
    [/adresse|address|\brue\b|street/, 'street'],
  ];
  const FORBIDDEN = /pass|cc-|card|carte|cvv|cvc|iban|bic|secu|ssn/;
  const labelOf = (el) => {
    const byFor = el.id ? document.querySelector(`label[for="${CSS.escape(el.id)}"]`) : null;
    return ((byFor || el.closest('label'))?.textContent || '') + ' ' + (el.getAttribute('aria-label') || '');
  };
  const keyOf = (el) => {
    const auto = (el.getAttribute('autocomplete') || '').toLowerCase().split(/\s+/).pop();
    const hint = [el.name, el.id, el.placeholder, labelOf(el)].join(' ').toLowerCase();
    if (FORBIDDEN.test(auto) || FORBIDDEN.test(hint)) return null;
    if (KEYS[auto]) return KEYS[auto];
    const found = GUESS.find(([pattern]) => pattern.test(hint));
    return found ? found[1] : null;
  };
  const valueFor = (key) => key === 'fullName'
    ? [card.givenName, card.familyName].filter(Boolean).join(' ') : card[key];
  const put = (el, value) => {
    const proto = el instanceof HTMLSelectElement ? HTMLSelectElement : el instanceof HTMLTextAreaElement
      ? HTMLTextAreaElement : HTMLInputElement;
    if (el instanceof HTMLSelectElement) {
      const wanted = value.toLowerCase();
      const option = [...el.options]
        .find((o) => o.value.toLowerCase() === wanted || o.text.trim().toLowerCase() === wanted);
      if (!option) return false;
      value = option.value;
    }
    Object.getOwnPropertyDescriptor(proto.prototype, 'value').set.call(el, value);
    el.dispatchEvent(new Event('input', { bubbles: true }));
    el.dispatchEvent(new Event('change', { bubbles: true }));
    return true;
  };
  let filled = 0;
  for (const el of document.querySelectorAll('input, select, textarea')) {
    const type = (el.type || '').toLowerCase();
    const skipped = ['password', 'hidden', 'file', 'submit', 'button', 'checkbox', 'radio', 'image', 'reset'];
    if (skipped.includes(type) || el.disabled || el.readOnly || el.offsetParent === null) continue;
    if (el.value && !(el instanceof HTMLSelectElement)) continue;
    const key = keyOf(el);
    const value = key ? valueFor(key) : '';
    if (value && put(el, String(value))) filled += 1;
  }
  return filled;
})
