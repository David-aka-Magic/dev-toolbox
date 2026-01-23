import { writable } from 'svelte/store';

export interface MediaViewerState {
  isOpen: boolean;
  filePath: string | null;
  fileName: string | null;
  fileType: 'image' | 'video' | null;
}

function createMediaViewerStore() {
  const { subscribe, set, update } = writable<MediaViewerState>({
    isOpen: false,
    filePath: null,
    fileName: null,
    fileType: null
  });

  return {
    subscribe,
    openImage: (path: string, name: string) => {
      set({
        isOpen: true,
        filePath: path,
        fileName: name,
        fileType: 'image'
      });
    },
    openVideo: (path: string, name: string) => {
      set({
        isOpen: true,
        filePath: path,
        fileName: name,
        fileType: 'video'
      });
    },
    close: () => {
      set({
        isOpen: false,
        filePath: null,
        fileName: null,
        fileType: null
      });
    }
  };
}

export const mediaViewer = createMediaViewerStore();