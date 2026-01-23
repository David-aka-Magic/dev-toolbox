/**
 * File Manager View Mode Store
 * Controls how files are displayed: grid, list, or details
 */

import { writable } from 'svelte/store';

export type ViewMode = 'grid' | 'list' | 'details';

export const viewMode = writable<ViewMode>('grid');

// Sort configuration
export type SortField = 'name' | 'size' | 'modified' | 'type';
export type SortDirection = 'asc' | 'desc';

export interface SortConfig {
  field: SortField;
  direction: SortDirection;
}

export const sortConfig = writable<SortConfig>({
  field: 'name',
  direction: 'asc'
});

/**
 * Toggle sort direction or change field
 */
export function toggleSort(field: SortField) {
  sortConfig.update(config => {
    if (config.field === field) {
      // Toggle direction
      return { field, direction: config.direction === 'asc' ? 'desc' : 'asc' };
    } else {
      // New field, default to ascending
      return { field, direction: 'asc' };
    }
  });
}

/**
 * Sort files based on current configuration
 */
export function sortFiles(files: any[], config: SortConfig): any[] {
  const sorted = [...files].sort((a, b) => {
    let comparison = 0;
    
    // Folders always come first
    if (a.is_dir !== b.is_dir) {
      return a.is_dir ? -1 : 1;
    }
    
    switch (config.field) {
      case 'name':
        comparison = a.name.localeCompare(b.name, undefined, { sensitivity: 'base' });
        break;
      case 'size':
        comparison = (a.size || 0) - (b.size || 0);
        break;
      case 'modified':
        comparison = (a.modified || 0) - (b.modified || 0);
        break;
      case 'type':
        const extA = a.name.split('.').pop() || '';
        const extB = b.name.split('.').pop() || '';
        comparison = extA.localeCompare(extB);
        break;
    }
    
    return config.direction === 'asc' ? comparison : -comparison;
  });
  
  return sorted;
}