// node --experimental-strip-types --test worker/entitle.test.ts   (from site/; `npm test`)
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { OWNERS, isAdminRow } from './entitle.ts';

const v = (email: string, verified = 1) => ({ email, email_verified: verified });

test('owners are admins once a provider verified the email', () => {
  for (const e of OWNERS) assert.equal(isAdminRow(v(e), false, 1), true);
  assert.equal(isAdminRow(v('KhubaibNazeer8@Gmail.com'), false, 1), true);
});

test('an owner email without provider verification is not an admin', () => {
  assert.equal(isAdminRow(v(OWNERS[0], 0), false, 1), false);
  assert.equal(isAdminRow(v(OWNERS[0]), false, 0), false);
});

test('anyone on the admins list counts, verified the same way', () => {
  assert.equal(isAdminRow(v('someone@example.com'), true, 2), true);
  assert.equal(isAdminRow(v('someone@example.com', 0), true, 2), false);
  assert.equal(isAdminRow(v('someone@example.com'), false, 2), false);
});
