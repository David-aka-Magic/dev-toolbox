import { writable, derived } from 'svelte/store';

export interface FileTab {
    id: string;
    name: string;
    path: string;
    history: string[];
    historyIndex: number;
}

const initialTab: FileTab = {
    id: crypto.randomUUID(),
    name: 'Home',
    path: 'C:\\', 
    history: ['C:\\'],
    historyIndex: 0
};

function createFileTabStore() {
    const { subscribe, update } = writable<{ tabs: FileTab[], activeId: string }>({
        tabs: [initialTab],
        activeId: initialTab.id
    });

    return {
        subscribe,
        
        // ... (Keep addTab, closeTab, setActive, updateActivePath, etc.) ...
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
            return {
                ...state,
                tabs: state.tabs.map(tab => {
                    if (tab.id === state.activeId) {
                        const newHistory = tab.history.slice(0, tab.historyIndex + 1);
                        newHistory.push(newPath);
                        return { 
                            ...tab, 
                            path: newPath, 
                            // Only auto-rename if the user hasn't manually renamed it? 
                            // For now, let's keep auto-rename behavior or you can remove this line:
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

        // ... (Keep goBack and goForward) ...
        goBack: () => update(state => {
             return {
                ...state,
                tabs: state.tabs.map(tab => {
                    if (tab.id === state.activeId && tab.historyIndex > 0) {
                        const newIndex = tab.historyIndex - 1;
                        return { 
                            ...tab, 
                            historyIndex: newIndex, 
                            path: tab.history[newIndex],
                            name: tab.history[newIndex].split(/[\\/]/).filter(p=>p).pop() || 'Drive'
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
                        return { 
                            ...tab, 
                            historyIndex: newIndex, 
                            path: tab.history[newIndex],
                            name: tab.history[newIndex].split(/[\\/]/).filter(p=>p).pop() || 'Drive'
                        };
                    }
                    return tab;
                })
            };
        })
    };
}

export const fileTabs = createFileTabStore();
export const activeTab = derived(fileTabs, $s => $s.tabs.find(t => t.id === $s.activeId));