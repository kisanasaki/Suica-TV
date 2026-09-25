/**
 * ReactアプリケーションをブラウザDOMへマウントするエントリーポイント。
 * StrictModeによる開発時検査以外の初期化はAppへ委譲する。
 */

import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App';
import './styles/theme.css';
import './styles/global.css';

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
