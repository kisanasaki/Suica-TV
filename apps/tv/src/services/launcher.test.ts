/**
 * 外部URLのHTTPS制約、ホスト許可リスト、メニュー設定との整合性を検証する。
 */

import { describe, expect, it, vi } from 'vitest';
import { launchExternalUrl } from './launcher';

describe('launchExternalUrl', () => {
  it('opens an HTTPS URL', () => {
    const navigate = vi.fn();
    launchExternalUrl('https://zaim.net/user_session/new', navigate);
    expect(navigate).toHaveBeenCalledWith('https://zaim.net/user_session/new');
  });

  it('uses the current Zaim web login URL', async () => {
    const { MENU_ITEMS } = await import('../config/menu');
    expect(MENU_ITEMS.find((item) => item.id === 'zaim')?.target).toBe('https://zaim.net/user_session/new');
  });

  it('uses the television-optimized YouTube URL expected by kiosk Chromium', async () => {
    const { MENU_ITEMS } = await import('../config/menu');
    expect(MENU_ITEMS.find((item) => item.id === 'youtube')?.target).toBe('https://www.youtube.com/tv');
  });

  it('rejects non-HTTPS URLs', () => {
    expect(() => launchExternalUrl('http://example.com', vi.fn())).toThrow('HTTPS');
    expect(() => launchExternalUrl('javascript:alert(1)', vi.fn())).toThrow('HTTPS');
  });

  it('rejects HTTPS hosts that are not on the home menu allowlist', () => {
    expect(() => launchExternalUrl('https://example.com/', vi.fn())).toThrow('許可');
    expect(() => launchExternalUrl('https://zaim.net.evil.example/', vi.fn())).toThrow('許可');
  });
});
