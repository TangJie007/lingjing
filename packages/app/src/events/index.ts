import { contextBridge, ipcRenderer } from 'electron'
import type { BrowserWindow,IpcRendererEvent } from 'electron'

export type RendererRegisterType = 'invoke' | 'send'
export type electronEventsOptions = {
    once?:boolean,
    signalId:string
}
export type RenderderEventsCallback = (event:IpcRendererEvent, ...args: any[])=>void

// export const renderderEvents = (options?:any)=>{
//     const addListenerFnMaps:Record<string,RenderderEventsCallback | null> = {}
//     contextBridge.exposeInMainWorld('electronEvents',{
//         ...options,
//         addListener(channel:string,callBack:RenderderEventsCallback){
//             ipcRenderer.addListener(channel,callBack)
//             addListenerFnMaps[channel] = callBack
//         },
//         removeListener(channel:string,isAll:boolean){
//             if(isAll){
//                 ipcRenderer.removeAllListeners(channel)
//             }else{
//                 if(addListenerFnMaps[channel]){
//                     ipcRenderer.removeListener(channel,addListenerFnMaps[channel])
//                     addListenerFnMaps[channel] = null
//                 }
//             }
//         }
//     })
// }

export class RenderderEvents {
    // private eventsMap:Record<string,Record<string,RenderderEventsCallback>> = {}
    // private eventRun:Record<string,RenderderEventsCallback> = {}
    // private addListenerFn :any
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
        // addListener:(type:string,callBack:RenderderEventsCallback,options:electronEventsOptions)=>{
        //     if(!this.eventsMap[type]){
        //         this.eventsMap[type] = {}
        //     }
        //     this.eventsMap[type][options.signalId] = callBack
        //     if(!this.eventRun[type]){
        //         this.eventRun[type] = (event:IpcRendererEvent, ...args: any[])=>{
        //             Object.values(this.eventsMap[type]).forEach(itemFn=>{
        //                 itemFn(event,args)
        //             })
        //         }
        //         ipcRenderer.addListener(type,this.eventRun[type])
        //     }
        //     // console.dir(callBack)
        //     // this.addListenerFn = callBack
        //     //ipcRenderer.addListener(type,callBack)
        // },
        // removeListener:(type:string,callBack:RenderderEventsCallback)=>{

        //     // console.log(callBack ===this.addListenerFn )
        //     // ipcRenderer.removeListener(type,callBack)
        // },
        removeAllListeners:(channel?:string)=>{
            console.log(channel,'asdfasdfsafd')
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