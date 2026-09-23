import { parseServerMessage, type ServerMessage } from './messages';
import type { ConnectionStatus, DisplayMode } from '../state/types';

interface CoreClientOptions {
  url?: string;
  onStatus: (status: ConnectionStatus) => void;
  onMessage: (message: ServerMessage) => void;
  webSocketFactory?: (url: string) => WebSocket;
}

interface PendingRequest {
  resolve: () => void;
  reject: (error: Error) => void;
  timeout: number;
}

export function getCoreWebSocketUrl() {
  if (import.meta.env.VITE_CORE_WS_URL) return import.meta.env.VITE_CORE_WS_URL;
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  return `${protocol}//${window.location.host}/ws?role=tv&protocolVersion=1`;
}

export class CoreClient {
  private socket?: WebSocket;
  private reconnectTimer?: number;
  private reconnectAttempt = 0;
  private stopped = true;
  private readonly pending = new Map<string, PendingRequest>();
  private readonly factory: (url: string) => WebSocket;

  constructor(private readonly options: CoreClientOptions) {
    this.factory = options.webSocketFactory ?? ((url) => new WebSocket(url));
  }

  start() {
    if (!this.stopped) return;
    this.stopped = false;
    this.connect();
  }

  stop() {
    this.stopped = true;
    if (this.reconnectTimer !== undefined) window.clearTimeout(this.reconnectTimer);
    this.socket?.close(1000, 'TV application stopped');
    this.socket = undefined;
    for (const request of this.pending.values()) {
      window.clearTimeout(request.timeout);
      request.reject(new Error('Suica Coreとの接続が終了しました。'));
    }
    this.pending.clear();
    this.options.onStatus('disconnected');
  }

  sendModeSwitch(mode: DisplayMode) {
    if (!this.socket || this.socket.readyState !== WebSocket.OPEN) {
      return Promise.reject(new Error('Suica Coreに接続されていません。'));
    }

    const requestId = crypto.randomUUID();
    const command = {
      type: 'remote.command',
      requestId,
      action: 'system.switch_mode',
      params: { mode },
    };

    return new Promise<void>((resolve, reject) => {
      const timeout = window.setTimeout(() => {
        this.pending.delete(requestId);
        reject(new Error('モード切り替えがタイムアウトしました。'));
      }, 12_000);
      this.pending.set(requestId, { resolve, reject, timeout });
      this.socket!.send(JSON.stringify(command));
    });
  }

  private connect() {
    if (this.stopped) return;
    this.options.onStatus(this.reconnectAttempt === 0 ? 'connecting' : 'reconnecting');
    const socket = this.factory(this.options.url ?? getCoreWebSocketUrl());
    this.socket = socket;
    let handshakeComplete = false;

    socket.addEventListener('open', () => {
      if (socket !== this.socket) return;
      this.options.onStatus(this.reconnectAttempt === 0 ? 'connecting' : 'reconnecting');
    });
    socket.addEventListener('message', (event) => {
      if (typeof event.data !== 'string') return;
      const message = parseServerMessage(event.data);
      if (!message) {
        console.warn('Suica Coreから不正または未対応のメッセージを受信しました。');
        return;
      }
      if (!handshakeComplete) {
        if (message.type !== 'server.hello' || message.protocolVersion !== 1 || message.role !== 'tv') {
          socket.close(1002, 'invalid server handshake');
          return;
        }
        handshakeComplete = true;
        this.reconnectAttempt = 0;
        this.options.onStatus('connected');
      }
      if (message.type === 'command.result') this.settleRequest(message);
      this.options.onMessage(message);
    });
    socket.addEventListener('error', () => socket.close());
    socket.addEventListener('close', () => {
      if (socket !== this.socket || this.stopped) return;
      this.socket = undefined;
      this.options.onStatus('reconnecting');
      const delays = [1_000, 2_000, 4_000, 8_000, 10_000];
      const delay = delays[Math.min(this.reconnectAttempt, delays.length - 1)];
      this.reconnectAttempt += 1;
      this.reconnectTimer = window.setTimeout(() => this.connect(), delay);
    });
  }

  private settleRequest(message: Extract<ServerMessage, { type: 'command.result' }>) {
    const request = this.pending.get(message.requestId);
    if (!request) return;
    window.clearTimeout(request.timeout);
    this.pending.delete(message.requestId);
    if (message.ok) request.resolve();
    else request.reject(new Error(message.error?.message ?? '操作を完了できませんでした。'));
  }
}
