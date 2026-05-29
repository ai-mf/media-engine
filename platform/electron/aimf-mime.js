// Electron MIME type registration

const { app, shell, protocol } = require('electron');
const path = require('path');

// Register custom protocol for AIMF files
app.whenReady().then(() => {
  protocol.registerFileProtocol('aimf', (request, callback) => {
    const url = request.url.slice('aimf://'.length);
    const filePath = path.normalize(decodeURIComponent(url));
    callback({ path: filePath });
  });
});

// Set as default handler for .avid/.aimg/.aaud files
app.setAsDefaultProtocolClient('aimf');

// Handle file associations
const isAIMFFile = (filePath) => {
  const ext = path.extname(filePath).toLowerCase();
  return ['.avid', '.aimg', '.aaud'].includes(ext);
};

// Register with system
if (process.platform === 'win32') {
  // Windows registry
  const registry = require('winreg');
  // Add registry entries (see Windows section)
} else if (process.platform === 'darwin') {
  // macOS UTIs (see above)
  app.setAsDefaultProtocolClient('aimf');
} else if (process.platform === 'linux') {
  // Linux desktop entries (see MIME XML above)
}