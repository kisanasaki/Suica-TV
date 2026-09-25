/**
 * WebSocketメッセージの正常系と、欠落・型違い・未知値の拒否を検証する。
 */

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

  it.each([
    '{"type":"server.hello","protocolVersion":"1","serverVersion":"0.1.0","connectionId":"bad","role":"tv"}',
    '{"type":"system.state","mode":"invalid"}',
    '{"type":"system.state","mode":"tv","transitioning":"yes"}',
    '{"type":"remote.command","requestId":"bad","action":"navigation.select"}',
    '{"type":"remote.command","requestId":"550e8400-e29b-41d4-a716-446655440000","action":"shell.run"}',
    '{"type":"command.result","requestId":"550e8400-e29b-41d4-a716-446655440000","ok":"yes"}',
    '{"type":"error","error":{"code":1,"message":"bad"}}',
    '{"type":"server.shutdown","retryAfterSeconds":-1}',
  ])('rejects invalid payload: %s', (payload) => {
    expect(parseServerMessage(payload)).toBeUndefined();
  });

  it('allows unknown fields on otherwise valid messages', () => {
    expect(parseServerMessage(
      '{"type":"system.state","mode":"pc","future":true}',
    )).toEqual({ type: 'system.state', mode: 'pc', future: true });
  });
});
