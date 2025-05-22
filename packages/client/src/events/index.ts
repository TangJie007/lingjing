export const addEventListener = (channel: string, callBack: any) => {
    let removetigger: any = null
    const removeFn = () => {
        window.queueMicrotask(() => {
            if (removetigger) {
                removetigger()
            }
        })
    }

    const run = () => {
        removetigger = window.electronEvents[channel](callBack)
    }

    return [run, removeFn]
}