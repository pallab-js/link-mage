<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type { TransformInfo, Machine } from '$lib/stores';

  // ── Props ─────────────────────────────────────────────────────────────────

  interface Props {
    /** Viewport X coordinate (pixels from left of window). */
    x: number;
    /** Viewport Y coordinate (pixels from top of window). */
    y: number;
    /** The node that was right-clicked. */
    nodeId:    string;
    nodeType:  string;
    nodeLabel: string;
    /** All transforms applicable to this node type, pre-filtered by caller. */
    transforms: TransformInfo[];
    /** All automation sequences. */
    machines: Machine[];
    /** Called when the user selects a transform. */
    onrun:   (nodeId: string, transformName: string) => void;
    /** Called when the user selects a machine. */
    onrunmachine: (nodeId: string, machineName: string) => void;
    /** Called when the menu should close (Escape / outside click / after run). */
    onclose: () => void;
  }

  let {
    x, y,
    nodeId, nodeType, nodeLabel,
    transforms,
    machines,
    onrun, onrunmachine, onclose,
  }: Props = $props();

  // ── Running state ─────────────────────────────────────────────────────────

  let runningItem: string | null = $state(null);

  async function handleRun(name: string) {
    if (runningItem) return;
    runningItem = name;
    try {
      onrun(nodeId, name);
    } finally {
      runningItem = null;
      onclose();
    }
  }

  async function handleRunMachine(name: string) {
    if (runningItem) return;
    runningItem = name;
    try {
      onrunmachine(nodeId, name);
    } finally {
      runningItem = null;
      onclose();
    }
  }

  // ── Position clamping ─────────────────────────────────────────────────────
  //
  // Keep the menu inside the viewport by flipping it left/up when it would
  // overflow.  We read the menu size after mount.

  let menuEl: HTMLElement | null = $state(null);

  let clampedX = $derived.by(() => {
    if (!menuEl) return x;
    const { width } = menuEl.getBoundingClientRect();
    return x + width > window.innerWidth ? x - width : x;
  });

  let clampedY = $derived.by(() => {
    if (!menuEl) return y;
    const { height } = menuEl.getBoundingClientRect();
    return y + height > window.innerHeight ? y - height : y;
  });

  // ── Dismiss on outside click or Escape ────────────────────────────────────

  function handleWindowClick(e: MouseEvent) {
    if (menuEl && !menuEl.contains(e.target as Node)) onclose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onclose();
  }

  onMount(() => {
    // Defer so the click that opened the menu doesn't immediately close it.
    setTimeout(() => {
      window.addEventListener('click', handleWindowClick);
      window.addEventListener('contextmenu', handleWindowClick);
    }, 0);
    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener('click', handleWindowClick);
    window.removeEventListener('contextmenu', handleWindowClick);
    window.removeEventListener('keydown', handleKeydown);
  });
</script>

<!-- Portal: rendered at top of body via fixed positioning -->
<div
  class="context-menu"
  role="menu"
  aria-label="Node transforms"
  tabindex="-1"
  bind:this={menuEl}
  style:left="{clampedX}px"
  style:top="{clampedY}px"
  oncontextmenu={e => e.preventDefault()}
