/**
 * 物理キーボード操作をRemote APIと同じNavigationActionへ正規化する。
 * ブラウザ既定動作を抑止し、UI側に入力経路の違いを持ち込まない。
 */

import { useEffect } from 'react';
import type { NavigationAction } from '../state/types';

const KEY_ACTIONS: Readonly<Record<string, NavigationAction>> = {
  ArrowUp: 'navigation.up',
  ArrowDown: 'navigation.down',
  ArrowLeft: 'navigation.left',
  ArrowRight: 'navigation.right',
  Enter: 'navigation.select',
  Escape: 'navigation.back',
  Home: 'navigation.home',
};

export function useKeyboardNavigation(onAction: (action: NavigationAction) => void) {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const action = KEY_ACTIONS[event.key];
      if (!action || (event.key === 'Enter' && event.repeat)) return;
      event.preventDefault();
      onAction(action);
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [onAction]);
}
