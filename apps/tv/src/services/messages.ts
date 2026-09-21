import type { DisplayMode, NavigationAction } from '../state/types';

export interface ServerHelloMessage {
  type: 'server.hello';
  protocolVersion: number;
  serverVersion: string;
  connectionId: string;
  role: 'tv';
}

export interface SystemStateMessage {
  type: 'system.state';
  mode: DisplayMode;
  transitioning?: boolean;
  targetMode?: DisplayMode;
  changedAt?: string;
}

export interface RemoteCommandMessage {
  type: 'remote.command';
  requestId: string;
  action: NavigationAction;
}

export interface CommandResultMessage {
  type: 'command.result';
  requestId: string;
  ok: boolean;
  error?: { code: string; message: string; retryable?: boolean };
}

export interface ErrorMessage {
  type: 'error';
  error: { code: string; message: string; retryable?: boolean };
}

export type ServerMessage =
  | ServerHelloMessage
  | SystemStateMessage
  | RemoteCommandMessage
  | CommandResultMessage
  | ErrorMessage
  | { type: 'server.shutdown'; retryAfterSeconds?: number };

export function parseServerMessage(value: string): ServerMessage | undefined {
  let candidate: unknown;
  try {
    candidate = JSON.parse(value);
  } catch {
    return undefined;
  }
  if (!candidate || typeof candidate !== 'object' || !('type' in candidate)) return undefined;
  const message = candidate as Record<string, unknown>;
  if (typeof message.type !== 'string') return undefined;

  switch (message.type) {
    case 'server.hello':
    case 'system.state':
    case 'remote.command':
    case 'command.result':
    case 'error':
    case 'server.shutdown':
      return candidate as ServerMessage;
    default:
      return undefined;
  }
}
