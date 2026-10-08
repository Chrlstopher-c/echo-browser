// Vue de lecture : l'article extrait par Readability remplace la page, dans la page elle-meme (meme origine, aucun
// privilege d'Echo). Theme clair ou sombre selon `prefers-color-scheme`, qu'Echo aligne sur le sien.
(Readability) => {
  const article = new Readability(document.cloneNode(true), { charThreshold: 300 }).parse();
  // Readability rend toujours quelque chose : moins de 250 caracteres, ce n'est pas un article.
  if (!article || !article.content || (article.textContent || '').trim().length < 250) {
    const toast = document.createElement('div');
    toast.textContent = 'Pas d’article à lire sur cette page.';
    toast.style.cssText = 'position:fixed;left:50%;bottom:24px;transform:translateX(-50%);z-index:2147483647;'
      + 'padding:10px 16px;border-radius:12px;background:#222326;color:#eee;font:13px system-ui;';
    document.documentElement.appendChild(toast);
    setTimeout(() => toast.remove(), 2500);
    return;
  }
  const minutes = Math.max(1, Math.round((article.textContent || '').split(/\s+/).length / 230));
  const style = `
    :root { color-scheme: light dark; --fond: #eceef1; --carte: #f4f5f7; --encre: #1f2124; --doux: #6b7078;
      --lien: #2f7d5b; --ombre: 6px 6px 14px #cfd2d8, -6px -6px 14px #ffffff; }
    @media (prefers-color-scheme: dark) { :root { --fond: #1d1e21; --carte: #232428; --encre: #e6e4df;
      --doux: #8d9097; --lien: #6fcf9f; --ombre: 6px 6px 14px #141517, -6px -6px 14px #27292d; } }
    html { background: var(--fond); }
    body { margin: 0; color: var(--encre); font: 19px/1.7 Charter, 'Iowan Old Style', Georgia, serif; }
    .barre { position: sticky; top: 0; display: flex; justify-content: space-between; align-items: center;
      padding: 10px 20px; background: var(--fond); font: 12.5px system-ui, sans-serif; color: var(--doux); }
    .barre button { border: 0; border-radius: 999px; padding: 7px 14px; background: var(--carte); color: var(--encre);
      box-shadow: var(--ombre); font: inherit; cursor: pointer; }
    main { max-width: 680px; margin: 0 auto; padding: 24px 24px 120px; }
    h1 { font: 600 34px/1.2 system-ui, sans-serif; margin: 12px 0 8px; letter-spacing: -0.01em; }
    .meta { color: var(--doux); font: 13.5px system-ui, sans-serif; margin-bottom: 32px; }
    img, video, figure { max-width: 100%; height: auto; border-radius: 10px; }
    figure { margin: 24px 0; } figcaption { color: var(--doux); font-size: 14px; }
    a { color: var(--lien); } pre, code { font-size: 15px; white-space: pre-wrap; }
    table { font: 14.5px/1.5 system-ui, sans-serif; border-collapse: collapse; margin: 20px 0; }
    td, th { padding: 4px 10px 4px 0; vertical-align: top; text-align: left; }
    blockquote { margin: 20px 0; padding-left: 18px; border-left: 3px solid var(--lien); color: var(--doux); }`;
  const meta = [article.byline, article.siteName, `${minutes} min de lecture`].filter(Boolean).join(' · ');
  const html = document.documentElement;
  html.innerHTML = '<head><meta charset="utf-8"><title></title><style></style></head><body>'
    + '<div class="barre"><span>Mode lecture</span><button type="button">Quitter la lecture</button></div>'
    + '<main><h1></h1><p class="meta"></p><article></article></main></body>';
  document.title = article.title || document.title;
  html.querySelector('style').textContent = style;
  html.querySelector('h1').textContent = article.title || '';
  html.querySelector('.meta').textContent = meta;
  html.querySelector('article').innerHTML = article.content;
  html.querySelectorAll('article script, article iframe, article object, article embed, article form')
    .forEach((el) => el.remove());
  html.querySelectorAll('article *').forEach((el) => [...el.attributes]
    .filter((a) => a.name.startsWith('on')).forEach((a) => el.removeAttribute(a.name)));
  html.querySelector('.barre button').addEventListener('click', () => location.reload());
  html.dataset.echoLecture = '1';
  window.scrollTo(0, 0);
  console.debug('echo:lecture:on');
}
