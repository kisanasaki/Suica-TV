/**
 * キー入力からNavigationActionへの対応と、長押しEnterの多重決定防止を検証する。
 */

import { render } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { useKeyboardNavigation } from './useKeyboardNavigation';
import type { NavigationAction } from '../state/types';

function Harness({ onAction }: { onAction: (action: NavigationAction) => void }) {
  useKeyboardNavigation(onAction);
  return <div>Keyboard harness</div>;
}

describe('useKeyboardNavigation', () => {
  it('maps supported keys to navigation actions', async () => {
    const onAction = vi.fn();
    render(<Harness onAction={onAction} />);
    await userEvent.keyboard('{ArrowRight}{ArrowDown}{Enter}{Escape}{Home}');
    expect(onAction.mock.calls.map(([action]) => action)).toEqual([
      'navigation.right', 'navigation.down', 'navigation.select', 'navigation.back', 'navigation.home',
    ]);
  });

  it('ignores a repeated Enter key', () => {
    const onAction = vi.fn();
    render(<Harness onAction={onAction} />);
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', repeat: true }));
    expect(onAction).not.toHaveBeenCalled();
  });
});
