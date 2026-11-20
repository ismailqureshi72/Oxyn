// Email checks for grants (admin panel). The admin UI runs the same rules in the browser
// (admin-ui/app.js, "Email checks") for instant feedback, but only this copy decides: every grant
// request is checked again here. Keep the two in step; the local API test compares them.

export const EMAIL_MAX = 254;

/** Common providers, for "Did you mean …?" on a near-miss domain. */
export const COMMON_DOMAINS = [
  'gmail.com', 'googlemail.com', 'yahoo.com', 'hotmail.com', 'outlook.com', 'live.com', 'icloud.com', 'me.com',
  'proton.me', 'protonmail.com', 'aol.com', 'msn.com', 'gmx.com', 'mail.com', 'zoho.com', 'yandex.com', 'hey.com',
];
/** Known misspellings seen in the wild → the domain meant. Checked before the edit-distance guess. */
export const DOMAIN_TYPOS: Record<string, string> = {
  'gmial.com': 'gmail.com', 'gamil.com': 'gmail.com', 'gmai.com': 'gmail.com', 'gmal.com': 'gmail.com', 'gmaill.com': 'gmail.com',
  'gnail.com': 'gmail.com', 'gmali.com': 'gmail.com', 'gmail.co': 'gmail.com', 'gmail.cm': 'gmail.com', 'gmail.con': 'gmail.com',
  'gmail.om': 'gmail.com', 'gmail.comm': 'gmail.com', 'gmail.cmo': 'gmail.com', 'gmail.ocm': 'gmail.com', 'gmail.vom': 'gmail.com',
  'gmail.xom': 'gmail.com', 'gmail.cpm': 'gmail.com', 'gmaik.com': 'gmail.com', 'gmeil.com': 'gmail.com', 'gimail.com': 'gmail.com',
  'hotmial.com': 'hotmail.com', 'hotmal.com': 'hotmail.com', 'hotmai.com': 'hotmail.com', 'hotnail.com': 'hotmail.com',
  'hotmail.co': 'hotmail.com', 'hotmail.con': 'hotmail.com', 'hotmali.com': 'hotmail.com', 'homail.com': 'hotmail.com',
  'yahooo.com': 'yahoo.com', 'yaho.com': 'yahoo.com', 'yhaoo.com': 'yahoo.com', 'yahoo.co': 'yahoo.com', 'yahoo.con': 'yahoo.com',
  'yahho.com': 'yahoo.com', 'tahoo.com': 'yahoo.com',
  'outlok.com': 'outlook.com', 'outllok.com': 'outlook.com', 'outloo.com': 'outlook.com', 'outlook.co': 'outlook.com',
  'outlook.con': 'outlook.com', 'outlookk.com': 'outlook.com', 'otlook.com': 'outlook.com',
  'iclod.com': 'icloud.com', 'icoud.com': 'icloud.com', 'icloud.co': 'icloud.com', 'icloud.con': 'icloud.com', 'iclud.com': 'icloud.com',
  'protonmail.co': 'protonmail.com', 'protonmal.com': 'protonmail.com', 'proton.m': 'proton.me',
};
const BAD_TLDS: Record<string, string> = { con: 'com', cmo: 'com', ocm: 'com', vom: 'com', xom: 'com', comm: 'com', cm: 'com', om: 'com' };

/** Edit distance with adjacent swaps (gmial → gmail is one edit). Domains are short. */
function lev(a: string, b: string): number {
  const d: number[][] = Array.from({ length: a.length + 1 }, (_, i) => Array.from({ length: b.length + 1 }, (_, j) => (i === 0 ? j : j === 0 ? i : 0)));
  for (let i = 1; i <= a.length; i++) {
    for (let j = 1; j <= b.length; j++) {
      d[i][j] = Math.min(d[i - 1][j] + 1, d[i][j - 1] + 1, d[i - 1][j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1));
      if (i > 1 && j > 1 && a[i - 1] === b[j - 2] && a[i - 2] === b[j - 1]) d[i][j] = Math.min(d[i][j], d[i - 2][j - 2] + 1);
    }
  }
  return d[a.length][b.length];
}

