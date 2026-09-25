/**
 * CoreClientのhandshake、要求完了、切断、再接続、古いsocketの無効化を検証する。
 * FakeWebSocketで時系列を制御し、実ネットワーク由来の揺らぎを排除する。
 */

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { CoreClient } from './coreClient';

class FakeWebSocket extends EventTarget {
  static readonly CONNECTING = 0;
  static readonly OPEN = 1;
  static readonly CLOSED = 3;
  readyState = FakeWebSocket.CONNECTING;
  sent: string[] = [];
  close = vi.fn(() => { this.readyState = FakeWebSocket.CLOSED; });
  send = vi.fn((message: string) => this.sent.push(message));

  open() { this.readyState = FakeWebSocket.OPEN; this.dispatchEvent(new Event('open')); }
  receive(value: unknown) { this.dispatchEvent(new MessageEvent('message', { data: JSON.stringify(value) })); }
  disconnect() { this.readyState = FakeWebSocket.CLOSED; this.dispatchEvent(new CloseEvent('close')); }
}

describe('CoreClient', () => {
  let sockets: FakeWebSocket[];

  beforeEach(() => {
    vi.useFakeTimers();
    sockets = [];
    Object.defineProperty(globalThis, 'WebSocket', { value: FakeWebSocket, writable: true });
  });

  afterEach(() => vi.useRealTimers());

  function createClient(onStatus = vi.fn(), onMessage = vi.fn()) {
    const client = new CoreClient({
      url: 'ws://localhost:3030/ws?role=tv&protocolVersion=1',
      onStatus,
      onMessage,
      webSocketFactory: () => {
        const socket = new FakeWebSocket();
        sockets.push(socket);
        return socket as unknown as WebSocket;
      },
    });
    return { client, onStatus, onMessage };
  }

  function completeHandshake(socket: FakeWebSocket) {
    socket.receive({
      type: 'server.hello',
      protocolVersion: 1,
      serverVersion: '0.1.0',
      connectionId: crypto.randomUUID(),
      role: 'tv',
    });
  }

  it('connects, parses state messages and reports status', () => {
    const { client, onStatus, onMessage } = createClient();
    client.start();
    expect(onStatus).toHaveBeenLastCalledWith('connecting');
    sockets[0].open();
    expect(onStatus).toHaveBeenLastCalledWith('connecting');
    completeHandshake(sockets[0]);
    expect(onStatus).toHaveBeenLastCalledWith('connected');
    sockets[0].receive({ type: 'system.state', mode: 'tv' });
    expect(onMessage).toHaveBeenCalledWith({ type: 'system.state', mode: 'tv' });
    client.stop();
  });

  it('reconnects with exponential backoff', () => {
    const { client, onStatus } = createClient();
    client.start();
    sockets[0].disconnect();
    expect(onStatus).toHaveBeenLastCalledWith('reconnecting');
    vi.advanceTimersByTime(999);
    expect(sockets).toHaveLength(1);
    vi.advanceTimersByTime(1);
    expect(sockets).toHaveLength(2);
    sockets[1].disconnect();
    vi.advanceTimersByTime(1_999);
    expect(sockets).toHaveLength(2);
    vi.advanceTimersByTime(1);
    expect(sockets).toHaveLength(3);
    client.stop();
  });

  it('restores connected state only after a new hello following a Core restart', () => {
    const { client, onStatus } = createClient();
    client.start();
    sockets[0].open();
    completeHandshake(sockets[0]);
    expect(onStatus).toHaveBeenLastCalledWith('connected');

    sockets[0].disconnect();
    vi.advanceTimersByTime(1_000);
    sockets[1].open();
    expect(onStatus).toHaveBeenLastCalledWith('reconnecting');

    completeHandshake(sockets[1]);
    expect(onStatus).toHaveBeenLastCalledWith('connected');
    client.stop();
  });

  it('sends a PC mode request and resolves its matching result', async () => {
    const { client } = createClient();
    client.start();
    sockets[0].open();
    completeHandshake(sockets[0]);
    const promise = client.sendModeSwitch('pc');
    const command = JSON.parse(sockets[0].sent[0]);
    expect(command).toMatchObject({ action: 'system.switch_mode', params: { mode: 'pc' } });
    sockets[0].receive({ type: 'command.result', requestId: command.requestId, ok: true });
    await expect(promise).resolves.toBeUndefined();
    client.stop();
  });

  it('times out a mode switch after 12 seconds', async () => {
    const { client } = createClient();
    client.start();
    sockets[0].open();
    const promise = client.sendModeSwitch('pc');
    const assertion = expect(promise).rejects.toThrow('タイムアウト');
    await vi.advanceTimersByTimeAsync(12_000);
    await assertion;
    client.stop();
  });

  it('does not report connected before a valid TV hello', () => {
    const { client, onStatus } = createClient();
    client.start();
    sockets[0].open();
    sockets[0].receive({ type: 'system.state', mode: 'tv' });

    expect(onStatus).not.toHaveBeenCalledWith('connected');
    expect(sockets[0].close).toHaveBeenCalledWith(1002, 'invalid server handshake');
    client.stop();
  });

  it('drops an invalid payload after the handshake', () => {
    const { client, onMessage } = createClient();
    client.start();
    sockets[0].open();
    completeHandshake(sockets[0]);
    onMessage.mockClear();

    sockets[0].receive({ type: 'remote.command', requestId: 'bad', action: 'navigation.select' });

    expect(onMessage).not.toHaveBeenCalled();
    expect(sockets[0].close).not.toHaveBeenCalled();
    client.stop();
  });
});
