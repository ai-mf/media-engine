// Service worker for AIMF file handling in PWAs

const AIMF_FILES = ['.avid', '.aimg', '.aaud'];

self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);
  const isAIMF = AIMF_FILES.some(ext => url.pathname.endsWith(ext));
  
  if (isAIMF) {
    event.respondWith(
      fetch(event.request).then(response => {
        // Add AIMF metadata to response headers
        const modifiedHeaders = new Headers(response.headers);
        
        const contentType = url.pathname.endsWith('.avid') ? 'video/avid' :
                           url.pathname.endsWith('.aimg') ? 'image/aimg' :
                           'audio/aaud';
        
        modifiedHeaders.set('Content-Type', contentType);
        modifiedHeaders.set('X-AIMF-Version', '1.0');
        
        return new Response(response.body, {
          status: response.status,
          statusText: response.statusText,
          headers: modifiedHeaders
        });
      })
    );
  }
});

// Register as file handler (Chrome 102+)
if ('launchQueue' in self) {
  launchQueue.setConsumer(async (launchParams) => {
    if (!launchParams.files.length) return;
    
    for (const fileHandle of launchParams.files) {
      const file = await fileHandle.getFile();
      console.log('Opened AIMF file:', file.name);
      // Handle the file
    }
  });
}