import { BrowserWindow } from 'electron'
import type { BrowserWindowConstructorOptions } from 'electron';

export const createWindow = (config:BrowserWindowConstructorOptions = {
    width: 800,
    height: 600,
},options:{
    loadURL:string,
    openDevTool:boolean
})=>{
    const mainWindow = new BrowserWindow(config);
    mainWindow.loadURL(options.loadURL)
    if(options.openDevTool){
        mainWindow.webContents.openDevTools();
    }

    return mainWindow
}