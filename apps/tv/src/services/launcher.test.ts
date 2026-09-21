import { describe, expect, it, vi } from 'vitest';
import { launchExternalUrl } from './launcher';

describe('launchExternalUrl', () => {
  it('opens an HTTPS URL', () => {
    const navigate = vi.fn();
    launchExternalUrl('https://example.com/path', navigate);
    expect(navigate).toHaveBeenCalledWith('https://example.com/path');
  });

  it('uses the standard YouTube URL expected by Chromium', async () => {
    const { MENU_ITEMS } = await import('../config/menu');
    expect(MENU_ITEMS.find((item) => item.id === 'youtube')?.target).toBe('https://www.youtube.com/');
  });

  it('rejects non-HTTPS URLs', () => {
    expect(() => launchExternalUrl('http://example.com', vi.fn())).toThrow('HTTPS');
    expect(() => launchExternalUrl('javascript:alert(1)', vi.fn())).toThrow('HTTPS');
  });
});
