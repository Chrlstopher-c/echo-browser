// Le liseré du bord gauche : quand la souris l'atteint, la barre repliee se montre.
let sent = false;
function reveal() {
  if (sent) return;
  sent = true;
  fetch('echo://ui/ipc', { method: 'POST', body: JSON.stringify({ kind: 'revealSidebar', reveal: true }) });
  setTimeout(() => { sent = false; }, 400);
}
document.addEventListener('mouseover', reveal);
document.addEventListener('mousemove', reveal);