/** The domain someone most likely meant, or null when the domain looks intended. */
export function suggestDomain(domain: string): string | null {
  if (COMMON_DOMAINS.includes(domain)) return null;
  if (DOMAIN_TYPOS[domain]) return DOMAIN_TYPOS[domain];
  const dot = domain.lastIndexOf('.');
  const tld = domain.slice(dot + 1);
  if (BAD_TLDS[tld]) return domain.slice(0, dot + 1) + BAD_TLDS[tld];
  let best: string | null = null;
  let bestD = 3;
  for (const c of COMMON_DOMAINS) {
    const d = lev(domain, c);
    if (d < bestD) { bestD = d; best = c; }
  }
  // One edit away from a big provider is almost always a slip (gmail.cmo, yahoo.cpm, outlo0k.com);
  // two edits only for longer domains, so short real domains (live.co.uk) are not "corrected".
  return best && (bestD === 1 || (bestD === 2 && domain.length >= 9)) ? best : null;
}

export interface EmailCheck {
  input: string;
  email: string; // normalised: trimmed, lower case, <…> unwrapped
  ok: boolean;
  code: 'ok' | 'empty' | 'invalid' | 'too_long' | 'typo';
  message: string;
  suggestion?: string; // a full address, for "Did you mean …?"
  notes: string[]; // warnings that do not block (gmail dots / +tags, normalised input)
}

const LOCAL = /^[a-z0-9!#$%&'*+/=?^_`{|}~.-]{1,64}$/;
const LABEL = /^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$/;

/** Format + typo check of one address. Normalises (trim, lower case, unwraps "Name <a@b.c>")
 *  but never rewrites the address itself: a suggestion is only offered. */
export function checkEmail(raw: string): EmailCheck {
  const input = String(raw ?? '');
  let email = input.trim();
  const notes: string[] = [];
  const angle = email.match(/<([^<>]+)>\s*$/);
  if (angle) email = angle[1].trim();
  email = email.replace(/^mailto:/i, '');
  const lowered = email.toLowerCase();
  if (lowered !== input.trim() && lowered) notes.push(`Read as ${lowered}.`);
  email = lowered;
  const out = (ok: boolean, code: EmailCheck['code'], message: string, suggestion?: string): EmailCheck =>
    ({ input, email, ok, code, message, ...(suggestion ? { suggestion } : {}), notes });
  if (!email) return out(false, 'empty', 'Enter an email address.');
  if (email.length > EMAIL_MAX) return out(false, 'too_long', `Email addresses are at most ${EMAIL_MAX} characters.`);
  const at = email.split('@');
  if (at.length !== 2) return out(false, 'invalid', at.length < 2 ? 'This is missing the @ and the domain.' : 'An email address has exactly one @.');
  const [local, domain] = at;
  if (!local || !LOCAL.test(local) || local.startsWith('.') || local.endsWith('.') || local.includes('..'))
    return out(false, 'invalid', 'The part before the @ does not look right.');
  const labels = domain.split('.');
  if (labels.length < 2 || !labels.every((l) => LABEL.test(l)) || !/^[a-z]{2,24}$/.test(labels[labels.length - 1]))
    return out(false, 'invalid', 'The domain after the @ does not look right.');
  if (domain === 'gmail.com' || domain === 'googlemail.com') {
    if (local.includes('.') || local.includes('+'))
      notes.push('Gmail ignores dots and anything after +, but this grant matches only this exact address: the person must sign in with exactly this email.');
  } else if (local.includes('+')) notes.push('Has a +tag: the grant matches only this exact address.');
  const sug = suggestDomain(domain);
  if (sug) return out(false, 'typo', `Did you mean ${local}@${sug}?`, `${local}@${sug}`);
  return out(true, 'ok', '');
}
