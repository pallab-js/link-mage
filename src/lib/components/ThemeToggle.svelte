<script lang="ts">
  import { themeStore } from '$lib/stores';

  const isDark = $derived($themeStore === 'dark');
</script>

<!--
  Pill-shaped toggle: moon = dark mode  |  sun = light mode.
  Uses role="switch" + aria-checked for proper screen-reader semantics.
-->
<button
  class="theme-toggle"
  role="switch"
  aria-checked={!isDark}
  aria-label={isDark ? 'Switch to light mode' : 'Switch to dark mode'}
  title={isDark ? 'Light mode' : 'Dark mode'}
  onclick={() => themeStore.toggle()}
>
  <span class="track" class:light={!isDark}>
    <span class="thumb" class:light={!isDark}>
      {#if isDark}
        <!-- Moon icon -->
        <svg width="9" height="9" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
          <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
        </svg>
      {:else}
        <!-- Sun icon -->
        <svg width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" aria-hidden="true">
          <circle cx="12" cy="12" r="4"/>
          <line x1="12" y1="2"  x2="12" y2="5"/>
          <line x1="12" y1="19" x2="12" y2="22"/>
          <line x1="4.22" y1="4.22"   x2="6.34" y2="6.34"/>
          <line x1="17.66" y1="17.66" x2="19.78" y2="19.78"/>
          <line x1="2"  y1="12" x2="5"  y2="12"/>
          <line x1="19" y1="12" x2="22" y2="12"/>
          <line x1="4.22" y1="19.78" x2="6.34" y2="17.66"/>
          <line x1="17.66" y1="6.34" x2="19.78" y2="4.22"/>
        </svg>
      {/if}
    </span>
  </span>
  <span class="sr-only">{isDark ? 'Dark' : 'Light'} mode active</span>
</button>

<style>
  .theme-toggle {
    display: inline-flex;
    align-items: center;
    background: none;
    border: none;
    padding: 2px;
    cursor: pointer;
    border-radius: var(--radius-md);
    outline-offset: 2px;
  }
  .theme-toggle:focus-visible {
    outline: 2px solid var(--accent);
  }

  /* ── Track ─────────────────────────────────────────────────────────────── */
  .track {
    position: relative;
    width: 36px;
    height: 20px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 10px;
    transition: background 200ms ease, border-color 200ms ease;
  }
  .track.light {
    background: color-mix(in srgb, var(--accent) 18%, var(--bg-surface));
    border-color: var(--accent);
  }

  /* ── Thumb ─────────────────────────────────────────────────────────────── */
  .thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    background: var(--text-secondary);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--bg-base);
    transition: transform 200ms cubic-bezier(.4, 0, .2, 1), background 200ms ease;
  }
  .thumb.light {
    transform: translateX(16px);
    background: var(--accent);
  }

  .sr-only {
    position: absolute; width: 1px; height: 1px;
    padding: 0; margin: -1px; overflow: hidden;
    clip: rect(0,0,0,0); white-space: nowrap; border-width: 0;
  }
</style>
