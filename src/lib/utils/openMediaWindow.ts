import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

export async function openMediaInNewWindow(filePath: string, fileName: string, type: 'image' | 'video') {
  try {
    // Create a unique label for the window
    const label = `media-viewer-${Date.now()}`;
    
    // Encode the data to pass to the new window
    const params = new URLSearchParams({
      filePath,
      fileName,
      type
    });
    
    // Create new window
    const webview = new WebviewWindow(label, {
      url: `/media-viewer?${params.toString()}`,
      title: fileName,
      width: 1200,
      height: 800,
      center: true,
      resizable: true,
      decorations: true,
      transparent: false
    });

    console.log('Creating media viewer window for:', fileName);

    // Optional: Wait for window to be ready
    webview.once('tauri://created', () => {
      console.log('Media viewer window created successfully');
    });

    // Handle errors
    webview.once('tauri://error', (e) => {
      console.error('Error creating media viewer window:', e);
    });
    
    return webview;
  } catch (error) {
    console.error('Failed to create window:', error);
    alert('Failed to open media viewer: ' + error);
    throw error;
  }
}