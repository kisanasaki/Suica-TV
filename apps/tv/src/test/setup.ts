/**
 * VitestへDOM matcherと副作用のないWebSocket既定実装を設定する。
 * 各テストは必要に応じて接続時系列を制御できるFakeへ差し替える。
 */

import '@testing-library/jest-dom/vitest';

class PassiveWebSocket extends EventTarget {
  static readonly CONNECTING = 0;
  static readonly OPEN = 1;
  static readonly CLOSING = 2;
  static readonly CLOSED = 3;
  readonly url: string;
  readyState = PassiveWebSocket.CONNECTING;

  constructor(url: string | URL) {
    super();
    this.url = url.toString();
  }

  send() {}
  close() { this.readyState = PassiveWebSocket.CLOSED; }
}

Object.defineProperty(globalThis, 'WebSocket', { value: PassiveWebSocket, writable: true });
Object.defineProperty(globalThis.crypto, 'randomUUID', {
  value: () => '550e8400-e29b-41d4-a716-446655440000',
  configurable: true,
});
