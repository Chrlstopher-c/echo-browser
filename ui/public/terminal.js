const THEME = {
  background: '#141517', foreground: '#e7e8ea', cursor: '#55b98d', cursorAccent: '#141517',
  selectionBackground: '#2f3a36', black: '#1b1d21', brightBlack: '#63686f', red: '#d5564c',
  green: '#55b98d', yellow: '#d2a056', blue: '#7a9bd6', magenta: '#b08fd0', cyan: '#6bb7c4',
  white: '#e7e8ea', brightWhite: '#ffffff',
};
const BASE = 'echo://ui/term/';
const MIN_DELAY = 16;
const MAX_DELAY = 250;

const term = new Terminal({
  fontFamily: "'JetBrains Mono', ui-monospace, monospace", fontSize: 13, lineHeight: 1.15,
  cursorBlink: true, scrollback: 5000, theme: THEME, allowProposedApi: true,
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
addEventListener('resize', resize);
addEventListener('focus', () => term.focus());
