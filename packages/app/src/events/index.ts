import { contextBridge, ipcRenderer } from 'electron'
import type {
    IpcRendererEvent
} from 'electron'

export type ElectronEventsCallback = (event: IpcRendererEvent, ...args: any[]) => void
export const registerPreloadEvent = () => {
    const registerEventMap: Record<string, any> = {}
    return {
        run: () => {
            contextBridge.exposeInMainWorld('electronEvents', registerEventMap)
        },
        invoke() {

        },
        send() {

        },
        on(channel: string, callBack: ElectronEventsCallback) {
            registerEventMap[channel] = (ipcRendererCallBack: ElectronEventsCallback) => {
                const runFn: ElectronEventsCallback = (event, ...args) => {
                    const res = callBack(event, ...args)
                    ipcRendererCallBack(event, res)
                }
                ipcRenderer.on(channel, runFn)
                return () => {
                    ipcRenderer.removeListener(channel, runFn)
                }
            }
        }
    }
}