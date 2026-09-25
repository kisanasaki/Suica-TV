/**
 * 設定されたメニューをTV向けグリッドとして表示する。
 * Coreが必要な項目だけ接続状態に応じて無効化し、選択処理はAppへ返す。
 */

import { AppTile } from '../components/AppTile';
import { MENU_ITEMS } from '../config/menu';
import type { MenuItemId } from '../state/types';

interface Props {
  selectedId: MenuItemId;
  coreConnected: boolean;
  modeSwitchPending: boolean;
  onFocusItem: (id: MenuItemId) => void;
  onSelectItem: (id: MenuItemId) => void;
}

export function HomePage({
  selectedId,
  coreConnected,
  modeSwitchPending,
  onFocusItem,
  onSelectItem,
}: Props) {
  return (
    <section className="home-page" aria-label="アプリメニュー">
      <div className="menu-grid">
        {MENU_ITEMS.map((item) => (
          <AppTile
            key={item.id}
            item={item}
            selected={selectedId === item.id}
            disabled={item.id === 'pc-mode' && (!coreConnected || modeSwitchPending)}
            busy={item.id === 'pc-mode' && modeSwitchPending}
            onFocus={() => onFocusItem(item.id)}
            onSelect={() => onSelectItem(item.id)}
          />
        ))}
      </div>
    </section>
  );
}
