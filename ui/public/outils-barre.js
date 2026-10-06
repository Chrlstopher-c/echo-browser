// Barre au-dessus des outils de developpement ancres : la croix les referme.
document.getElementById('fermer').addEventListener('click', () => {
  fetch('echo://ui/ipc', { method: 'POST', body: JSON.stringify({ kind: 'closeDevTools' }) });
});
