import { INITIAL_MENU_ID, MENU_ITEMS } from '../config/menu';
import type { AppAction, MenuItemId, NavigationAction, TvAppState } from './types';

export const initialState: TvAppState = {
  page: 'home',
  selectedId: INITIAL_MENU_ID,
  connection: 'disconnected',
  mode: 'tv',
  modeSwitchPending: false,
};

function moveSelection(currentId: MenuItemId, direction: NavigationAction): MenuItemId {
  const current = MENU_ITEMS.find((item) => item.id === currentId);
  if (!current) return INITIAL_MENU_ID;

  const deltas: Partial<Record<NavigationAction, readonly [number, number]>> = {
    'navigation.up': [-1, 0],
    'navigation.down': [1, 0],
    'navigation.left': [0, -1],
    'navigation.right': [0, 1],
  };
  const delta = deltas[direction];

  if (!delta) return currentId;
  const [rowDelta, columnDelta] = delta;
  return (
    MENU_ITEMS.find(
      (item) => item.row === current.row + rowDelta && item.column === current.column + columnDelta,
    )?.id ?? currentId
  );
}

export function appReducer(state: TvAppState, action: AppAction): TvAppState {
  switch (action.type) {
    case 'navigate':
      if (state.page !== 'home') return state;
      return { ...state, selectedId: moveSelection(state.selectedId, action.direction) };
    case 'select-item':
      return { ...state, selectedId: action.id };
    case 'show-page':
      return { ...state, page: action.page };
    case 'go-home':
      return { ...state, page: 'home', selectedId: INITIAL_MENU_ID };
    case 'go-back':
      return state.page === 'home' ? state : { ...state, page: 'home' };
    case 'connection-changed':
      return { ...state, connection: action.status };
    case 'mode-changed':
      return { ...state, mode: action.mode };
    case 'mode-switch-started':
      return { ...state, modeSwitchPending: true, lastError: undefined };
    case 'mode-switch-finished':
      return { ...state, modeSwitchPending: false };
    case 'show-error':
      return { ...state, lastError: action.message, modeSwitchPending: false };
    case 'clear-error': {
      const { lastError: _lastError, ...rest } = state;
      return rest;
    }
  }
}