>
  <!-- Header showing node identity -->
  <div class="menu-header">
    <span class="menu-node-type">{nodeType}</span>
    <span class="menu-node-label" title={nodeId}>{nodeLabel}</span>
  </div>

  <div class="menu-divider" role="separator"></div>

  <!-- Transform list -->
  {#if transforms.length === 0 && machines.length === 0}
    <div class="menu-empty">No actions available</div>
  {:else}
    <!-- Machines (Automation) -->
    {#if machines.length > 0}
      <div class="menu-section-label">Automation</div>
      {#each machines as m}
        <button
          class="menu-item machine-item"
          role="menuitem"
          onclick={() => handleRunMachine(m.name)}
          disabled={runningItem !== null}
          title={m.description}
        >
          <span class="menu-item-icon" aria-hidden="true">
            {#if runningItem === m.name}
              <span class="spinner" aria-hidden="true"></span>
            {:else}
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="m12 14 4-4-4-4m-8 8 4-4-4-4"/>
                <path d="M21 12H7"/>
              </svg>
            {/if}
          </span>
          <span class="menu-item-label">{m.name}</span>
        </button>
      {/each}
      <div class="menu-divider" role="separator"></div>
    {/if}

    <!-- Individual Transforms -->
    {#if transforms.length > 0}
      <div class="menu-section-label">Transforms</div>
      {#each transforms as t}
        <button
          class="menu-item"
          role="menuitem"
          onclick={() => handleRun(t.name)}
          disabled={runningItem !== null}
          title={t.description}
        >
          <span class="menu-item-icon" aria-hidden="true">
            {#if runningItem === t.name}
              <span class="spinner" aria-hidden="true"></span>
            {:else}
              <!-- Icon per transform family -->
              {#if t.name.toLowerCase().includes('dns')}
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="10"/>
                  <line x1="2" y1="12" x2="22" y2="12"/>
                  <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
                </svg>
              {:else if t.name.toLowerCase().includes('whois')}
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                  <polyline points="14 2 14 8 20 8"/>
                  <line x1="16" y1="13" x2="8" y2="13"/>
                  <line x1="16" y1="17" x2="8" y2="17"/>
                  <polyline points="10 9 9 9 8 9"/>
                </svg>
              {:else if t.name.toLowerCase().includes('geo')}
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"/>
                  <circle cx="12" cy="10" r="3"/>
                </svg>
              {:else}
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="13 2 13 9 20 9"/>
                  <path d="M20 14.5v3.5l-3 3H6a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8"/>
                </svg>
              {/if}
            {/if}
          </span>
          <span class="menu-item-label">{t.name}</span>
        </button>
      {/each}
    {/if}
  {/if}
</div>

<style>
  .context-menu {
    position: fixed;
    z-index: 9999;
    min-width: 190px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md), 0 0 0 1px rgba(0,0,0,.1);
    padding: 4px;
    /* Animate in */
    animation: menu-in 100ms ease-out;
  }

  @keyframes menu-in {
    from { opacity: 0; transform: scale(0.95) translateY(-4px); }
    to   { opacity: 1; transform: scale(1)    translateY(0);    }
  }

  /* ── Header ───────────────────────────────────────────────────────────── */
  .menu-header {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 5px 8px 6px;
  }

  .menu-node-type {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: .07em;
    color: var(--accent);
    flex-shrink: 0;
  }

  .menu-node-label {
    font-size: 11px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--font-mono);
  }

  .menu-section-label {
    font-size: 9px;
    font-weight: 700;
    text-transform: uppercase;
    color: var(--text-muted);
    padding: 4px 8px 2px;
    letter-spacing: 0.05em;
  }

  /* ── Divider ──────────────────────────────────────────────────────────── */
  .menu-divider {
    height: 1px;
    background: var(--border);
    margin: 4px;
  }

  /* ── Items ────────────────────────────────────────────────────────────── */
  .menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 8px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    font-family: var(--font-sans);
    text-align: left;
    cursor: pointer;
    transition: background 80ms, color 80ms;
  }

  .machine-item {
    color: var(--accent);
    font-weight: 500;
  }

  .menu-item:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--accent);
  }

  .machine-item:hover:not(:disabled) {
    background: color-mix(in srgb, var(--accent) 15%, transparent);
  }

  .menu-item:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .menu-item-icon {
    width: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    color: var(--text-muted);
  }
  .menu-item:hover:not(:disabled) .menu-item-icon { color: var(--accent); }

  .menu-item-label { flex: 1; }

  /* ── Empty state ──────────────────────────────────────────────────────── */
  .menu-empty {
    padding: 8px;
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    text-align: center;
  }

  /* ── Spinner ──────────────────────────────────────────────────────────── */
  .spinner {
    display: inline-block;
    width: 11px; height: 11px;
    border: 1.5px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin .5s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
