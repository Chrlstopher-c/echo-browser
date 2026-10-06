// Meme moteur que core/shell/src/search.rs : a changer des deux cotes.
const SEARCH = 'https://www.google.com/search?q=';
const champ = document.getElementById('champ');
const liste = document.getElementById('suggestions');

let lignes = [];
let courant = -1;
let minuteur = 0;

function target(raw) {
  const text = raw.trim();
  if (text === '') return null;
  if (/^[a-z][a-z0-9+.-]*:\/\//i.test(text)) return text;
  if (!/\s/.test(text) && (/^localhost(:\d+)?(\/|$)/.test(text) || /^[^\s/]+\.[a-z]{2,}(:\d+)?(\/|$)/i.test(text))) {
    return 'https://' + text;
  }
  return SEARCH + encodeURIComponent(text).replace(/%20/g, '+');
}

function hote(url) {
  try { return new URL(url).host; } catch { return url; }
}

function ouvrir(ligne) {
  if (ligne.tab !== undefined) {
    fetch('echo://ui/ipc', { method: 'POST', body: JSON.stringify({ kind: 'selectTab', id: ligne.tab }) });
  } else {
    window.location.href = ligne.url;
  }
}

function dessiner(groupes) {
  liste.replaceChildren();
  lignes = [];
  for (const groupe of groupes) {
    if (groupe.items.length === 0) continue;
    const bloc = document.createElement('section');
    bloc.className = 'groupe';
    const titre = document.createElement('h2');
    titre.textContent = groupe.titre;
    bloc.append(titre);
    for (const item of groupe.items) {
      const ligne = document.createElement('div');
      ligne.className = 'ligne';
      ligne.setAttribute('role', 'option');
      const t = document.createElement('span');
      t.className = 'titre';
      t.textContent = item.title || hote(item.url);
      const h = document.createElement('span');
      h.className = 'hote';
      h.textContent = hote(item.url);
      ligne.append(t, h);
      if (groupe.action) {
        const a = document.createElement('span');
        a.className = 'action';
        a.textContent = groupe.action;
        ligne.append(a);
      }
      const donnee = { el: ligne, url: item.url, tab: groupe.tab ? item.id : undefined };
      ligne.addEventListener('mousedown', (e) => { e.preventDefault(); ouvrir(donnee); });
      ligne.addEventListener('mousemove', () => choisir(lignes.indexOf(donnee)));
      lignes.push(donnee);
      bloc.append(ligne);
    }
    liste.append(bloc);
  }
  courant = -1;
}

function choisir(index) {
  if (courant >= 0 && lignes[courant]) lignes[courant].el.classList.remove('actif');
  courant = index;
  if (courant >= 0 && lignes[courant]) lignes[courant].el.classList.add('actif');
}

async function charger() {
  try {
    const reponse = await fetch('echo://ui/data/suggest?q=' + encodeURIComponent(champ.value.trim()));
    const data = await reponse.json();
    if (!data.ok) return;
    dessiner([
      { titre: 'Onglets ouverts', items: data.tabs, tab: true, action: 'Aller à l’onglet' },
      { titre: 'Favoris', items: data.bookmarks },
      { titre: champ.value.trim() === '' ? 'Récemment' : 'Historique', items: data.history },
    ]);
  } catch (erreur) {
    console.error('suggestions indisponibles', erreur);
  }
}

champ.addEventListener('input', () => {
  clearTimeout(minuteur);
  minuteur = setTimeout(charger, 80);
});

champ.addEventListener('keydown', (event) => {
  if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault();
    if (lignes.length === 0) return;
    const pas = event.key === 'ArrowDown' ? 1 : -1;
    const total = lignes.length + 1;
    choisir((((courant + 1 + pas) % total) + total) % total - 1);
  }
});

document.getElementById('recherche').addEventListener('submit', (event) => {
  event.preventDefault();
  if (courant >= 0 && lignes[courant]) return ouvrir(lignes[courant]);
  const url = target(champ.value);
  if (url !== null) window.location.href = url;
});

charger();
