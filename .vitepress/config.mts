import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "deskapp",
  description: "桌面应用程序开发",
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: '首页', link: '/' },
      { text: '快速上手Electron应用', link: '/docs/study/' },
      { text: 'Electron工具包', link: '/markdown-examples' }
    ],

    sidebar: {
      '/docs/study/':[
        { text: '进程间通信', link: '/docs/study/进程间通信' },
        { text: '首页', link: '/' }
      ]
    },

    socialLinks: [
      { icon: 'github', link: 'https://gitee.com/wetspace/wet-deskapp' }
    ]
  }
})
