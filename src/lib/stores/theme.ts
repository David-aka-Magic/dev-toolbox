import { writable } from 'svelte/store';
import { browser } from '$app/environment';

type Theme = 'light' | 'dark';

const initialTheme: Theme = browser 
  ? (localStorage.getItem('theme') as Theme) || 'dark'
  : 'dark';

export const theme = writable<Theme>(initialTheme);

if (browser) {
  theme.subscribe((value) => {
    document.documentElement.setAttribute('data-theme', value);
    localStorage.setItem('theme', value);
  });
}

export function toggleTheme() {
  theme.update((current) => (current === 'dark' ? 'light' : 'dark'));
}