// See the Electron documentation for details on how to use preload scripts:
// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts
import { RenderderEvents } from '../../packages/app'

// rendererEvents()
// initRenderEvents()
const rdEvents = new RenderderEvents()
// rdEvents.registerEvent('')
rdEvents.init()