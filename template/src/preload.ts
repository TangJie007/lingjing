// 使用预加载脚本，在主进程中运行，用于监听渲染进程的事件
import { exposeIpcRendererEvent } from '@wetspace/deskapp'

const { run, on, send } = exposeIpcRendererEvent()
// 渲染进程向主进程发送消息
send('set:title')
// 渲染进程监听主进程消息
on('privews:files')
// 最终执行
run()