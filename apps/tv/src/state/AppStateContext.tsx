import { createContext, useContext, useReducer, type Dispatch, type ReactNode } from 'react';
import { appReducer, initialState } from './appReducer';
import type { AppAction, TvAppState } from './types';

interface AppStateValue {
  state: TvAppState;
  dispatch: Dispatch<AppAction>;
}

const AppStateContext = createContext<AppStateValue | undefined>(undefined);

export function AppStateProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(appReducer, initialState);
  return <AppStateContext.Provider value={{ state, dispatch }}>{children}</AppStateContext.Provider>;
}

export function useAppState() {
  const context = useContext(AppStateContext);
  if (!context) throw new Error('useAppState must be used inside AppStateProvider');
  return context;
}
