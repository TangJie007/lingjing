// 使用预加载脚本，在主进程中运行，用于监听渲染进程的事件
import { registerPreloadEvent } from '@wetspace/deskapp'

const { run, on } = registerPreloadEvent()
on('privews:files', (event, args) => {
    console.log(args, event)
    if (args === '3') {
        return { value: '我是测试' }
    }
    return { value: '123' }
})
run()