import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "deskapp",
  description: "桌面应用程序开发",
  ignoreDeadLinks:true,
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: '首页', link: '/' },
      { text: '快速上手Electron应用', link: '/docs/study/' },
      { text: 'Electron工具包', link: '/docs/tools/' }
    ],

    sidebar: {
      '/docs/study/':[
        { text: '进程间通信', link: '/docs/study/进程间通信' },
        { text: '首页', link: '/' }
      ],
      '/docs/tools/':[
        { text: '工具包简介',link:'/docs/tools/' },
        {
          text:'主程序端工具包',
          items:[

          ]
        },
        {
          text:'渲染端工具包',
          items:[]
        }
      ],
      // '/docs/client':[
      //   { text: '事件' }
      // ]
    },

    socialLinks: [
      { icon: 'github', link: 'https://gitee.com/wetspace/wet-deskapp' }
    ]
  }
})
