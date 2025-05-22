import type {App } from  'electron'
import { ipcMain,BrowserWindow,app } from 'electron'
import { createDialog } from '@wetspace/deskapp'
import path from 'node:path'

const loadURL = app.isPackaged ? path.join(__dirname,'dist-views','index.html'):'http://localhost:5173/'

export default (app:App,mainWindow:BrowserWindow)=>{
    console.log(app,mainWindow)
    // 可以将主进程的监听渲染进程的事件逻辑放在这里
    ipcMain.handle('create:dialog',(e,args)=>{
        console.log(args,createDialog,'createDialog')
        createDialog('browser',{
            parent:mainWindow,
            loadURL
        })
        return "处理后的值"
    })
}