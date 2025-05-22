import type {App, BrowserWindow} from  'electron'

export default (app:App,mainWindow:BrowserWindow)=>{
    console.log(app,mainWindow)
    // 可以将主进程的监听渲染进程的事件逻辑放在这里
}