// Poignee du coin bas droit de la fenetre d'extension : glisser l'agrandit ou la retrecit.
let last = null;
let dx = 0, dy = 0, timer = 0;
function flush() {
  timer = 0;
  if (dx === 0 && dy === 0) return;
  const body = JSON.stringify({ kind: 'resizeExtensionPopup', dx: Math.round(dx), dy: Math.round(dy) });
  dx = 0; dy = 0;
  fetch('echo://ui/ipc', { method: 'POST', body });
}
document.addEventListener('pointerdown', (e) => { last = [e.screenX, e.screenY]; document.body.setPointerCapture(e.pointerId); document.body.classList.add('actif'); });
document.addEventListener('pointermove', (e) => {
  if (last === null) return;
  dx += e.screenX - last[0]; dy += e.screenY - last[1]; last = [e.screenX, e.screenY];
  if (timer === 0) timer = setTimeout(flush, 16);
});
const stop = () => { last = null; document.body.classList.remove('actif'); flush(); };
document.addEventListener('pointerup', stop);
document.addEventListener('pointercancel', stop);
