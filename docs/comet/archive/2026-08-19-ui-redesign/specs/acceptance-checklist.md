# ui-redesign 验收清单

本文件汇总所有 spec 的 acceptance items，供 Verifier 单文件检查。

## Shell & Rebrand (BR)

- BR-1: Tauri windows[0].title === "灵镜 LINGJING"
- BR-2: TopBar.vue 第 1 行 .text-sm === "灵镜 LINGJING"
- BR-3: 项目根目录 grep "灵境" 0 命中
- BR-4: 项目根目录 grep "LingScape" 0 命中
- BR-5: tauri.conf.json bundle identifier 仍是 com.lingscape.app
- BR-6: package.json name 仍是 ling-scape

## Design Tokens (T)

- T-1: :root 包含 --bg / --surface / --surface-2 / --border / --border-strong
- T-2: :root 包含 --primary / --primary-hover / --primary-soft / --secondary
- T-3: :root 包含 --text / --text-2 / --text-3
- T-4: :root 包含 --success / --warning / --error / --info
- T-5: :root 包含 --dur-fast 150ms / --dur-base 280ms / --dur-slow 520ms
- T-6: :root 包含 --ease / --ease-out / --ease-spring
- T-7: :root 包含 --s1..s12 间距变量
- T-8: :root 包含 --r-sm / --r-md / --r-lg / --r-xl / --r-pill
- T-9: :root 包含 --sh-sm / --sh-md / --sh-win
- T-10: :root 包含 --font 含 Inter/YaHei/PingFang
- T-11: [data-theme="dark"] 覆盖 --bg / --surface / --primary / --text
- T-12: [data-theme="dark"] --primary 是 #818CF8
- T-13: 全项目 grep `#[0-9A-Fa-f]{6}` 0 命中
- T-14: 全项目 grep `transition.*[0-9]+ms` 0 命中

## Sidebar Nav (S)

- S-1: IconRail.vue 不再含 emoji
- S-2: 5 个 nav-item 各含 1 个 `<svg viewBox="0 0 24 24">`
- S-3: `.nav-indicator` 通过 --iy/--ih 定位
- S-4: 选中态移动时 indicator 用 transform: translateY()
- S-5: `@keyframes navRipple 380ms var(--ease-out)`
- S-6: `@keyframes navPop 280ms var(--ease-spring)`
- S-7: `@keyframes gearSpin .6s var(--ease)`
- S-8: `@keyframes heartBeat .72s var(--ease)`
- S-9: `@keyframes pulse 2s var(--ease) infinite`
- S-10: "我的"项含 .badge-dot
- S-11: useAudio.ts 含 playClick(freq)
- S-12: playClick 用 createOscillator + triangle + 120ms 衰减
- S-13: 5 频率常量 523.25/587.33/659.25/783.99/880.00 全部出现
- S-14: audioCtx.resume() 处理 suspended
- S-15: 音效 toggle 关闭时 playClick 立即 return
- S-16: 每个 nav-item role=button + tabindex=0 + aria-label
- S-17: Enter/Space 在 nav-item 上触发 fire

## Topbar (TB)

- TB-1: 搜索框 rounded-full / rounded-[var(--r-pill)]
- TB-2: 搜索框 focus-within primary border + primary-soft shadow
- TB-3: 搜索框内含"大家都在搜："+ "极光" 关键词 + "· 二次元 · 赛博城市"
- TB-4: 排序器容器 rounded-[var(--r-pill)] 背景 surface-2
- TB-5: 排序项选中态有 box-shadow: var(--sh-sm)
- TB-6: 排序项按下 scale(.94) 过渡 150ms
- TB-7: TopBar 含主题切换/最小化/最大化/关闭 4 按钮
- TB-8: 主题按钮切换 light/dark（非 frost-light/frost-dark）

## Wallpaper Grid (G)

- G-1: grid grid-template-columns: repeat(3,1fr)
- G-2: 卡片 hover translateY(-4px) + sh-md
- G-3: 卡片按下 translateY(-1px) scale(.99)
- G-4: .card.selected 含 primary border + primary-soft 2px shadow
- G-5: .thumb-bg hover scale(1.07) 过渡 280ms
- G-6: .badge 在 type==video||gif 时显示 "LIVE"
- G-7: .vol 显示 item.size 在缩略图右下
- G-8: .hover-acts opacity 0→1 + translateY(8→0) 过渡 150ms
- G-9: .ha-btn.preview 与 .ha-btn.apply 存在
- G-10: 入场 6 卡片 .card:nth-child(1..6) 60/120/180/240/300/360ms delay
- G-11: 入场用 @keyframes fadeUp transform/opacity
- G-12: .info 内 .t 标题与 .m 副文 存在
- G-13: 保留 activeCat 与 topbarSearch 过滤逻辑
- G-14: 响应式 @media (max-width: 680px) 降 2 列

## Detail Drawer (D)

