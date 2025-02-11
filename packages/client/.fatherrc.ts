import { defineConfig } from 'father';
const path = require('path');

export default defineConfig({
  esm: {},
  cjs: {},
  alias: {
    '@': path.resolve(__dirname, './src'),
  },
  platform: 'node',
});