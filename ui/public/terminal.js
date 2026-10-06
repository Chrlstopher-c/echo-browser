const DARK = {
  background: '#222326', foreground: '#ebeced', cursor: '#55b98d', cursorAccent: '#222326',
  selectionBackground: '#38413d', black: '#1b1c1f', brightBlack: '#6c707a', red: '#d5564c',
  green: '#55b98d', yellow: '#d2a056', blue: '#7a9bd6', magenta: '#b08fd0', cyan: '#6bb7c4',
  white: '#ebeced', brightWhite: '#ffffff',
};
const LIGHT = {
  foreground: '#23252a', cursor: '#2f8f66', black: '#23252a', brightBlack: '#7b808a', red: '#b8423a',
  green: '#2f8f66', yellow: '#a4712a', blue: '#3f67b5', magenta: '#8a5fb0', cyan: '#2d8693',
  white: '#8a8e98', brightWhite: '#23252a',
};

/** Les couleurs du terminal suivent l'espace : le fond est la matiere de la fenetre. */
function termTheme(theme) {
  const light = theme && theme.scheme === 'light';
  const base = light ? LIGHT : DARK;
  const shell = (theme && theme.shell) || DARK.background;
  const selection = light ? 'rgba(0, 0, 0, 0.12)' : DARK.selectionBackground;
  return { ...base, background: shell, cursorAccent: shell, selectionBackground: selection };
}

const stored = window.echoTheme ? window.echoTheme.read() : null;
const BASE = 'echo://ui/term/';
const MIN_DELAY = 16;
const MAX_DELAY = 250;

const term = new Terminal({
  fontFamily: "'JetBrains Mono', ui-monospace, monospace", fontSize: 13, lineHeight: 1.15,
  cursorBlink: true, scrollback: 5000, theme: termTheme(stored), allowProposedApi: true,
});
const fit = new FitAddon.FitAddon();
term.loadAddon(fit);

let since = 0;
let ended = false;
let delay = MIN_DELAY;
let timer = 0;

function schedule(ms) {
  clearTimeout(timer);
  timer = setTimeout(poll, ms);
}

async function poll() {
  try {
    const response = await fetch(`${BASE}read?since=${since}&cols=${term.cols}&rows=${term.rows}`);
    const bytes = new Uint8Array(await response.arrayBuffer());
    if (bytes.length >= 9) {
      const view = new DataView(bytes.buffer);
      since = view.getUint32(0) * 4294967296 + view.getUint32(4);
      const data = bytes.subarray(9);
      if (data.length > 0) {
        term.write(data);
        delay = MIN_DELAY;
      } else {
        delay = Math.min(delay * 1.5, MAX_DELAY);
      }
      if (bytes[8] === 1 && data.length === 0 && !ended) {
        ended = true;
        term.write('\r\n\x1b[2m[session terminée — toute touche la relance]\x1b[0m\r\n');
      }
    }
  } catch (error) {
    console.error('terminal indisponible', error);
    delay = MAX_DELAY;
  }
  schedule(delay);
}

term.onData((data) => {
  if (ended) {
    ended = false;
    since = 0;
    term.reset();
  }
  fetch(`${BASE}write?cols=${term.cols}&rows=${term.rows}`, { method: 'POST', body: new TextEncoder().encode(data) });
  delay = MIN_DELAY;
  schedule(MIN_DELAY);
});

function resize() {
  fit.fit();
  const cell = term._core?._renderService?.dimensions?.css?.cell;
  console.log(`TERM_SIZE ${term.cols}x${term.rows} cell=${cell?.height} host=${document.getElementById('terminal').clientHeight} screen=${term.element.querySelector('.xterm-screen').clientHeight}`);
  fetch(`${BASE}resize?cols=${term.cols}&rows=${term.rows}`, { method: 'POST', body: '' });
}

document.fonts.load("13px 'JetBrains Mono'").finally(() => {
  term.open(document.getElementById('terminal'));
  fit.fit();
  term.focus();
  console.log('TERM_PAGE_READY');
  poll();
});
window.onEchoTheme = (theme) => { term.options.theme = termTheme(theme); };
addEventListener('resize', resize);
addEventListener('focus', () => term.focus());
