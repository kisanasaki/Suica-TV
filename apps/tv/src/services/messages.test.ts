import { describe, expect, it } from 'vitest';
import { parseServerMessage } from './messages';

describe('parseServerMessage', () => {
  it('parses supported messages', () => {
    expect(parseServerMessage('{"type":"system.state","mode":"tv"}')).toEqual({ type: 'system.state', mode: 'tv' });
  });

  it('ignores malformed and unknown messages', () => {
    expect(parseServerMessage('{bad json')).toBeUndefined();
    expect(parseServerMessage('{"type":"future.message"}')).toBeUndefined();
    expect(parseServerMessage('[]')).toBeUndefined();
  });
});
