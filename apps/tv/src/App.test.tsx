import { act, render } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { launchExternalUrl } from './services/launcher';

vi.mock('./services/launcher', () => ({ launchExternalUrl: vi.fn() }));

class FakeWebSocket extends EventTarget {
  static readonly CONNECTING = 0;
  static readonly OPEN = 1;
  static readonly CLOSED = 3;
  readyState = FakeWebSocket.CONNECTING;

  constructor(_url: string | URL) { super(); }
  close() { this.readyState = FakeWebSocket.CLOSED; }
  send() { }
  open() { this.readyState = FakeWebSocket.OPEN; this.dispatchEvent(new Event('open')); }
  receive(value: unknown) {
    this.dispatchEvent(new MessageEvent('message', { data: JSON.stringify(value) }));
  }
}

describe('App remote commands', () => {
  let socket: FakeWebSocket;

  beforeEach(() => {
    vi.mocked(launchExternalUrl).mockReset();
    const WebSocketFactory = class extends FakeWebSocket {
      constructor(url: string | URL) {
        super(url);
        socket = this;
      }
    };
    Object.defineProperty(globalThis, 'WebSocket', { value: WebSocketFactory, writable: true });
  });

  it('activates the selected tile when the iOS remote sends select', () => {
    render(<App />);
    act(() => {
      socket.open();
      socket.receive({
        type: 'remote.command',
        requestId: crypto.randomUUID(),
        action: 'navigation.select',
      });
    });
    expect(launchExternalUrl).toHaveBeenCalledWith('https://www.youtube.com/tv');
  });
});
