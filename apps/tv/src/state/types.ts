/**
 * TV画面で共有する状態、メニュー、NavigationActionの型契約を定義する。
 * Wire上の文字列はRemote API v1と一致させる。
 */

export type PageId = 'home' | 'settings';
export type DisplayMode = 'tv' | 'pc';
export type ConnectionStatus =
  | 'disconnected'
  | 'connecting'
  | 'connected'
  | 'reconnecting';

export type NavigationAction =
  | 'navigation.up'
  | 'navigation.down'
  | 'navigation.left'
  | 'navigation.right'
  | 'navigation.select'
  | 'navigation.back'
  | 'navigation.home';

export type MenuItemId = 'youtube' | 'browser' | 'zaim' | 'pc-mode' | 'settings';

export interface MenuItem {
  id: MenuItemId;
  label: string;
  description: string;
  icon: string;
  kind: 'external-url' | 'system-command' | 'internal-page';
  target: string;
  row: number;
  column: number;
}

export interface TvAppState {
  page: PageId;
  selectedId: MenuItemId;
  connection: ConnectionStatus;
  mode: DisplayMode;
  modeSwitchPending: boolean;
  lastError?: string;
}

export type AppAction =
  | { type: 'navigate'; direction: NavigationAction }
  | { type: 'select-item'; id: MenuItemId }
  | { type: 'show-page'; page: PageId }
  | { type: 'go-home' }
  | { type: 'go-back' }
  | { type: 'connection-changed'; status: ConnectionStatus }
  | { type: 'mode-changed'; mode: DisplayMode }
  | { type: 'mode-switch-started' }
  | { type: 'mode-switch-finished' }
  | { type: 'show-error'; message: string }
  | { type: 'clear-error' };
