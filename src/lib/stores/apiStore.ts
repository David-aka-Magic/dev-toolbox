import { writable, get } from 'svelte/store';
import { browser } from '$app/environment';

export interface KeyValue {
  key: string;
  value: string;
  enabled: boolean;
}

export interface ApiRequest {
  id: string;
  name: string;
  method: string;
  url: string;
  params: KeyValue[];
  headers: KeyValue[];
  body: string;
  bodyType: 'none' | 'json' | 'text' | 'form';
}

export interface ApiResponse {
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: string;
  time: number;
  size: number;
  error?: boolean;
}

export interface Collection {
  id: string;
  name: string;
  requests: ApiRequest[];
  expanded: boolean;
}

interface ApiState {
  activeRequest: ApiRequest | null;
  response: ApiResponse | null;
  collections: Collection[];
  history: ApiRequest[];
}

function generateId(): string {
  return Math.random().toString(36).substring(2, 15);
}

function createDefaultRequest(): ApiRequest {
  return {
    id: generateId(),
    name: '',
    method: 'GET',
    url: '',
    params: [{ key: '', value: '', enabled: true }],
    headers: [{ key: '', value: '', enabled: true }],
    body: '',
    bodyType: 'json',
  };
}

function getMaxHistoryItems(): number {
  if (!browser) return 50;
  try {
    const stored = localStorage.getItem('app-settings');
    if (stored) {
      const settings = JSON.parse(stored);
      return settings.apiMaxHistoryItems || 50;
    }
  } catch (e) {}
  return 50;
}

function loadState(): ApiState {
  if (!browser) {
    return {
      activeRequest: createDefaultRequest(),
      response: null,
      collections: [],
      history: [],
    };
  }

  try {
    const saved = localStorage.getItem('apiTesterState');
    if (saved) {
      const parsed = JSON.parse(saved);
      return {
        activeRequest: parsed.activeRequest || createDefaultRequest(),
        response: null,
        collections: parsed.collections || [],
        history: parsed.history || [],
      };
    }
  } catch (e) {
    console.error('Failed to load API tester state:', e);
  }

  return {
    activeRequest: createDefaultRequest(),
    response: null,
    collections: [],
    history: [],
  };
}

function saveState(state: ApiState) {
  if (!browser) return;
  
  const maxHistory = getMaxHistoryItems();
  
  try {
    localStorage.setItem('apiTesterState', JSON.stringify({
      activeRequest: state.activeRequest,
      collections: state.collections,
      history: state.history.slice(0, maxHistory),
    }));
  } catch (e) {
    console.error('Failed to save API tester state:', e);
  }
}

function createApiStore() {
  const { subscribe, set, update } = writable<ApiState>(loadState());

  let saveTimeout: ReturnType<typeof setTimeout>;
  
  function debouncedSave() {
    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => {
      saveState(get({ subscribe }));
    }, 500);
  }

  return {
    subscribe,

    createNewRequest() {
      update(state => {
        const newRequest = createDefaultRequest();
        return { ...state, activeRequest: newRequest, response: null };
      });
      debouncedSave();
    },

    setActiveRequest(request: ApiRequest) {
      update(state => ({ ...state, activeRequest: { ...request }, response: null }));
      debouncedSave();
    },

    updateActiveRequest(updates: Partial<ApiRequest>) {
      update(state => {
        if (!state.activeRequest) return state;
        return {
          ...state,
          activeRequest: { ...state.activeRequest, ...updates },
        };
      });
      debouncedSave();
    },

    setResponse(response: ApiResponse, saveToHistory: boolean = true) {
      update(state => {
        if (!saveToHistory || !state.activeRequest || !state.activeRequest.url) {
          return { ...state, response };
        }
        
        const maxHistory = getMaxHistoryItems();
        const historyEntry: ApiRequest = { ...state.activeRequest, id: generateId() };
        const history = [historyEntry, ...state.history.filter(h => h.url !== historyEntry.url)].slice(0, maxHistory);
        
        return { ...state, response, history };
      });
      debouncedSave();
    },

    createCollection(name: string) {
      update(state => {
        const newCollection: Collection = {
          id: generateId(),
          name,
          requests: [],
          expanded: true,
        };
        return { ...state, collections: [...state.collections, newCollection] };
      });
      debouncedSave();
    },

    deleteCollection(collectionId: string) {
      update(state => ({
        ...state,
        collections: state.collections.filter(c => c.id !== collectionId),
      }));
      debouncedSave();
    },

    renameCollection(collectionId: string, newName: string) {
      update(state => ({
        ...state,
        collections: state.collections.map(c =>
          c.id === collectionId ? { ...c, name: newName } : c
        ),
      }));
      debouncedSave();
    },

    toggleCollection(collectionId: string) {
      update(state => ({
        ...state,
        collections: state.collections.map(c =>
          c.id === collectionId ? { ...c, expanded: !c.expanded } : c
        ),
      }));
      debouncedSave();
    },

    saveRequestToCollection(collectionId: string, request: ApiRequest) {
      update(state => ({
        ...state,
        collections: state.collections.map(c =>
          c.id === collectionId
            ? { ...c, requests: [...c.requests, { ...request, id: generateId() }] }
            : c
        ),
      }));
      debouncedSave();
    },

    deleteRequestFromCollection(collectionId: string, requestId: string) {
      update(state => ({
        ...state,
        collections: state.collections.map(c =>
          c.id === collectionId
            ? { ...c, requests: c.requests.filter(r => r.id !== requestId) }
            : c
        ),
      }));
      debouncedSave();
    },

    clearHistory() {
      update(state => ({ ...state, history: [] }));
      debouncedSave();
    },
  };
}

export const apiStore = createApiStore();