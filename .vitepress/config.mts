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
      { text:'文档',link:'/docs/overview/'},
      { text: '快速上手Electron应用', link: '/docs/study/' },
    ],

    sidebar: {
      '/docs/study/':[
        { text: '进程间通信', link: '/docs/study/进程间通信' },
        { text: '首页', link: '/' }
      ],
      '/docs/overview/':[
        { text: '工具包简介',link:'/docs/overview/' },
        {
          text:'事件管理',
          items:[
            {text:'主进程端',link:'/docs/overview/主进程事件处理.md'},
            {text:'渲染进程端',link:'/docs/overview/渲染进程事件处理.md'}
          ]
        },
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