- D-1: 抽屉 width 320 + border-left + 从右滑入
- D-2: 滑入用 transform/opacity 过渡 280ms var(--ease)
- D-3: .d-prev aspect 16:10
- D-4: @keyframes kenburns 12s var(--ease) infinite alternate
- D-5: .detail.open .d-prev .thumb-bg 触发 kenburns
- D-6: .wheel 50×50 圆形 right 12 bottom 12 backdrop-blur
- D-7: .wheel-svg viewBox `0 0 50 50` rotate -90deg
- D-8: stroke-dasharray 138.23
- D-9: @keyframes countdown 60s linear forwards
- D-10: .wheel-num .n 18px font-weight 700 显示秒数
- D-11: .wheel-num .l 9px "秒后取消"
- D-12: 倒计时归零 emit('close')
- D-13: 取消按钮 cancelAutoClose 停 timer 不 emit close
- D-14: h3 + by + meta 3 列 + tags + ghost 3 + apply + d-note 结构
- D-15: .btn-ghost.liked 红色 + 红边 + 浅红底
- D-16: [data-theme="dark"] .btn-ghost.liked rgba 浅红
- D-17: 卡片点击同步 thumb 渐变到大图
- D-18: 卡片 selected 态与抽屉状态联动

## Playback Bar (PB)

- PB-1: .playbar h-14 flex gap-3 border-t
- PB-2: .pb-thumb 46×32 圆角 6px
- PB-3: .pb-now .bar height 4px 圆角 4px 背景 border
- PB-4: @keyframes prog 18s linear infinite
- PB-5: scaleX(0)→scaleX(1) 流动条
- PB-6: .pb-now .bar i.paused animation-play-state paused
- PB-7: .pb-btn 32×32 圆角 50%
- PB-8: .pb-btn:hover scale(1.08)
- PB-9: .pb-btn.play 38×38 主色
- PB-10: 3 按钮文字 `⟨` / `▶ 或 ❚❚` / `⟩`
- PB-11: .pill font-size 12 padding 6 12 圆角 pill
- PB-12: .pill.imp primary 色 pill
- PB-13: 选中新卡片同步缩略图 + 标题
- PB-14: 播放键切换同步 paused class
- PB-15: cycleLoop 顺序 list/single/random
- PB-16: 未选 current 时 "未选择壁纸"

## Settings (SET)

- SET-1: src/components/SettingsView.vue 存在
- SET-2: App.vue 根据 nav active 切换
- SET-3: <h3>基本设置</h3> 4 个 set-row
- SET-4: 4 个 toggle：开机启动/双击隐藏/全屏静态/界面音效
- SET-5: 默认 on: 开机启动/全屏静态/界面音效；默认 off: 双击隐藏
- SET-6: 界面音效 toggle 切换 soundOn 全局状态
- SET-7: <h3>壁纸路径</h3> + path-ctrl input + 更改路径 btn
- SET-8: path-ctrl input readonly 背景 surface-2
- SET-9: <h3>控件设置</h3> + 音量滑块
- SET-10: 滑块 160×6 圆角 6px
- SET-11: 滑块 thumb 18×18 圆 50% #fff box-shadow md
- SET-12: 滑块 thumb `left: calc(${vol}% - 9px)`
- SET-13: 注脚 "设置项将在 Phase 3 接入真实系统设置"
- SET-14: .toggle 44×26 圆角 pill
- SET-15: .toggle::after 20×20 left 3 top 3
- SET-16: .toggle::after 过渡 left var(--dur-base) var(--ease)
- SET-17: .toggle.on 背景 primary + left 21px
- SET-18: toggle role=switch tabindex=0 aria-checked
- SET-19: Enter/Space 在 toggle 上触发 flip

## Favorites (FAV)

- FAV-1: src/components/FavoritesView.vue 存在
- FAV-2: h3 "我的收藏" font-size 18 font-weight 700
- FAV-3: 副文字 "已收藏 N 张壁纸 · 本地保存，无需登录"
- FAV-4: N 由 computed 自 catalog 过滤
- FAV-5: 卡片复用 .card .thumb .info .hover-acts 类
- FAV-6: 仅渲染 favorite===true 项
- FAV-7: 空态文案存在
- FAV-8: App.vue 根据 active 切换 main 区
- FAV-9: 卡片点击触发 select
- FAV-10: local/about 占位 "该页面将在 Phase 2/3 接入"

## Motion & Microinteractions (M)

- M-1: src/components/Toast.vue 存在
- M-2: src/composables/useToast.ts 存在
- M-3: Toast 浮窗 bottom 36px transform translate(-50%, 20px) 起步
- M-4: Toast 进场 transform: translate(-50%, 0); opacity: 1 280ms
- M-5: Toast 自动 2400ms 消失
- M-6: Toast role=status aria-live=polite
- M-7: 抽屉"设为壁纸"触发 Toast "壁纸已成功应用到桌面"
- M-8: ❤️ 切换触发 "已收藏 / 已取消收藏" Toast
- M-9: 音效 toggle 触发 Toast
- M-10: *:focus-visible 2px primary outline 2px offset
- M-11: @media (prefers-reduced-motion: reduce) 全覆盖规则
- M-12: reduce 下 Ken Burns 不启用
- M-13: reduce 下 fadeUp 不启用
- M-14: reduce 下 progress 流动不启用

## Build

- BUILD-1: pnpm build 干净通过 (vue-tsc + vite build exit 0)
- BUILD-2: 35+ modules 打包成功
