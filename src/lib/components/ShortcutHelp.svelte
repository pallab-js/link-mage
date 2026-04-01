<script lang="ts">
  import { fade, slide } from 'svelte/transition';

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();

  const isMac = typeof navigator !== 'undefined' && /mac/i.test(navigator.platform);
  const modKey = isMac ? '⌘' : 'Ctrl';
  const modName = isMac ? 'Command' : 'Ctrl';

  interface ShortcutGroup {
    title: string;
    icon: string;
    shortcuts: Array<{ keys: string[]; description: string; category?: string }>;
  }

  const shortcutGroups: ShortcutGroup[] = [
    {
      title: 'File Operations',
      icon: '📁',
      shortcuts: [
        { keys: [modKey, 'N'], description: 'New graph' },
        { keys: [modKey, 'O'], description: 'Open graph' },
        { keys: [modKey, 'S'], description: 'Save graph' },
        { keys: [modKey, '⇧', 'S'], description: 'Save as...' },
      ],
    },
    {
      title: 'View & Navigation',
      icon: '👁️',
      shortcuts: [
        { keys: [modKey, 'F'], description: 'Fit graph to viewport' },
        { keys: [modKey, '+'], description: 'Zoom in' },
        { keys: [modKey, '-'], description: 'Zoom out' },
        { keys: [modKey, 'L'], description: 'Auto layout' },
        { keys: [modKey, 'H'], description: 'Cycle layout' },
        { keys: ['F'], description: 'Fit (when canvas focused)' },
        { keys: ['L'], description: 'Layout (when canvas focused)' },
      ],
    },
    {
      title: 'Selection',
      icon: '🎯',
      shortcuts: [
        { keys: [modKey, 'A'], description: 'Select all nodes' },
        { keys: ['Escape'], description: 'Deselect all' },
        { keys: ['Delete'], description: 'Delete selected' },
        { keys: ['Backspace'], description: 'Delete selected' },
      ],
    },
    {
      title: 'Tags & Annotations',
      icon: '🏷️',
      shortcuts: [
        { keys: ['1', '2', '3', '4', '5'], description: 'Quick tag assignment (1-5)' },
        { keys: [modKey, 'T'], description: 'Toggle tags panel' },
        { keys: [modKey, 'M'], description: 'Add annotation' },
      ],
    },
    {
      title: 'Pipelines',
      icon: '⚙️',
      shortcuts: [
        { keys: [modKey, 'P'], description: 'Run pipeline' },
        { keys: [modKey, '⇧', 'P'], description: 'Open pipeline manager' },
      ],
    },
    {
      title: 'Search & Filter',
      icon: '🔍',
      shortcuts: [
        { keys: [modKey, 'K'], description: 'Command palette / Search' },
        { keys: [modKey, 'F', 'F'], description: 'Focus filter panel' },
        { keys: [modKey, '⇧', 'F'], description: 'Clear all filters' },
      ],
    },
    {
      title: 'UI Navigation',
      icon: '🖥️',
      shortcuts: [
        { keys: [modKey, ','], description: 'Open settings' },
        { keys: [modKey, 'B'], description: 'Toggle sidebar' },
        { keys: [modKey, '?'], description: 'Show this help' },
        { keys: ['?'], description: 'Show help (anywhere)' },
      ],
    },
    {
      title: 'Node Actions',
      icon: '⬤',
      shortcuts: [
        { keys: [modKey, 'D'], description: 'Duplicate node' },
        { keys: [modKey, 'G'], description: 'Group nodes (collapse)' },
        { keys: [modKey, 'U'], description: 'Ungroup collection' },
        { keys: [modKey, 'E'], description: 'Add edge' },
      ],
    },
    {
      title: 'Transforms',
      icon: '🔄',
      shortcuts: [
        { keys: [modKey, 'R'], description: 'Run transform on selection' },
        { keys: [modKey, '⇧', 'R'], description: 'Run last transform' },
        { keys: [modKey, 'T', 'T'], description: 'Open transform menu' },
      ],
    },
  ];

  function formatKey(key: string): string {
    const replacements: Record<string, string> = {
      'mod': modKey,
      'ctrl': isMac ? '⌃' : 'Ctrl',
      'alt': isMac ? '⌥' : 'Alt',
      'shift': '⇧',
      'meta': '⊞',
      'escape': 'Esc',
      'delete': 'Del',
      'backspace': '⌫',
      'enter': '↵',
      'space': '␣',
      'arrowup': '↑',
      'arrowdown': '↓',
      'arrowleft': '←',
      'arrowright': '→',
    };
    return replacements[key.toLowerCase()] || key.toUpperCase();
  }
