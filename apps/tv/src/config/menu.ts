import type { MenuItem } from '../state/types';

export const MENU_ITEMS: readonly MenuItem[] = [
  {
    id: 'youtube',
    label: 'YouTube',
    description: '動画をテレビで楽しむ',
    icon: '▶',
    kind: 'external-url',
    target: 'https://www.youtube.com/tv',
    row: 0,
    column: 0,
  },
  {
    id: 'browser',
    label: 'Browser',
    description: 'ウェブを開く',
    icon: '◎',
    kind: 'external-url',
    target: 'https://www.google.com/',
    row: 0,
    column: 1,
  },
  {
    id: 'zaim',
    label: 'Zaim',
    description: '家計を確認する',
    icon: '¥',
    kind: 'external-url',
    target: 'https://zaim.net/user_session/new',
    row: 1,
    column: 0,
  },
  {
    id: 'pc-mode',
    label: 'PC Mode',
    description: 'デスクトップへ切り替える',
    icon: '▣',
    kind: 'system-command',
    target: 'pc',
    row: 1,
    column: 1,
  },
  {
    id: 'settings',
    label: 'Settings',
    description: '接続状態と設定を確認',
    icon: '⚙',
    kind: 'internal-page',
    target: 'settings',
    row: 2,
    column: 0,
  },
] as const;

export const INITIAL_MENU_ID = 'youtube' as const;
