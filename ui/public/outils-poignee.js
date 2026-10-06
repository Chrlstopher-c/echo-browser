// Poignee entre la page et les outils : glisser elargit ou retrecit le panneau.
// Le deplacement part en pas relatifs : le coeur deplace aussi la poignee, ses coordonnees bougent avec elle.
let last = null;
let pending = 0;
let timer = 0;

function flush() {
  timer = 0;
  if (pending === 0) return;
  const dx = Math.round(pending);
  pending -= dx;
  fetch('echo://ui/ipc', { method: 'POST', body: JSON.stringify({ kind: 'resizeDevTools', dx }) });
}

document.addEventListener('pointerdown', (event) => {
  last = event.screenX;
  document.body.setPointerCapture(event.pointerId);
  document.body.classList.add('actif');
});
document.addEventListener('pointermove', (event) => {
  if (last === null) return;
  pending += event.screenX - last;
  last = event.screenX;
  if (timer === 0) timer = setTimeout(flush, 16);
});
const stop = () => {
  last = null;
  document.body.classList.remove('actif');
  flush();
};
document.addEventListener('pointerup', stop);
document.addEventListener('pointercancel', stop);
