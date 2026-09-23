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

export async function listPairedDevices(signal?: AbortSignal): Promise<PairedDevice[]> {
  const response = await fetch('/api/v1/devices', { signal });
  if (!response.ok) throw new Error(await errorMessage(response));
  const body = await response.json() as DevicesResponse;
  return body.devices;
}

export async function revokePairedDevice(deviceId: string): Promise<void> {
  const response = await fetch(`/api/v1/devices/${encodeURIComponent(deviceId)}`, {
    method: 'DELETE',
  });
  if (!response.ok) throw new Error(await errorMessage(response));
}
