import { contextBridge, ipcRenderer } from 'electron'
import type {
    IpcRendererEvent
} from 'electron'

export type ElectronEventsCallback = (event: IpcRendererEvent, ...args: any[]) => void
export type ExposeIpcRendererCallBack =(...args:any[]) =>void
// 注册到ipcerender的事件函数
export const exposeIpcRendererEvent = () => {
    const registerEventMap: Record<string, any> = {}
    return {
        run: () => {
            contextBridge.exposeInMainWorld('electronEvents', registerEventMap)
        },
        invoke() {

        },
        send(channel: string) {
            registerEventMap[channel] = <T>(data:T)=>ipcRenderer.send(channel,data)
        },
        on(channel: string, callBack: ElectronEventsCallback = (_,args)=> args) {
            registerEventMap[channel] = (ipcRendererCallBack: ExposeIpcRendererCallBack) => {
                const runFn: ElectronEventsCallback = (event, ...args) => {
                    const res = callBack(event, ...args)
                    ipcRendererCallBack(res)
                }
                ipcRenderer.on(channel, runFn)
                return () => {
                    ipcRenderer.removeListener(channel, runFn)
                }
            }
        }
    }
}