</script>

{#if open}
  <div class="shortcut-help-overlay" transition:fade={{ duration: 200 }} onclick={(e) => { e.stopPropagation(); onClose(); }}>
    <div 
      class="shortcut-help-modal" 
      transition:slide={{ duration: 300, axis: 'y' }}
      onclick={(e) => e.stopPropagation()}
    >
      <div class="modal-header">
        <div class="modal-title">
          <span class="title-icon">⌨️</span>
          <h2>Keyboard Shortcuts</h2>
        </div>
        <button class="modal-close" onclick={onClose} aria-label="Close">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <div class="modal-platform-hint">
        Showing shortcuts for <strong>{isMac ? 'macOS' : 'Windows/Linux'}</strong>
        {#if !isMac}
          <span class="hint-note">On macOS, Command (⌘) replaces Ctrl for most shortcuts</span>
        {/if}
      </div>

      <div class="shortcut-groups">
        {#each shortcutGroups as group}
          <div class="shortcut-group">
            <h3 class="group-title">
              <span class="group-icon">{group.icon}</span>
              {group.title}
            </h3>
            <div class="shortcut-list">
              {#each group.shortcuts as shortcut}
                <div class="shortcut-item">
                  <div class="shortcut-keys">
                    {#each shortcut.keys as key}
                      <kbd class="key-chip">{formatKey(key)}</kbd>
                    {/each}
                  </div>
                  <span class="shortcut-description">{shortcut.description}</span>
                </div>
              {/each}
            </div>
          </div>
        {/each}
      </div>

      <div class="modal-footer">
        <p class="footer-note">
          <span class="tip-icon">💡</span>
          Tip: Many shortcuts work globally, but some require the graph canvas to be focused.
          Press <kbd>?</kbd> anytime to reopen this help.
        </p>
      </div>
    </div>
  </div>
{/if}

<style>
  .shortcut-help-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(4px);
    z-index: 10000;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;
    overflow-y: auto;
  }

  .shortcut-help-modal {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-xl);
    max-width: 900px;
    width: 100%;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-base);
  }

  .modal-title {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .title-icon {
    font-size: 20px;
  }

  .modal-header h2 {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .modal-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s;
  }

  .modal-close:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .modal-platform-hint {
    padding: 10px 20px;
    background: var(--bg-elevated);
    border-bottom: 1px solid var(--border);
    font-size: 12px;
    color: var(--text-secondary);
  }

  .hint-note {
    margin-left: 8px;
    opacity: 0.8;
  }

  .shortcut-groups {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 20px;
  }

  .shortcut-group {
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .group-title {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    background: var(--bg-elevated);
    border-bottom: 1px solid var(--border);
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .group-icon {
    font-size: 16px;
  }

  .shortcut-list {
    padding: 8px;
  }

  .shortcut-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    transition: background 0.1s;
  }

  .shortcut-item:hover {
    background: var(--bg-hover);
  }

  .shortcut-keys {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }

  .key-chip {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 24px;
    height: 24px;
    padding: 0 6px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 600;
    color: var(--text-primary);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  }

  .shortcut-description {
    font-size: 12px;
    color: var(--text-secondary);
    text-align: right;
    flex: 1;
  }

  .modal-footer {
    padding: 14px 20px;
    border-top: 1px solid var(--border);
    background: var(--bg-base);
  }

  .footer-note {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    font-size: 12px;
    color: var(--text-muted);
  }

  .tip-icon {
    font-size: 14px;
  }

  .footer-note kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 20px;
    height: 20px;
    padding: 0 5px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  /* Scrollbar styling */
  .shortcut-groups::-webkit-scrollbar {
    width: 8px;
  }

  .shortcut-groups::-webkit-scrollbar-track {
    background: var(--bg-base);
    border-radius: 4px;
  }

  .shortcut-groups::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 4px;
  }

  .shortcut-groups::-webkit-scrollbar-thumb:hover {
    background: var(--text-muted);
  }

  /* Responsive */
  @media (max-width: 768px) {
    .shortcut-groups {
      grid-template-columns: 1fr;
    }

    .shortcut-item {
      flex-direction: column;
      align-items: flex-start;
      gap: 6px;
    }

    .shortcut-description {
      text-align: left;
    }
  }
</style>
