/**
 * ホーム画面の1項目を、フォーカス・無効・処理中状態付きで表示する。
 * 選択後の処理は親へ通知し、この部品ではURLやシステム操作を実行しない。
 */

import { FocusFrame } from './FocusFrame';
import type { MenuItem } from '../state/types';

interface Props {
  item: MenuItem;
  selected: boolean;
  disabled?: boolean;
  busy?: boolean;
  onFocus: () => void;
  onSelect: () => void;
}

export function AppTile({ item, selected, disabled, busy, onFocus, onSelect }: Props) {
  return (
    <button
      type="button"
      className="app-tile"
      aria-current={selected ? 'true' : undefined}
      disabled={disabled}
      onFocus={onFocus}
      onMouseEnter={onFocus}
      onClick={onSelect}
      tabIndex={selected ? 0 : -1}
      data-testid={`tile-${item.id}`}
    >
      <FocusFrame active={selected}>
        <span className="app-tile__icon" aria-hidden="true">{busy ? '…' : item.icon}</span>
        <span className="app-tile__copy">
          <strong>{busy ? 'Switching…' : item.label}</strong>
          <small>{disabled ? 'Suica Coreへの接続が必要です' : item.description}</small>
        </span>
      </FocusFrame>
    </button>
  );
}
