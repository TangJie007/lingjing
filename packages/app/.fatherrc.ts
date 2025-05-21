import { defineConfig } from 'father';
import path from 'path';

export default defineConfig({
  esm: {},
  cjs: {},
  alias: {
    '@': path.resolve(__dirname, './src'),
  },
  platform: 'node',
});