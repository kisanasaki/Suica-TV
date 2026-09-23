import { useCallback, useEffect, useRef } from 'react';
import { ConnectionBadge } from './components/ConnectionBadge';
import { MENU_ITEMS } from './config/menu';
import { useKeyboardNavigation } from './hooks/useKeyboardNavigation';
import { useRemoteCommands } from './hooks/useRemoteCommands';
import { HomePage } from './pages/HomePage';
import { SettingsPage } from './pages/SettingsPage';
import { launchExternalUrl } from './services/launcher';
import type { CoreClient } from './services/coreClient';
import { AppStateProvider, useAppState } from './state/AppStateContext';
import type { MenuItemId, NavigationAction } from './state/types';

function TvApplication() {
  const { state, dispatch } = useAppState();
  const coreClientRef = useRef<CoreClient | null>(null);
  const settingsNavigationRef = useRef<((action: NavigationAction) => void) | null>(null);

  const showError = useCallback((message: string) => dispatch({ type: 'show-error', message }), [dispatch]);
  const handleNavigation = useCallback((action: NavigationAction) => {
    if (action === 'navigation.home') dispatch({ type: 'go-home' });
    else if (action === 'navigation.back') dispatch({ type: 'go-back' });
    else if (action !== 'navigation.select') dispatch({ type: 'navigate', direction: action });
  }, [dispatch]);

  const selectItem = useCallback(async (id: MenuItemId) => {
    const item = MENU_ITEMS.find((candidate) => candidate.id === id);
    if (!item) return;
    if (item.kind === 'external-url') {
      try { launchExternalUrl(item.target); } catch { showError('このURLは安全に開けません。'); }
      return;
    }
    if (item.kind === 'internal-page') {
      dispatch({ type: 'show-page', page: 'settings' });
      return;
    }
    if (state.connection !== 'connected' || state.modeSwitchPending) return;
    const coreClient = coreClientRef.current;
    if (!coreClient) {
      showError('Suica Coreに接続されていません。');
      return;
    }
    dispatch({ type: 'mode-switch-started' });
    try {
      await coreClient.sendModeSwitch('pc');
    } catch (error) {
      showError(error instanceof Error ? error.message : 'モード切り替えに失敗しました。');
    } finally {
      dispatch({ type: 'mode-switch-finished' });
    }
  }, [dispatch, showError, state.connection, state.modeSwitchPending]);

  const processAction = useCallback((action: NavigationAction) => {
    if (state.page === 'settings') {
      settingsNavigationRef.current?.(action);
      return;
    }
    if (action === 'navigation.select') {
      if (state.page === 'home') void selectItem(state.selectedId);
      return;
    }
    handleNavigation(action);
  }, [handleNavigation, selectItem, state.page, state.selectedId]);

  const coreClient = useRemoteCommands({
    onStatus: useCallback((status) => dispatch({ type: 'connection-changed', status }), [dispatch]),
    onNavigation: processAction,
    onMode: useCallback((mode) => dispatch({ type: 'mode-changed', mode }), [dispatch]),
    onError: showError,
  });

  useEffect(() => {
    coreClientRef.current = coreClient;
    return () => {
      if (coreClientRef.current === coreClient) coreClientRef.current = null;
    };
  }, [coreClient]);

  useKeyboardNavigation(processAction);

  useEffect(() => {
    if (!state.lastError) return;
    const timer = window.setTimeout(() => dispatch({ type: 'clear-error' }), 5_000);
    return () => window.clearTimeout(timer);
  }, [dispatch, state.lastError]);

  return (
    <main className="tv-shell">
      <header className="top-bar">
        <div className="brand">
          <span className="brand__mark" aria-hidden="true">S</span>
          <div><p>HOME ENTERTAINMENT</p><h1>Suica TV</h1></div>
        </div>
        {state.page === 'home' && <ConnectionBadge status={state.connection} />}
      </header>

      {state.lastError && <div className="error-toast" role="alert">{state.lastError}</div>}

      {state.page === 'home' ? (
        <HomePage
          selectedId={state.selectedId}
          coreConnected={state.connection === 'connected'}
          modeSwitchPending={state.modeSwitchPending}
          onFocusItem={(id) => dispatch({ type: 'select-item', id })}
          onSelectItem={(id) => void selectItem(id)}
        />
      ) : (
        <SettingsPage
          connection={state.connection}
          mode={state.mode}
          onBack={() => dispatch({ type: 'go-back' })}
          registerNavigationHandler={(handler) => { settingsNavigationRef.current = handler; }}
        />
      )}

      <footer><span>矢印キーで移動</span><span>Enterで決定</span><span>Escで戻る</span></footer>
    </main>
  );
}

export function App() {
  return <AppStateProvider><TvApplication /></AppStateProvider>;
}
