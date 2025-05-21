import type { IpcRendererEvent } from 'electron'

export type RenderderEventsCallback = (event: IpcRendererEvent, ...args: any[]) => void
export type electronEventsOptions = {
    once?: boolean,
    signalId?: string
}

export type ClientWindow = Window & {
    electronEvents: Record<string, any>
    // electronEvents:{
    //     addListener:(channel:string,fn:RenderderEventsCallback,options?:any)=> void,
    //     removeListener:(channel:string,id:string )=>void,
    //     removeAllListeners:(channel?:string)=>void
    // }
} & typeof globalThis