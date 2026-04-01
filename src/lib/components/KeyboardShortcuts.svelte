<script lang="ts">
  import { onMount } from 'svelte';
  import { fade } from 'svelte/transition';
  import { get } from 'svelte/store';
  import { selectionStore } from '$lib/stores';
  import { newGraph, saveGraph, saveGraphAs, loadGraph, deleteSelected } from '$lib/graphActions';
  import ShortcutHelp from './ShortcutHelp.svelte';

  // ── Props ─────────────────────────────────────────────────────────────────

  interface Props {
    /** Called when Ctrl+F fires — zooms the canvas to fit all nodes. */
    onFit?: () => void;
  }

  let { onFit }: Props = $props();

  // ── Help modal state ──────────────────────────────────────────────────────

  let showHelp = $state(false);

  function toggleHelp() {
    showHelp = !showHelp;
  }

  // ── Badge state ───────────────────────────────────────────────────────────

  interface Badge { combo: string; label: string }
  let badge: Badge | null = $state(null);
  let badgeTimer: ReturnType<typeof setTimeout> | null = null;

  function flash(combo: string, label: string) {
    if (badgeTimer) clearTimeout(badgeTimer);
    badge = { combo, label };
    badgeTimer = setTimeout(() => { badge = null; }, 1100);
  }

  // ── Platform detection ────────────────────────────────────────────────────
  // On macOS the primary modifier is ⌘ (metaKey); show the correct symbol.

  const isMac =
    typeof navigator !== 'undefined' &&
    /mac/i.test(navigator.platform);

  function modKey(e: KeyboardEvent): boolean {
    return isMac ? e.metaKey : e.ctrlKey;
  }

  function modSym(): string {
    return isMac ? '⌘' : 'Ctrl';
  }

  // ── Shortcut handler ──────────────────────────────────────────────────────

  function handleKeydown(e: KeyboardEvent) {
    // Skip when the user is typing in a text control.
    const target = e.target as HTMLElement;
    if (
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement ||
      target.isContentEditable
    ) return;

    const mod = modKey(e);
    const M = modSym();

    // ── Ctrl+Shift+S — Save As ────────────────────────────────────────────
    if (mod && e.shiftKey && e.key === 'S') {
      e.preventDefault();
      flash(`${M}⇧S`, 'Save As…');
      saveGraphAs();
      return;
    }

    // ── Ctrl+S — smart save ───────────────────────────────────────────────
    if (mod && !e.shiftKey && e.key === 's') {
      e.preventDefault();
      flash(`${M}S`, 'Save');
      saveGraph();
      return;
    }

    // ── Ctrl+O — open ─────────────────────────────────────────────────────
    if (mod && e.key === 'o') {
      e.preventDefault();
      flash(`${M}O`, 'Open…');
      loadGraph();
      return;
    }

    // ── Ctrl+N — new graph ────────────────────────────────────────────────
    if (mod && e.key === 'n') {
      e.preventDefault();
      flash(`${M}N`, 'New Graph');
      newGraph();
      return;
    }

    // ── Ctrl+F — fit to viewport ──────────────────────────────────────────
    if (mod && e.key === 'f') {
      e.preventDefault();
      flash(`${M}F`, 'Fit Graph');
      onFit?.();
      return;
    }

    // ── Ctrl+L — run layout ───────────────────────────────────────────────
    if (mod && e.key === 'l') {
      e.preventDefault();
      flash(`${M}L`, 'Auto Layout');
      // Dispatch custom event for Graph component to listen
      window.dispatchEvent(new CustomEvent('graph-layout-auto'));
      return;
    }

    // ── Ctrl+H — cycle layout ─────────────────────────────────────────────
    if (mod && e.key === 'h') {
      e.preventDefault();
      flash(`${M}H`, 'Cycle Layout');
      window.dispatchEvent(new CustomEvent('graph-layout-cycle'));
      return;
    }

    // ── Ctrl+Plus — zoom in ───────────────────────────────────────────────
    if (mod && (e.key === '+' || e.key === '=')) {
      e.preventDefault();
      flash(`${M}+`, 'Zoom In');
      window.dispatchEvent(new CustomEvent('graph-zoom-in'));
      return;
    }

    // ── Ctrl+Minus — zoom out ─────────────────────────────────────────────
    if (mod && e.key === '-') {
      e.preventDefault();
      flash(`${M}-`, 'Zoom Out');
      window.dispatchEvent(new CustomEvent('graph-zoom-out'));
      return;
    }

    // ── Ctrl+D — duplicate selected node ──────────────────────────────────
    if (mod && e.key === 'd') {
      e.preventDefault();
      flash(`${M}D`, 'Duplicate');
      // Future: implement duplicate functionality
      return;
    }

    // ── Ctrl+G — group selected nodes ─────────────────────────────────────
    if (mod && e.key === 'g') {
      e.preventDefault();
      flash(`${M}G`, 'Group');
      // Future: implement collapse to collection
      return;
    }

    // ── Ctrl+Shift+F — clear filters ──────────────────────────────────────
    if (mod && e.shiftKey && e.key === 'F') {
      e.preventDefault();
      flash(`${M}⇧F`, 'Clear Filters');
      window.dispatchEvent(new CustomEvent('graph-filters-clear'));
      return;
    }

    // ── Ctrl+A — select all ───────────────────────────────────────────────
    if (mod && e.key === 'a') {
      e.preventDefault();
      flash(`${M}A`, 'Select All');
      window.dispatchEvent(new CustomEvent('graph-select-all'));
      return;
    }

    // ── Escape — deselect all ─────────────────────────────────────────────
    if (e.key === 'Escape') {
      e.preventDefault();
      flash('Esc', 'Deselect');
      window.dispatchEvent(new CustomEvent('graph-deselect-all'));
      return;
    }

    // ── ? — show keyboard shortcuts help ──────────────────────────────────
    if (e.key === '?' || (e.shiftKey && e.key === '/')) {
      e.preventDefault();
      flash(`${M}?`, 'Keyboard Shortcuts');
      toggleHelp();
      return;
    }

    // ── Delete / Backspace — delete selected ──────────────────────────────
    if (e.key === 'Delete' || e.key === 'Backspace') {
      const sel = get(selectionStore);
      if (!sel) return;
      e.preventDefault();
      flash('⌫', `Delete ${sel.kind}`);
      deleteSelected(sel);
    }
  }

  // ── Lifecycle ─────────────────────────────────────────────────────────────

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
    return () => {
      window.removeEventListener('keydown', handleKeydown);
      if (badgeTimer) clearTimeout(badgeTimer);
    };
  });
