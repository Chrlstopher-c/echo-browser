// Meme moteur que core/shell/src/search.rs : a changer des deux cotes.
const SEARCH = 'https://www.google.com/search?q=';

function target(raw) {
  const text = raw.trim();
  if (text === '') return null;
  if (/^[a-z][a-z0-9+.-]*:\/\//i.test(text)) return text;
  if (!/\s/.test(text) && (/^localhost(:\d+)?(\/|$)/.test(text) || /^[^\s/]+\.[a-z]{2,}(:\d+)?(\/|$)/i.test(text))) {
    return 'https://' + text;
  }
  return SEARCH + encodeURIComponent(text).replace(/%20/g, '+');
}

document.getElementById('recherche').addEventListener('submit', (event) => {
  event.preventDefault();
  const url = target(document.getElementById('champ').value);
  if (url !== null) window.location.href = url;
});
