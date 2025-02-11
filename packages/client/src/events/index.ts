import { contextBridge, ipcRenderer } from 'electron'
import type { BrowserWindow } from 'electron'

export type RendererRegisterType = 'invoke' | 'send'

export class RenderderEvents {
    private eventsMap:Record<string,any> = {}
    public electronEvents = {
        addListener:(type:string,callBack:any,options:any)=>{
            // console.log(this.eventsMap,this.eventsMap === callBack)
            console.dir(callBack)
            this.eventsMap = callBack
            ipcRenderer.addListener(type,callBack)
        },
        removeListener:(type:string,callBack:any)=>{
            ipcRenderer.removeListener(type,callBack)
        }
    }
    public init(){
        contextBridge.exposeInMainWorld('electronEvents',this.electronEvents)
    }
    
    public addListener(){
        // ipcRenderer.addListener()
    }
    public removeListener(){

    }

    public sender(){

    }
}

export class MainEvents {
    private win:BrowserWindow
    constructor(win:BrowserWindow){
        this.win = win
    }
    public send<T>(channel:string,p:T){
        this.win.webContents.send(channel,p)
    }
}