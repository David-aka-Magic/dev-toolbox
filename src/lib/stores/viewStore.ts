import { writable } from 'svelte/store';

export const currentView = writable<string>('terminal');

export const sidebarVisible = writable<boolean>(true);