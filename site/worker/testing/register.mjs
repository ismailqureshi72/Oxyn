// Lets `node --test` run the Worker's TypeScript as wrangler bundles it: extensionless relative
// imports, JSON imports and the admin UI files imported as text. Used by `npm test` only.
import { register } from 'node:module';
register('./loader.mjs', import.meta.url);
