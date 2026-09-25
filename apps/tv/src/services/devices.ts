/**
 * ループバック限定の登録端末管理HTTP APIをTV画面から利用する。
 * APIエラー本文を利用者向けメッセージへ変換し、UIへHTTP詳細を漏らさない。
 */

export interface PairedDevice {
  deviceId: string;
  deviceName: string;
  createdAt: string | null;
}

interface DevicesResponse {
  devices: PairedDevice[];
}

async function errorMessage(response: Response): Promise<string> {
  try {
    const body = await response.json() as { error?: { message?: string } };
    return body.error?.message ?? `HTTP ${response.status}`;
  } catch {
    return `HTTP ${response.status}`;
  }
}

/** Coreと同じ端末から登録端末一覧を取得する。 */
export async function listPairedDevices(signal?: AbortSignal): Promise<PairedDevice[]> {
  const response = await fetch('/api/v1/devices', { signal });
  if (!response.ok) throw new Error(await errorMessage(response));
  const body = await response.json() as DevicesResponse;
  return body.devices;
}

/** 指定端末を失効し、再ペアリングが必要な状態にする。 */
export async function revokePairedDevice(deviceId: string): Promise<void> {
  const response = await fetch(`/api/v1/devices/${encodeURIComponent(deviceId)}`, {
    method: 'DELETE',
  });
  if (!response.ok) throw new Error(await errorMessage(response));
}
