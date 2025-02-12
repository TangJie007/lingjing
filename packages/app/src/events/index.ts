import { contextBridge, ipcRenderer } from 'electron'
import type { BrowserWindow,IpcRendererEvent } from 'electron'

export type RendererRegisterType = 'invoke' | 'send'
export type electronEventsOptions = {
    once?:boolean,
    signalId:string
}
export type RenderderEventsCallback = (event:IpcRendererEvent, ...args: any[])=>void

export class RenderderEvents {
    public addListenerFnMaps:Record<string,RenderderEventsCallback | null> = {}
    public electronEvents = {
        addListener:(channel:string,callBack:RenderderEventsCallback)=>{
            ipcRenderer.addListener(channel,callBack)
            this.addListenerFnMaps[channel] = callBack
        },
        removeListener:(channel:string)=>{
            const fn = this.addListenerFnMaps[channel]
            if(fn){
                ipcRenderer.removeListener(channel,fn)
                this.addListenerFnMaps[channel] = null
            }
        },
        removeAllListeners:(channel?:string)=>{
            ipcRenderer.removeAllListeners(channel)
        }
    }
    public init(){
        contextBridge.exposeInMainWorld('electronEvents',this.electronEvents)
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