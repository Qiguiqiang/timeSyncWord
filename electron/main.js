const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const { createServer } = require('../server/index');

let mainWindow = null;
let server = null;

const MAX_RETRIES = 5;
const RETRY_DELAY = 2000;

async function startApp(retries = MAX_RETRIES) {
  try {
    server = await createServer();
  } catch (err) {
    if (err.code === 'EADDRINUSE' && retries > 0) {
      console.log(`Port in use, retrying in ${RETRY_DELAY}ms... (${retries} left)`);
      await new Promise(r => setTimeout(r, RETRY_DELAY));
      return startApp(retries - 1);
    }
    throw err;
  }

  mainWindow = new BrowserWindow({
    width: 800,
    height: 500,
    minWidth: 600,
    minHeight: 400,
    frame: false,
    backgroundColor: '#0a0e1a',
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false
    }
  });

  mainWindow.loadURL(`http://localhost:${server.port}`);

  mainWindow.on('closed', () => {
    mainWindow = null;
  });
}

ipcMain.on('window:minimize', () => mainWindow?.minimize());
ipcMain.on('window:maximize', () => {
  if (mainWindow?.isMaximized()) mainWindow.unmaximize();
  else mainWindow?.maximize();
});
ipcMain.on('window:close', () => mainWindow?.close());

app.whenReady().then(startApp);

app.on('window-all-closed', () => {
  if (server) server.close();
  app.quit();
});

app.on('activate', () => {
  if (mainWindow === null) startApp();
});