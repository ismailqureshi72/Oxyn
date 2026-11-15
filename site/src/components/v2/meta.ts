// Build-time site facts: the repository's star count (fetched once per build, never required).
import { REPO } from '../../lib/data';
let stars: Promise<number | null> | null = null;
export function starCount(): Promise<number | null> {
  stars ??= (async () => {
    if (process.env.OXYN_OFFLINE) return null;
    try {
      const api = REPO.replace('https://github.com/', 'https://api.github.com/repos/');
      const r = await fetch(api, { signal: AbortSignal.timeout(4000), headers: { accept: 'application/vnd.github+json' } });
      if (!r.ok) return null;
      const j = (await r.json()) as { stargazers_count?: number };
      return typeof j.stargazers_count === 'number' ? j.stargazers_count : null;
    } catch {
      return null;
    }
  })();
  return stars;
}
