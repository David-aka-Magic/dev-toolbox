/**
 * Pure utility functions for file operations
 * No state, just helpers
 */

// Image extensions to show as thumbnails
export const imageExtensions = ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'svg', 'ico'];

// Video extensions - show icon, don't load
export const videoExtensions = ['mp4', 'webm', 'ogg', 'mov', 'avi', 'mkv', 'm4v'];

/**
 * Cross-platform path joining
 */
export function joinPath(base: string, name: string): string {
  base = base.replace(/[/\\]$/, '');
  const separator = base.includes('/') ? '/' : '\\';
  return `${base}${separator}${name}`;
}

/**
 * Check if a file is an image based on extension
 */
export function isImageFile(filename: string): boolean {
  const ext = filename.split('.').pop()?.toLowerCase();
  return ext ? imageExtensions.includes(ext) : false;
}

/**
 * Check if a file is a video based on extension
 */
export function isVideoFile(filename: string): boolean {
  const ext = filename.split('.').pop()?.toLowerCase();
  return ext ? videoExtensions.includes(ext) : false;
}

/**
 * Get MIME type for an image file
 */
export function getImageMimeType(filename: string): string {
  const ext = filename.split('.').pop()?.toLowerCase();
  return ext === 'svg' ? 'image/svg+xml' : `image/${ext}`;
}

/**
 * Calculate grid columns based on container width
 */
export function calculateGridColumns(containerWidth: number, itemWidth: number = 100): number {
  return Math.floor(containerWidth / itemWidth);
}