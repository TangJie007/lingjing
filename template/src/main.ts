import { app, BrowserWindow,ipcMain } from 'electron';
import path from 'node:path';
import appConfig from './config'
import started from 'electron-squirrel-startup';
import { createWindow, MainEvents } from '@wetspace/deskapp'

// Handle creating/removing shortcuts on Windows when installing/uninstalling.
if (started) {
  app.quit();
}

let mainWin:BrowserWindow;
let mainEvents:MainEvents;
const createMainWindow = ()=>{
  mainWin = createWindow({
    ...appConfig.shape,
    webPreferences:{
      preload: path.join(__dirname, 'preload.js'),
    }
  },{
    loadURL:appConfig.loadFile,
    openDevTool:true
  })

  mainEvents = new MainEvents(mainWin)
  setInterval(()=>{
    mainEvents.send('privews:files','hhhh')
  },1000)
  setInterval(()=>{
    mainEvents.send('privews:imags','xxx')
  },2000)
}

// const createWindow = () => {
//   // Create the browser window.
//   const mainWindow = new BrowserWindow({
//     width: 800,
//     height: 600,
//     webPreferences: {
//       preload: path.join(__dirname, 'preload.js'),
//     },
//   });

//   // and load the index.html of the app.
//   if (MAIN_WINDOW_VITE_DEV_SERVER_URL) {
//     mainWindow.loadURL(MAIN_WINDOW_VITE_DEV_SERVER_URL);
//   } else {
//     mainWindow.loadFile(path.join(__dirname, `../renderer/${MAIN_WINDOW_VITE_NAME}/index.html`));
//   }

//   // Open the DevTools.
//   mainWindow.webContents.openDevTools();
// };

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.on('ready', ()=>{
  createMainWindow()
});

// Quit when all windows are closed, except on macOS. There, it's common
// for applications and their menu bar to stay active until the user quits
// explicitly with Cmd + Q.
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', () => {
  // On OS X it's common to re-create a window in the app when the
  // dock icon is clicked and there are no other windows open.
  if (BrowserWindow.getAllWindows().length === 0) {
    createMainWindow()
  }
});

// In this file you can include the rest of your app's specific main process
// code. You can also put them in separate files and import them here.
