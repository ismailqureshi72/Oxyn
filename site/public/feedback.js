// The Feedback panel (src/components/Feedback.astro): open/close, kind radios, screenshot, send to
// POST /api/feedback (site/worker/index.ts). Loaded once per page; the panel exists once.
(function () {
  const dlg = document.getElementById('feedback');
  if (!dlg || dlg.dataset.ready) return;
  dlg.dataset.ready = '1';
  const form = dlg.querySelector('form');
  const $ = (s) => dlg.querySelector(s);
  const kinds = [...dlg.querySelectorAll('[data-kind]')];
  const msg = form.elements.message;
  const email = form.elements.email;
  const file = form.elements.screenshot;
  const count = $('#fb-count');
  const err = $('[data-fb-err]');
  const send = $('[data-fb-send]');
  const fname = $('[data-fb-fname]');
  const rm = $('[data-fb-rm]');
  const done = $('[data-fb-done]');
  const body = $('[data-fb-form]');
  const MAX = 2 * 1024 * 1024;
  const TYPES = ['image/png', 'image/jpeg', 'image/webp', 'image/gif'];
  const HINT = { Bug: 'What went wrong? Steps help.', Idea: 'What would make Oxyn better?', Question: 'What would you like to know?' };
  let opener = null;
  let kind = 'Bug';
  let busy = false;

  const btns = () => document.querySelectorAll('[data-open-feedback]');
  const focusables = () => [...dlg.querySelectorAll('button,input,textarea,label[for],[tabindex="0"]')].filter((e) => !e.closest('[hidden]') && !e.closest('.fb-hp') && e.offsetParent !== null && e.tabIndex >= 0);

  function setKind(k, focus) {
    kind = k;
    kinds.forEach((b) => { const on = b.dataset.kind === k; b.setAttribute('aria-checked', String(on)); b.tabIndex = on ? 0 : -1; if (on && focus) b.focus(); });
    msg.placeholder = HINT[k];
  }
  function showErr(t, field) {
    err.textContent = t; err.hidden = !t;
    [msg, email].forEach((f) => f.removeAttribute('aria-invalid'));
    if (field === 'message') msg.setAttribute('aria-invalid', 'true');
    if (field === 'email') email.setAttribute('aria-invalid', 'true');
  }
  function reset() {
    form.reset(); setKind('Bug'); clearFile(); showErr('');
    count.textContent = '0 / 4000';
    body.hidden = false; done.hidden = true; form.dataset.state = 'form';
  }
  function clearFile() { file.value = ''; fname.hidden = true; rm.hidden = true; fname.textContent = ''; }

  function open(from) {
    opener = from || document.activeElement;
    dlg.hidden = false; dlg.classList.add('open');
    btns().forEach((b) => b.setAttribute('aria-expanded', 'true'));
    if (form.dataset.state === 'done') reset();
    requestAnimationFrame(() => (kinds.find((b) => b.tabIndex === 0) || msg).focus());
  }
  function close() {
    dlg.classList.remove('open'); dlg.hidden = true;
    btns().forEach((b) => b.setAttribute('aria-expanded', 'false'));
    if (opener && document.contains(opener)) opener.focus();
  }
  window.oxynFeedback = { open, close };

  document.addEventListener('click', (e) => {
    const o = e.target.closest('[data-open-feedback]');
    if (o) { e.preventDefault(); dlg.hidden ? open(o) : close(); return; }
    if (e.target.closest('[data-close-feedback]')) { close(); return; }
    if (!dlg.hidden && !e.target.closest('.fb-box')) close();
  });
  dlg.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') { e.preventDefault(); close(); return; }
    if (e.key === 'Tab') {
      const f = focusables(); if (!f.length) return;
      const first = f[0], last = f[f.length - 1];
      if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
      else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first.focus(); }
    }
    const k = e.target.closest('[data-kind]');
    if (k && /^Arrow(Left|Right|Up|Down)$/.test(e.key)) {
      e.preventDefault();
      const i = kinds.indexOf(k), d = /Right|Down/.test(e.key) ? 1 : -1;
      setKind(kinds[(i + d + kinds.length) % kinds.length].dataset.kind, true);
    }
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') { e.preventDefault(); form.requestSubmit(); }
  });
  // The label opens the file picker; Enter / Space on it should too.
  $('.fb-attach').addEventListener('keydown', (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); file.click(); } });
  $('.fb-attach').tabIndex = 0;
  kinds.forEach((b) => b.addEventListener('click', () => setKind(b.dataset.kind)));
  msg.addEventListener('input', () => { count.textContent = `${msg.value.length} / 4000`; if (err.textContent) showErr(''); });
  file.addEventListener('change', () => {
    const f = file.files && file.files[0];
    if (!f) return clearFile();
    if (!TYPES.includes(f.type)) { clearFile(); return showErr('Screenshots must be PNG, JPEG, WebP or GIF images.'); }
    if (f.size > MAX) { clearFile(); return showErr('That screenshot is over 2 MB. Try a smaller one.'); }
    showErr(''); fname.textContent = `${f.name} · ${Math.max(1, Math.round(f.size / 1024))} KB`; fname.hidden = false; rm.hidden = false;
  });
  rm.addEventListener('click', () => { clearFile(); $('.fb-attach').focus(); });
  $('[data-fb-again]').addEventListener('click', () => { reset(); requestAnimationFrame(() => kinds[0].focus()); });

  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    if (busy) return;
    const text = msg.value.trim();
    if (text.length < 3) { showErr('Please write a little more.', 'message'); return msg.focus(); }
    if (email.value.trim() && !email.checkValidity()) { showErr('That email address does not look right.', 'email'); return email.focus(); }
    const fd = new FormData(form);
    fd.set('kind', kind);
    fd.set('page_url', location.href);
    if (!(file.files && file.files[0])) fd.delete('screenshot');
    busy = true; form.dataset.state = 'sending'; send.disabled = true; send.textContent = 'Sending…'; showErr('');
    try {
      const r = await fetch('/api/feedback', { method: 'POST', body: fd, credentials: 'same-origin' });
      let j = null; try { j = await r.json(); } catch { /* not JSON */ }
      if (!r.ok || !j || !j.ok) {
        const m = j && j.error ? j.error.message : `Could not send (${r.status}). Please try again.`;
        form.dataset.state = 'error'; showErr(m, j && j.error && j.error.field);
        return;
      }
      form.dataset.state = 'done'; body.hidden = true; done.hidden = false; done.focus();
    } catch {
      form.dataset.state = 'error'; showErr('You seem to be offline. Please try again.');
    } finally {
      busy = false; send.disabled = false; send.textContent = 'Send';
    }
  });
})();
