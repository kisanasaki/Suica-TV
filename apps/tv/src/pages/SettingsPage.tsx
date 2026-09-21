import { ConnectionBadge } from '../components/ConnectionBadge';
import type { ConnectionStatus, DisplayMode } from '../state/types';

interface Props {
  connection: ConnectionStatus;
  mode: DisplayMode;
  onBack: () => void;
}

export function SettingsPage({ connection, mode, onBack }: Props) {
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
      <button type="button" className="back-button" onClick={onBack}>← ホームへ戻る</button>
    </section>
  );
}
