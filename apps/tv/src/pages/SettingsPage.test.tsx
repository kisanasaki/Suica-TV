/**
 * 登録端末一覧、Remote操作による選択、二段階確認、解除後の表示更新を検証する。
 */

import { act, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { NavigationAction } from '../state/types';
import { listPairedDevices, revokePairedDevice } from '../services/devices';
import { SettingsPage } from './SettingsPage';

vi.mock('../services/devices', () => ({
  listPairedDevices: vi.fn(),
  revokePairedDevice: vi.fn(),
}));

describe('SettingsPage device management', () => {
  beforeEach(() => {
    vi.mocked(listPairedDevices).mockReset().mockResolvedValue([
      { deviceId: 'one', deviceName: 'First phone', createdAt: null },
      { deviceId: 'two', deviceName: 'Second phone', createdAt: '2026-09-24T00:00:00Z' },
    ]);
    vi.mocked(revokePairedDevice).mockReset().mockResolvedValue();
  });

  it('lists paired devices and requires confirmation before revocation', async () => {
    let navigate: ((action: NavigationAction) => void) | null = null;
    render(
      <SettingsPage
        connection="connected"
        mode="tv"
        onBack={vi.fn()}
        registerNavigationHandler={(handler) => { navigate = handler; }}
      />,
    );

    expect(await screen.findByText('First phone')).toBeInTheDocument();
    expect(screen.getByText('Second phone')).toBeInTheDocument();

    act(() => navigate?.('navigation.down'));
    act(() => navigate?.('navigation.select'));
    expect(screen.getByText('もう一度押して解除')).toBeInTheDocument();
    expect(revokePairedDevice).not.toHaveBeenCalled();

    act(() => navigate?.('navigation.select'));
    await waitFor(() => expect(revokePairedDevice).toHaveBeenCalledWith('two'));
    await waitFor(() => expect(screen.queryByText('Second phone')).not.toBeInTheDocument());
  });

  it('cancels confirmation before leaving settings', async () => {
    const onBack = vi.fn();
    let navigate: ((action: NavigationAction) => void) | null = null;
    render(
      <SettingsPage
        connection="connected"
        mode="tv"
        onBack={onBack}
        registerNavigationHandler={(handler) => { navigate = handler; }}
      />,
    );
    await screen.findByText('First phone');
    act(() => navigate?.('navigation.select'));
    act(() => navigate?.('navigation.back'));
    expect(onBack).not.toHaveBeenCalled();
    act(() => navigate?.('navigation.back'));
    expect(onBack).toHaveBeenCalledOnce();
  });
});
