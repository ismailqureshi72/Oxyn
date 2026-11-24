// /ssr/<key>.htm: the server-rendered markup of a landing preview that sits in a hidden tab,
// fetched by <oxyn-demo data-ssr> when it is first shown (see lib/lazy-ssr.ts). A bare fragment.
import type { APIRoute, GetStaticPaths } from 'astro';
import { lazySsr } from '../../lib/lazy-ssr';

export const getStaticPaths: GetStaticPaths = () => lazySsr().map(({ key, html }) => ({ params: { key }, props: { html } }));

export const GET: APIRoute = ({ props }) =>
  new Response(props.html as string, { headers: { 'content-type': 'text/html; charset=utf-8' } });
