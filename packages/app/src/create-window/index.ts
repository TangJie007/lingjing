import { app, BrowserWindow,dialog } from 'electron'
import type { BrowserWindowConstructorOptions,App} from 'electron';

export const createWindow = (loadURL:string,config:BrowserWindowConstructorOptions = {
    width: 800,
    height: 600,
})=>{
    const mainWindow = new BrowserWindow(config);
    mainWindow.loadURL(loadURL)
    if(!app.isPackaged){
        mainWindow.webContents.openDevTools()
    }
    return mainWindow
}

export const createMainWindow = (loadURL:string,config:BrowserWindowConstructorOptions)=>{
    let mainWindow:BrowserWindow | null = null
    let eventsFn:(app: App, mainWindow: BrowserWindow) => void
    app.on('ready',()=>{
        mainWindow = createWindow(loadURL,config)
        eventsFn && eventsFn(app,mainWindow)
    })
    // 窗口全部关闭时应用退出,让应用程序及其菜单栏保持活动状态，直到用户退出
    app.on('window-all-closed', () => {
        if (process.platform !== 'darwin') {
            app.quit();
        }
    });

    // 应用激活时
    app.on('activate',()=>{
        // 单击dock图标，没有其他窗口打开
        if (BrowserWindow.getAllWindows().length === 0) {
           mainWindow = createWindow(loadURL,config)
        }
    })
    return (callBack:(app: App, mainWindow: BrowserWindow) => void)=>{
        eventsFn = callBack
    }
}

export type  CreateDialogType = 'browser' | 'dialog'
export  const createDialog = (type:CreateDialogType = 'dialog',options:BrowserWindowConstructorOptions & {
    loadURL:string
})=>{
    if(type === 'browser'){
        const child = new BrowserWindow({ 
            modal:true,
            show:false,
            // frame:false,
            ...options
        })
        child.setMenuBarVisibility(false)
        child.loadURL(options.loadURL)
        child.once('ready-to-show', () => {
            child.show()
        })
    }else{
        dialog.showOpenDialog(options)
    }
}