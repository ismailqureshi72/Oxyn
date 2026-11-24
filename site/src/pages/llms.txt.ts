// /llms.txt: see src/lib/llms.ts.
import type { APIRoute } from 'astro';
import { llmsTxt } from '../lib/llms';
export const GET: APIRoute = async () => new Response(await llmsTxt(), { headers: { 'content-type': 'text/plain; charset=utf-8' } });
