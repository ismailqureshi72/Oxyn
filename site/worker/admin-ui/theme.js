// Theme for the admin refusal pages (no app code, nothing private): the site's saved choice, else
// the system setting. app.js does the same for the panel itself.
(() => {
  const saved = localStorage.getItem('starlight-theme');
  const dark = saved === 'dark' || (saved !== 'light' && window.matchMedia && matchMedia('(prefers-color-scheme: dark)').matches);
  document.documentElement.dataset.theme = dark ? 'dark' : 'light';
})();
