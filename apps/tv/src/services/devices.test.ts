import { afterEach, describe, expect, it, vi } from 'vitest';
import { listPairedDevices, revokePairedDevice } from './devices';

afterEach(() => vi.unstubAllGlobals());

describe('paired device API', () => {
  it('loads registered devices', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify({
      devices: [{ deviceId: 'device-1', deviceName: 'Living room phone', createdAt: null }],
    }), { status: 200, headers: { 'Content-Type': 'application/json' } }));
    vi.stubGlobal('fetch', fetchMock);

    await expect(listPairedDevices()).resolves.toEqual([
      { deviceId: 'device-1', deviceName: 'Living room phone', createdAt: null },
    ]);
    expect(fetchMock).toHaveBeenCalledWith('/api/v1/devices', { signal: undefined });
  });

  it('revokes the selected device and surfaces API errors', async () => {
    const fetchMock = vi.fn()
      .mockResolvedValueOnce(new Response(null, { status: 204 }))
      .mockResolvedValueOnce(new Response(JSON.stringify({ error: { message: '登録端末が見つかりません。' } }), {
        status: 404,
        headers: { 'Content-Type': 'application/json' },
      }));
    vi.stubGlobal('fetch', fetchMock);

    await expect(revokePairedDevice('device/1')).resolves.toBeUndefined();
    expect(fetchMock).toHaveBeenNthCalledWith(1, '/api/v1/devices/device%2F1', { method: 'DELETE' });
    await expect(revokePairedDevice('missing')).rejects.toThrow('登録端末が見つかりません。');
  });
});
