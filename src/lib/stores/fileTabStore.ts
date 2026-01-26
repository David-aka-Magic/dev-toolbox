import { writable, derived, get } from 'svelte/store';
import { browser } from '$app/environment';

export interface FileTab {
    id: string;
    name: string;
    path: string;
    history: string[];
    historyIndex: number;
}

// Get initial path from settings
function getInitialPath(): string {
    if (!browser) return 'C:\\';
    
    try {
        const stored = localStorage.getItem('app-settings');
        if (stored) {
            const settings = JSON.parse(stored);
            
            // If remember last path is enabled and we have a last path
            if (settings.fileRememberLastPath && settings.fileLastPath) {
                return settings.fileLastPath;
            }
            
            // Otherwise use default start path
            if (settings.fileDefaultStartPath) {
                return settings.fileDefaultStartPath;
            }
        }
    } catch (e) {
        console.error('Failed to load initial path from settings:', e);
    }
    
    return 'C:\\';
}

function createInitialTab(): FileTab {
    const initialPath = getInitialPath();
    return {
        id: crypto.randomUUID(),
        name: initialPath.split(/[\\/]/).filter(p => p).pop() || 'Drive',
        path: initialPath,
        history: [initialPath],
        historyIndex: 0
    };
}

const initialTab = createInitialTab();

function createFileTabStore() {
    const { subscribe, update } = writable<{ tabs: FileTab[], activeId: string }>({
        tabs: [initialTab],
        activeId: initialTab.id
    });

    // Helper to save last path to settings
    function saveLastPath(path: string) {
        if (!browser) return;
        
        try {
            const stored = localStorage.getItem('app-settings');
            if (stored) {
                const settings = JSON.parse(stored);
                if (settings.fileRememberLastPath) {
                    settings.fileLastPath = path;
                    localStorage.setItem('app-settings', JSON.stringify(settings));
                }
            }
        } catch (e) {
            console.error('Failed to save last path:', e);
        }
    }

    return {
        subscribe,
        
        addTab: (startPath: string = 'C:\\') => update(state => {
            const newTab: FileTab = {
                id: crypto.randomUUID(),
                name: startPath.split('\\').pop() || 'Drive',
                path: startPath,
                history: [startPath],
                historyIndex: 0
            };
            return { ...state, tabs: [...state.tabs, newTab], activeId: newTab.id };
        }),

        closeTab: (id: string) => update(state => {
            if (state.tabs.length === 1) return state;
            const remaining = state.tabs.filter(t => t.id !== id);
            let newActiveId = state.activeId;
            if (id === state.activeId) {
                newActiveId = remaining[remaining.length - 1].id;
            }
            return { tabs: remaining, activeId: newActiveId };
        }),

        setActive: (id: string) => update(state => ({ ...state, activeId: id })),

        updateActivePath: (newPath: string) => update(state => {
            // Save to settings
            saveLastPath(newPath);
            
            return {
                ...state,
                tabs: state.tabs.map(tab => {
                    if (tab.id === state.activeId) {
                        const newHistory = tab.history.slice(0, tab.historyIndex + 1);
                        newHistory.push(newPath);
                        return { 
                            ...tab, 
                            path: newPath, 
                            name: newPath.split(/[\\/]/).filter(p=>p).pop() || 'Drive', 
                            history: newHistory,
                            historyIndex: newHistory.length - 1
                        };
                    }
                    return tab;
                })
            };
        }),

        renameTab: (id: string, newName: string) => update(state => ({
            ...state,
            tabs: state.tabs.map(t => t.id === id ? { ...t, name: newName } : t)
        })),

        goBack: () => update(state => {
            return {
                ...state,
                tabs: state.tabs.map(tab => {
                    if (tab.id === state.activeId && tab.historyIndex > 0) {
                        const newIndex = tab.historyIndex - 1;
                        const newPath = tab.history[newIndex];
                        saveLastPath(newPath);
                        return {
                            ...tab,
                            historyIndex: newIndex,
                            path: newPath,
                            name: newPath.split(/[\\/]/).filter(p=>p).pop() || 'Drive'
                        };
                    }
                    return tab;
                })
            };
        }),

        goForward: () => update(state => {
            return {
                ...state,
                tabs: state.tabs.map(tab => {
                    if (tab.id === state.activeId && tab.historyIndex < tab.history.length - 1) {
                        const newIndex = tab.historyIndex + 1;
                        const newPath = tab.history[newIndex];
                        saveLastPath(newPath);
                        return {
                            ...tab,
                            historyIndex: newIndex,
                            path: newPath,
                            name: newPath.split(/[\\/]/).filter(p=>p).pop() || 'Drive'
                        };
                    }
                    return tab;
                })
            };
        }),

        canGoBack: () => {
            const state = get({ subscribe });
            const activeTab = state.tabs.find(t => t.id === state.activeId);
            return activeTab ? activeTab.historyIndex > 0 : false;
        },

        canGoForward: () => {
            const state = get({ subscribe });
            const activeTab = state.tabs.find(t => t.id === state.activeId);
            return activeTab ? activeTab.historyIndex < activeTab.history.length - 1 : false;
        }
    };
}

export const fileTabs = createFileTabStore();

// Derived store for active tab
export const activeTab = derived(fileTabs, $store => 
    $store.tabs.find(t => t.id === $store.activeId)
);