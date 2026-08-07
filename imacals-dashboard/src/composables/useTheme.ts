import { ref, computed, type Ref, type ComputedRef } from 'vue';

// Dashboard colour scheme. Persisted per-browser so the choice survives reloads, and re-applied
// before first paint (see initTheme, called from main.ts) so there is no light flash on a dark load.
export type Theme = 'light' | 'dark';

const STORAGE_KEY: string = 'theme';

// Module-level singleton: every component shares one source of truth.
const theme: Ref<Theme> = ref('light');

function apply(value: Theme): void {
  // Light is the default token set on :root — only dark needs the attribute.
  if (value === 'dark') {
    document.documentElement.setAttribute('data-theme', 'dark');
  } else {
    document.documentElement.removeAttribute('data-theme');
  }
}

function read(): Theme {
  const stored: string | null = localStorage.getItem(STORAGE_KEY);
  if (stored === 'dark' || stored === 'light') return stored;
  // No stored choice — follow the OS preference on first visit.
  const prefersDark: boolean =
    typeof window.matchMedia === 'function' &&
    window.matchMedia('(prefers-color-scheme: dark)').matches;
  return prefersDark ? 'dark' : 'light';
}

// Call once at boot, before mount, so the stored theme paints immediately.
export function initTheme(): void {
  theme.value = read();
  apply(theme.value);
}

export function useTheme(): {
  theme: Ref<Theme>;
  isDark: ComputedRef<boolean>;
  toggleTheme: () => void;
} {
  const isDark: ComputedRef<boolean> = computed<boolean>(() => theme.value === 'dark');

  function toggleTheme(): void {
    theme.value = isDark.value ? 'light' : 'dark';
    localStorage.setItem(STORAGE_KEY, theme.value);
    apply(theme.value);
  }

  return { theme, isDark, toggleTheme };
}
