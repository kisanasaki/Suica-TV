/**
 * Suica TV ReactアプリのVite開発・テスト構成を定義する。
 * 実行時のCore接続設定はアプリ側の環境変数から解決する。
 */

import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: { port: 5173 },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: './src/test/setup.ts',
    css: true,
  },
});
