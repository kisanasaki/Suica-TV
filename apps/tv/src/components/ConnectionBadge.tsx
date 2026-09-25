/**
 * Coreとの接続状態を、TVから判別できるラベルと色で表示する。
 * 接続判定そのものはCoreClientとuseRemoteCommandsが担当する。
 */

import type { ConnectionStatus } from '../state/types';

const STATUS_LABELS: Record<ConnectionStatus, string> = {
  disconnected: '未接続',
  connecting: '接続中',
  connected: 'Core 接続済み',
  reconnecting: '再接続中',
};

export function ConnectionBadge({ status }: { status: ConnectionStatus }) {
  return (
    <div className={`connection-badge connection-badge--${status}`} role="status">
      <span className="connection-badge__dot" aria-hidden="true" />
      {STATUS_LABELS[status]}
    </div>
  );
}
