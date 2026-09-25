/**
 * リモコン操作で現在選択されている要素へ共通のフォーカス表現を付ける。
 */

import type { ReactNode } from 'react';

export function FocusFrame({ active, children }: { active: boolean; children: ReactNode }) {
  return <span className={active ? 'focus-frame focus-frame--active' : 'focus-frame'}>{children}</span>;
}
