import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // 排除 Tauri Rust 编译目录，避免 .pdb 文件被 cargo 锁定时 Vite 报 EBUSY
      ignored: ['**/src-tauri/**'],
    },
  },
})
