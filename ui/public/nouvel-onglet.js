// Nouvel onglet : les sites les plus frequentes du profil en tuiles, puis les pages recentes. La saisie se fait dans
// l'adresse de la barre (elle recoit le curseur a l'ouverture) : plus de second champ ici.
const tuiles = document.getElementById('tuiles');
const recents = document.getElementById('recents');

function hote(url) {
  try { return new URL(url).host.replace(/^www\./, ''); } catch { return url; }
}

/** Teinte stable par hote, comme les marques d'onglet de la barre. */
function teinte(host) {
  let hash = 0;
  for (const c of host) hash = (hash * 31 + c.charCodeAt(0)) % 360;
  return hash;
}

function marque(url, favicon) {
  const host = hote(url);
  if (favicon) {
    // L'initiale reste tant que l'icone n'est pas reellement chargee : une icone qui ne repond jamais ne laisse pas
    // de tuile vide.
    const initiale = marque(url, null);
    const img = new Image();
    img.className = 'marque';
    img.alt = '';
    img.addEventListener('load', () => { if (img.naturalWidth > 0) initiale.replaceWith(img); });
    img.src = favicon;
    return initiale;
  }
  const m = document.createElement('span');
  m.className = 'marque';
  m.textContent = host.charAt(0).toUpperCase();
  const h = teinte(host);
  m.style.background = `hsl(${h} 55% 55% / 0.28)`;
  m.style.color = `hsl(${h} 55% 60%)`;
  return m;
}

function tuile(item) {
  const a = document.createElement('a');
  a.className = 'tuile';
  a.href = item.url;
  a.title = item.title || item.url;
  const nom = document.createElement('span');
  nom.className = 'nom';
  nom.textContent = item.title ? item.title.split(/ [–—|·-] /)[0] : hote(item.url);
  a.append(marque(item.url, item.favicon), nom);
  return a;
}

function ligne(item) {
  const a = document.createElement('a');
  a.className = 'ligne';
  a.href = item.url;
  const t = document.createElement('span');
  t.className = 'titre';
  t.textContent = item.title || hote(item.url);
  const h = document.createElement('span');
  h.className = 'hote';
  h.textContent = hote(item.url);
  a.append(marque(item.url, item.favicon), t, h);
  return a;
}

async function charger() {
  try {
    const reponse = await fetch('echo://ui/data/suggest?q=');
    const data = await reponse.json();
    if (!data.ok) return;
    if (data.private) {
      document.body.classList.add('prive');
      document.querySelector('.accroche').textContent =
        'Navigation privée : ni historique, ni cookies, ni session ne sont gardés quand l’onglet se ferme.';
      return;
    }
    tuiles.replaceChildren(...(data.top || []).map(tuile));
    const vus = new Set((data.top || []).map((i) => i.url));
    const liste = (data.history || []).filter((i) => !vus.has(i.url)).slice(0, 5);
    if (liste.length > 0) {
      const titre = document.createElement('h2');
      titre.textContent = 'Récemment';
      recents.replaceChildren(titre, ...liste.map(ligne));
    }
    if ((data.top || []).length === 0 && liste.length === 0) {
      document.querySelector('.accroche').textContent =
        'Tapez une adresse ou une recherche dans la barre, à gauche. Vos sites fréquents apparaîtront ici.';
    }
  } catch (erreur) {
    console.error('sites frequents indisponibles', erreur);
  }
}

charger();
