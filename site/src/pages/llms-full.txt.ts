// /llms-full.txt: see src/lib/llms.ts.
import type { APIRoute } from 'astro';
import { llmsFullTxt } from '../lib/llms';
export const GET: APIRoute = async () => new Response(await llmsFullTxt(), { headers: { 'content-type': 'text/plain; charset=utf-8' } });
