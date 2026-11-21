// Who is entitled to everything, without D1: pure rules shared by plan_for() (billing.ts) and its
// tests (entitle.test.ts, `npm test` in site/).

/** The two owners: admins with all access even before they are on the D1 `admins` list. Like every
 *  admin they count only once Google or GitHub verified the email (users.email_verified = 1 and a
 *  linked provider account), so signing up with the address and a password is never enough. */
export const OWNERS = ['m.ismailqureshi62@gmail.com', 'khubaibnazeer8@gmail.com'];

/** True for an admin: an owner or an active row in D1 `admins` (`listed`), with a provider-verified
 *  email (`email_verified` and at least one `linked` OAuth account). */
export function isAdminRow(u: { email: string; email_verified: number }, listed: boolean, linked: number): boolean {
  return !!u.email_verified && linked > 0 && (listed || OWNERS.includes(u.email.trim().toLowerCase()));
}
