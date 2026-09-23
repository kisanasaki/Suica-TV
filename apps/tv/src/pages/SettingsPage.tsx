import { useCallback, useEffect, useState } from 'react';
import { ConnectionBadge } from '../components/ConnectionBadge';
import { listPairedDevices, revokePairedDevice, type PairedDevice } from '../services/devices';
import type { ConnectionStatus, DisplayMode, NavigationAction } from '../state/types';

interface Props {
  connection: ConnectionStatus;
  mode: DisplayMode;
  onBack: () => void;
  registerNavigationHandler?: (handler: ((action: NavigationAction) => void) | null) => void;
}

export function SettingsPage({ connection, mode, onBack, registerNavigationHandler }: Props) {
  const [devices, setDevices] = useState<PairedDevice[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [confirmDeviceId, setConfirmDeviceId] = useState<string>();
  const [loading, setLoading] = useState(true);
  const [working, setWorking] = useState(false);
  const [error, setError] = useState<string>();

  useEffect(() => {
    const controller = new AbortController();
    void listPairedDevices(controller.signal)
      .then(setDevices)
      .catch((reason: unknown) => {
        if (!controller.signal.aborted) {
          setError(reason instanceof Error ? reason.message : '登録端末を取得できませんでした。');
        }
      })
      .finally(() => {
        if (!controller.signal.aborted) setLoading(false);
      });
    return () => controller.abort();
  }, []);

  const requestRevoke = useCallback(async (device: PairedDevice) => {
    if (working) return;
    if (confirmDeviceId !== device.deviceId) {
      setConfirmDeviceId(device.deviceId);
      setError(undefined);
      return;
    }
    setWorking(true);
    try {
      await revokePairedDevice(device.deviceId);
      setDevices((current) => current.filter((candidate) => candidate.deviceId !== device.deviceId));
      setSelectedIndex((current) => Math.max(0, Math.min(current, devices.length - 2)));
      setConfirmDeviceId(undefined);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : 'ペアリングを解除できませんでした。');
    } finally {
      setWorking(false);
    }
  }, [confirmDeviceId, devices.length, working]);

  const handleNavigation = useCallback((action: NavigationAction) => {
    if (action === 'navigation.back' || action === 'navigation.home') {
      if (confirmDeviceId) setConfirmDeviceId(undefined);
      else onBack();
      return;
    }
    if (action === 'navigation.up') {
      setSelectedIndex((current) => Math.max(0, current - 1));
      setConfirmDeviceId(undefined);
      return;
    }
    if (action === 'navigation.down') {
      setSelectedIndex((current) => Math.min(Math.max(0, devices.length - 1), current + 1));
      setConfirmDeviceId(undefined);
      return;
    }
    if (action === 'navigation.select' && devices[selectedIndex]) {
      void requestRevoke(devices[selectedIndex]);
    }
  }, [confirmDeviceId, devices, onBack, requestRevoke, selectedIndex]);

  useEffect(() => {
    registerNavigationHandler?.(handleNavigation);
    return () => registerNavigationHandler?.(null);
  }, [handleNavigation, registerNavigationHandler]);

  return (
    <section className="settings-page" aria-labelledby="settings-title">
      <div className="settings-page__heading">
        <div>
          <p className="eyebrow">SYSTEM</p>
          <h2 id="settings-title">Settings</h2>
        </div>
        <ConnectionBadge status={connection} />
      </div>
      <dl className="settings-list">
        <div><dt>現在のモード</dt><dd>{mode === 'tv' ? 'TV Mode' : 'PC Mode'}</dd></div>
        <div><dt>Suica Core</dt><dd>{connection === 'connected' ? '利用可能' : '再接続を待っています'}</dd></div>
        <div><dt>バージョン</dt><dd>Suica TV v0.1.0</dd></div>
      </dl>
      <div className="paired-devices" aria-labelledby="paired-devices-title">
        <h3 id="paired-devices-title">登録端末</h3>
        {loading && <p>読み込み中…</p>}
        {!loading && devices.length === 0 && <p>登録端末はありません。</p>}
        {error && <p className="settings-error" role="alert">{error}</p>}
        <div className="paired-devices__list">
          {devices.map((device, index) => {
            const confirming = confirmDeviceId === device.deviceId;
            return (
              <button
                type="button"
                key={device.deviceId}
                className={`device-button${selectedIndex === index ? ' device-button--selected' : ''}`}
                aria-current={selectedIndex === index ? 'true' : undefined}
                disabled={working}
                onFocus={() => setSelectedIndex(index)}
                onClick={() => void requestRevoke(device)}
              >
                <span><strong>{device.deviceName}</strong><small>{device.createdAt ? new Date(device.createdAt).toLocaleDateString('ja-JP') : '登録日時不明'}</small></span>
                <span>{confirming ? 'もう一度押して解除' : '解除'}</span>
              </button>
            );
          })}
        </div>
      </div>
      <button type="button" className="back-button" onClick={onBack}>← ホームへ戻る</button>
    </section>
  );
}
