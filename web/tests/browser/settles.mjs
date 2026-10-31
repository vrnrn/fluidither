// Springs land when they land, so a fixed wait fails on a slow machine. This waits for the value.
import assert from 'node:assert/strict';
import { isDeepStrictEqual } from 'node:util';

const LONGEST_WAIT_MS = 5000;
const POLL_MS = 30;

export async function settles(read, wanted) {
  const deadline = Date.now() + LONGEST_WAIT_MS;
  let value = await read();
  while (!isDeepStrictEqual(value, wanted) && Date.now() < deadline) {
    await new Promise((resolve) => setTimeout(resolve, POLL_MS));
    value = await read();
  }
  assert.deepEqual(value, wanted);
}
