import { writable, derived, get } from 'svelte/store';

interface FileTab {
    id: string;
    path: string;
    name: string;
    history: string[];
    historyIndex: number;
    refreshCounter?: number;
}

interface FileTabState {
    tabs: FileTab[];
    activeId: string;
}

function saveLastPath(path: string) {
    localStorage.setItem('lastFilePath', path);
}

function loadLastPath(): string {
    return localStorage.getItem('lastFilePath') || 'C:\\';
}

function createFileTabStore() {
    const initialPath = loadLastPath();
    const initialTab: FileTab = {
        id: crypto.randomUUID(),
        path: initialPath,
        name: initialPath.split(/[\\/]/).filter(p => p).pop() || 'Drive',
        history: [initialPath],
        historyIndex: 0,
        refreshCounter: 0
    };

    const { subscribe, set, update } = writable<FileTabState>({
        tabs: [initialTab],
        activeId: initialTab.id
    });

    return {
        subscribe,

        addTab: (path?: string) => update(state => {
            const newPath = path || loadLastPath();
            const newTab: FileTab = {
                id: crypto.randomUUID(),
                path: newPath,
                name: newPath.split(/[\\/]/).filter(p => p).pop() || 'Drive',
                history: [newPath],
                historyIndex: 0,
                refreshCounter: 0
            };
            return {
                tabs: [...state.tabs, newTab],
                activeId: newTab.id
            };
        }),

        closeTab: (id: string) => update(state => {
            const remaining = state.tabs.filter(t => t.id !== id);
            let newActiveId = state.activeId;
            if (id === state.activeId) {
                newActiveId = remaining[remaining.length - 1].id;
            }
            return { tabs: remaining, activeId: newActiveId };
        }),

        setActive: (id: string) => update(state => ({ ...state, activeId: id })),

        updateActivePath: (newPath: string) => update(state => {
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

        refreshCurrentPath: () => update(state => {
            return {
                ...state,
                tabs: state.tabs.map(tab => {
                    if (tab.id === state.activeId) {
                        return { 
                            ...tab,
                            refreshCounter: (tab.refreshCounter || 0) + 1
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

export const activeTab = derived(fileTabs, $store => 
    $store.tabs.find(t => t.id === $store.activeId)
);