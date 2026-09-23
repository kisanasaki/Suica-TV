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

type JsonObject = Record<string, unknown>;

const navigationActions: ReadonlySet<NavigationAction> = new Set([
  'navigation.up',
  'navigation.down',
  'navigation.left',
  'navigation.right',
  'navigation.select',
  'navigation.back',
  'navigation.home',
]);

function isObject(value: unknown): value is JsonObject {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function isDisplayMode(value: unknown): value is DisplayMode {
  return value === 'tv' || value === 'pc';
}

function isUuid(value: unknown): value is string {
  return typeof value === 'string'
    && /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(value);
}

function isOptionalString(value: unknown): value is string | undefined {
  return value === undefined || typeof value === 'string';
}

function isErrorBody(value: unknown): value is ErrorMessage['error'] {
  if (!isObject(value)) return false;
  return typeof value.code === 'string'
    && typeof value.message === 'string'
    && (value.retryable === undefined || typeof value.retryable === 'boolean');
}

export function parseServerMessage(value: string): ServerMessage | undefined {
  let candidate: unknown;
  try {
    candidate = JSON.parse(value);
  } catch {
    return undefined;
  }
  if (!isObject(candidate)) return undefined;
  const message = candidate;
  if (typeof message.type !== 'string') return undefined;

  switch (message.type) {
    case 'server.hello':
      return Number.isInteger(message.protocolVersion)
        && typeof message.serverVersion === 'string'
        && isUuid(message.connectionId)
        && message.role === 'tv'
        ? candidate as unknown as ServerHelloMessage
        : undefined;
    case 'system.state':
      return isDisplayMode(message.mode)
        && (message.transitioning === undefined || typeof message.transitioning === 'boolean')
        && (message.targetMode === undefined || isDisplayMode(message.targetMode))
        && isOptionalString(message.changedAt)
        ? candidate as unknown as SystemStateMessage
        : undefined;
    case 'remote.command':
      return isUuid(message.requestId)
        && typeof message.action === 'string'
        && navigationActions.has(message.action as NavigationAction)
        ? candidate as unknown as RemoteCommandMessage
        : undefined;
    case 'command.result':
      return isUuid(message.requestId)
        && typeof message.ok === 'boolean'
        && (message.error === undefined || isErrorBody(message.error))
        ? candidate as unknown as CommandResultMessage
        : undefined;
    case 'error':
      return isErrorBody(message.error) ? candidate as unknown as ErrorMessage : undefined;
    case 'server.shutdown':
      return message.retryAfterSeconds === undefined
        || (typeof message.retryAfterSeconds === 'number'
          && Number.isFinite(message.retryAfterSeconds)
          && message.retryAfterSeconds >= 0)
        ? candidate as unknown as Extract<ServerMessage, { type: 'server.shutdown' }>
        : undefined;
    default:
      return undefined;
  }
}
