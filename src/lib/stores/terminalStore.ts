import { writable, get } from 'svelte/store';

export type Terminal = {
  id: string;
  title: string;
  profile: string; // 'pwsh', 'cmd', 'wsl', 'git-bash'
};

// 1. Create the store
export const terminals = writable<Terminal[]>([
  { id: crypto.randomUUID(), title: 'Terminal 1', profile: 'pwsh' }
]);

// 2. EXPORT ALIAS
export { terminals as tabs }; 

export const activeTabId = writable<string>('');

// Initialize activeTabId if strictly empty
terminals.subscribe(all => {
  const current = get(activeTabId);
  if (all.length > 0 && !all.find(t => t.id === current)) {
     activeTabId.set(all[0].id);
  }
});

// --- HELPER ACTIONS ---

export function setActiveTerminal(id: string) {
  activeTabId.set(id);
}

export function createTerminal(profile: string = 'pwsh') {
  const newId = crypto.randomUUID();
  
  terminals.update(all => {
    const num = all.length + 1;
    // Pretty names for profiles
    let title = `Terminal ${num}`;
    if (profile === 'cmd') title = `CMD ${num}`;
    if (profile === 'git-bash') title = `Git ${num}`;
    if (profile === 'wsl') title = `WSL ${num}`;
    
    return [...all, { id: newId, title, profile }];
  });
  
  // Set as active immediately
  activeTabId.set(newId);
}

export function closeTerminal(id: string) {
  terminals.update(all => {
    // FIX: Allow closing the last terminal
    const remaining = all.filter(t => t.id !== id);
    
    // Switch active tab if needed
    if (id === get(activeTabId)) {
        if (remaining.length > 0) {
            activeTabId.set(remaining[remaining.length - 1].id);
        } else {
            activeTabId.set('');
        }
    }
    
    return remaining;
  });
}

export function renameTerminal(id: string, newTitle: string) {
  terminals.update(all => {
    return all.map(t => {
      if (t.id === id) {
        return { ...t, title: newTitle };
      }
      return t;
    });
  });
}