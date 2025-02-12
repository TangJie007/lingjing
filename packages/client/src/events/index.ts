import type { ClientWindow,RenderderEventsCallback } from '../types'

const eventsMap:Record<string,Function[] | null> = {}

export const removeAllListeners = (channel:string)=>{
    (window as ClientWindow).electronEvents.removeAllListeners(channel);
}

export const removeListener = (channel:string,fn:Function)=>{
    if(!eventsMap[channel]) return
    eventsMap[channel] =  eventsMap[channel].filter(item=>{
        return item!==fn
    })
    if(eventsMap[channel].length === 0){
        eventsMap[channel] = null
        removeAllListeners(channel)
    }
}

export const addListener = (
    channel:string,
    fn:RenderderEventsCallback,
)=>{
    // removeAllListeners(channel)
    if(!eventsMap[channel]){
        eventsMap[channel] = [];
        (window as ClientWindow).electronEvents.addListener(channel,(event,params)=>{
            eventsMap[channel] && eventsMap[channel].forEach(itemFn=>{
                itemFn(event,params)
            })
        })
    }
    if(!eventsMap[channel].includes(fn)){
        eventsMap[channel].push(fn)
    }
}

