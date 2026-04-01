<script lang="ts">
  import '../app.css';
  import { themeStore, statusStore, settingsOpenStore } from '$lib/stores';
  import ThemeToggle   from '$lib/components/ThemeToggle.svelte';
  import Settings      from '$lib/components/Settings.svelte';

  let { children } = $props();

  // Sync the theme attribute to <html> so getComputedStyle() reads the
  // correct CSS variable values (needed by Graph.svelte's buildStylesheet).
  $effect(() => {
    document.documentElement.setAttribute('data-theme', $themeStore);
  });
</script>

<!-- Keep data-theme on the shell div as well (CSS variable inheritance). -->
<div class="app-shell" data-theme={$themeStore}>

  <!-- Top bar -->
  <header class="topbar">
    <div class="topbar-left">
      <span class="topbar-title">Link Mage</span>
      <span class="topbar-sep">—</span>
      <span class="topbar-subtitle">OSINT Link Analysis</span>
    </div>
    <div class="topbar-right">
      {#if $statusStore.kind !== 'idle'}
        <div class="status-pill status-{$statusStore.kind}" role="status">
          {$statusStore.text}
        </div>
      {/if}
      <ThemeToggle />
      <button
        class="topbar-icon-btn"
        onclick={() => settingsOpenStore.update(v => !v)}
        title="Settings"
        aria-label="Open settings"
      >⚙</button>
    </div>
  </header>

  <!-- Body: sidebar + page content -->
  <div class="app-body">
    {@render children()}
  </div>

</div>

<!-- Settings panel — slides in from the right -->
<Settings />

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-base);
    color: var(--text-primary);
  }

  /* ── Top bar ───────────────────────────────────────────────────────────── */
  .topbar {
    height: var(--topbar-height);
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    flex-shrink: 0;
    /* native window drag region on macOS */
    -webkit-app-region: drag;
    user-select: none;
  }

  /* allow button clicks inside draggable region */
  .topbar-right {
    -webkit-app-region: no-drag;
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .topbar-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .topbar-title {
    font-size: 13px;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: .01em;
  }

  .topbar-sep {
    color: var(--text-muted);
    font-size: 11px;
  }

  .topbar-subtitle {
    font-size: 11px;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: .06em;
  }

  /* ── Status pill ───────────────────────────────────────────────────────── */
  .status-pill {
    font-size: 11px;
    padding: 3px 10px;
    border-radius: 999px;
    border: 1px solid transparent;
  }

  .status-success {
    background: color-mix(in srgb, var(--accent-success) 15%, transparent);
    border-color: var(--accent-success);
    color: var(--accent-success);
  }

  .status-error {
    background: color-mix(in srgb, var(--accent-danger) 15%, transparent);
    border-color: var(--accent-danger);
    color: var(--accent-danger);
  }

  .status-info {
    background: color-mix(in srgb, var(--accent) 15%, transparent);
    border-color: var(--accent);
    color: var(--accent);
  }

  /* ── Gear icon ─────────────────────────────────────────────────────────── */
  .topbar-icon-btn {
    -webkit-app-region: no-drag;
    background:   transparent;
    border:       none;
    color:        var(--text-secondary);
    cursor:       pointer;
    font-size:    1rem;
    line-height:  1;
    padding:      .25rem .3rem;
    border-radius: 4px;
    transition:   background 0.15s, color 0.15s;
  }
  .topbar-icon-btn:hover {
    background: var(--bg-hover);
    color:      var(--text-primary);
  }

  /* ── App body ──────────────────────────────────────────────────────────── */
  .app-body {
    flex: 1;
    display: flex;
    overflow: hidden;
    min-height: 0;
  }
</style>
