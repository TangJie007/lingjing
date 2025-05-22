import { app } from 'electron';
import path from 'node:path';
import started from 'electron-squirrel-startup';
import { createMainWindow } from '@wetspace/deskapp';
import mainEvents from './events'


// 处理在安装/卸载时在Windows上创建/删除快捷方式
if (started) { app.quit() }
const loadURL = app.isPackaged ? path.join(__dirname,'dist-views','index.html'):'http://localhost:5173/'
createMainWindow(loadURL,{
  webPreferences: {
    contextIsolation: true,
    // 加载预加载脚本
    preload: path.join(__dirname, 'preload.js'),
  }
})((appIns,browserWinIns)=>{
  // 主进程事件注册
  mainEvents(appIns,browserWinIns)
}) 