</script>

<!-- ── Shortcut badge overlay ─────────────────────────────────────────────── -->

{#if badge}
  <div class="shortcut-badge" transition:fade={{ duration: 140 }} aria-live="polite">
    <kbd class="badge-key">{badge.combo}</kbd>
    <span class="badge-label">{badge.label}</span>
  </div>
{/if}

<!-- ── Help hint (always visible, bottom-left) ───────────────────────────── -->
<div class="shortcut-hint" aria-hidden="true" onclick={toggleHelp} style="cursor: pointer" title="Click for keyboard shortcuts">
  {#if isMac}
    ⌘N · ⌘O · ⌘S · ⌘F · ⌘L · ⌘G · ⌘A · ⌫ · Esc · ?
  {:else}
    Ctrl+N · Ctrl+O · Ctrl+S · Ctrl+F · Ctrl+L · Ctrl+G · Ctrl+A · Del · Esc · ?
  {/if}
</div>

<!-- Keyboard shortcuts help modal -->
<ShortcutHelp open={showHelp} onClose={() => showHelp = false} />

<style>
  /* ── Toast badge ────────────────────────────────────────────────────────── */
  .shortcut-badge {
    position: fixed;
    bottom: 36px;
    right: 20px;
    z-index: 9000;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: var(--bg-elevated);
    border: 1px solid var(--accent);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
    pointer-events: none;
    user-select: none;
  }

  .badge-key {
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    color: var(--accent);
    padding: 1px 5px;
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: 3px;
    line-height: 1.6;
  }

  .badge-label {
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    white-space: nowrap;
  }

  /* ── Persistent hint row ────────────────────────────────────────────────── */
  .shortcut-hint {
    position: fixed;
    bottom: 8px;
    right: 12px;
    z-index: 100;
    font-size: 10px;
    color: var(--text-muted);
    pointer-events: none;
    user-select: none;
    letter-spacing: .03em;
  }
</style>
