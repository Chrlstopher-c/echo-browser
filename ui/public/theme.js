// Les jetons de l'espace courant, posés par l'interface dans le stockage local, appliqués aux pages statiques
// (accueil, terminal, liseré). Chargé en tête de page, avant la première peinture.
(() => {
  const MAP = {
    shell: '--shell', glow: '--glow', field: '--field', hover: '--hover', hairline: '--hairline', ink: '--ink',
    inkMuted: '--ink-muted', inkFaint: '--ink-faint', tint: '--tint', hi: '--hi', lo: '--lo', guard: '--guard',
  };

  function apply(theme) {
    const root = document.documentElement;
    for (const [key, variable] of Object.entries(MAP)) {
      if (typeof theme[key] === 'string') root.style.setProperty(variable, theme[key]);
    }
    const shadows = theme.shadows || {};
    if (shadows.field) root.style.setProperty('--shadow-field', shadows.field);
    if (shadows.pressed) root.style.setProperty('--shadow-pressed', shadows.pressed);
    if (shadows.card) root.style.setProperty('--shadow-card', shadows.card);
    root.style.colorScheme = theme.scheme === 'light' ? 'light' : 'dark';
    if (typeof window.onEchoTheme === 'function') window.onEchoTheme(theme);
  }

  function read() {
    try {
      const raw = localStorage.getItem('echo.theme');
      return raw ? JSON.parse(raw) : null;
    } catch (error) {
      return null;
    }
  }

  const theme = read();
  if (theme) apply(theme);
  addEventListener('storage', (event) => {
    if (event.key !== 'echo.theme') return;
    const next = read();
    if (next) apply(next);
  });
  window.echoTheme = { read, apply };
})();
