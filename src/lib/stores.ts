import { writable, derived, get } from 'svelte/store';
import type { Writable } from 'svelte/store';
import type { Provenance } from './types/provenance';

// ── Domain types (mirrors Rust structs) ──────────────────────────────────────

export interface Node {
  id: string;
  type: string;
  properties: Record<string, string>;
  /** Optional provenance metadata (may not be present for legacy nodes) */
  provenance?: Provenance;
}

export interface Edge {
  id: string;
  source: string;
  target: string;
  label: string;
  properties: Record<string, string>;
  /** Optional provenance metadata (may not be present for legacy nodes) */
  provenance?: Provenance;
}

export interface Graph {
  nodes: Node[];
  edges: Edge[];
}

export interface EntityDefinition {
  node_type: string;
  display_name: string;
  icon: string;
  color: string;
  shape: string;
}

export type SelectionKind = 'node' | 'edge';

export interface Selection {
  kind: SelectionKind;
  ids: string[];
}

// ── Graph store ───────────────────────────────────────────────────────────────

function createGraphStore() {
  const { subscribe, set, update }: Writable<Graph> = writable({ nodes: [], edges: [] });

  return {
    subscribe,

    /** Replace the entire graph (e.g. after load). */
    setGraph(g: Graph) {
      set(g);
    },

    /** Merge a new node in (no-op if id already present). */
    addNode(node: Node) {
      update(g => {
        if (g.nodes.some(n => n.id === node.id)) return g;
        return { ...g, nodes: [...g.nodes, node] };
      });
    },

    /** Overwrite properties of an existing node. */
    updateNode(id: string, properties: Record<string, string>) {
      update(g => ({
        ...g,
        nodes: g.nodes.map(n => n.id === id ? { ...n, properties } : n),
      }));
    },

    /** Remove a node and all edges that touch it. */
    removeNode(id: string) {
      update(g => {
        const updated = {
          nodes: g.nodes.filter(n => n.id !== id),
          edges: g.edges.filter(e => e.source !== id && e.target !== id),
        };
        // Clear selection if the removed node was selected
        const sel = get(selectionStore);
        if (sel?.kind === 'node' && sel.ids.includes(id)) {
          selectionStore.set(null);
        }
        return updated;
      });
    },

    /** Merge a new edge in (no-op if id already present). */
    addEdge(edge: Edge) {
      update(g => {
        if (g.edges.some(e => e.id === edge.id)) return g;
        return { ...g, edges: [...g.edges, edge] };
      });
    },

    /** Remove an edge by id. */
    removeEdge(id: string) {
      update(g => {
        // Clear selection if the removed edge was selected
        const sel = get(selectionStore);
        if (sel?.kind === 'edge' && sel.ids.includes(id)) {
          selectionStore.set(null);
        }
        return { ...g, edges: g.edges.filter(e => e.id !== id) };
      });
    },

    /** Convenience getter (outside reactive context). */
    snapshot(): Graph {
      return get(graphStore);
    },
  };
}

export const graphStore = createGraphStore();

// ── Selection store ───────────────────────────────────────────────────────────

export const selectionStore: Writable<Selection | null> = writable(null);

/** Derived: the first selected Node object, or null. */
export const selectedNode = derived(
  [graphStore, selectionStore],
  ([$graph, $sel]) =>
    $sel?.kind === 'node' && $sel.ids.length > 0
      ? ($graph.nodes.find((n: Node) => n.id === $sel.ids[0]) ?? null)
      : null,
);

/** Derived: all selected Node objects. */
export const selectedNodes = derived(
  [graphStore, selectionStore],
  ([$graph, $sel]) => {
    if ($sel?.kind !== 'node') return [];
    const idSet = new Set($sel.ids);
    return $graph.nodes.filter((n: Node) => idSet.has(n.id));
  }
);

/** Derived: the first selected Edge object, or null. */
export const selectedEdge = derived(
  [graphStore, selectionStore],
  ([$graph, $sel]) =>
    $sel?.kind === 'edge' && $sel.ids.length > 0
      ? ($graph.edges.find((e: Edge) => e.id === $sel.ids[0]) ?? null)
      : null,
);

/** Derived: all selected Edge objects. */
export const selectedEdges = derived(
  [graphStore, selectionStore],
  ([$graph, $sel]) => {
    if ($sel?.kind !== 'edge') return [];
    const idSet = new Set($sel.ids);
    return $graph.edges.filter((e: Edge) => idSet.has(e.id));
  }
);

// ── Transform types ───────────────────────────────────────────────────────────

/** Mirrors the Rust `TransformInfo` struct from commands.rs */
export interface TransformInfo {
  name: string;
  description: string;
  applicable_types: string[];   // empty means "all types"
}

/** Mirrors the Rust `TransformResult` struct from commands.rs */
export interface TransformResult {
  new_nodes: Node[];
  new_edges: Edge[];
}

/** Mirrors the Rust `Machine` struct from transforms.rs */
export interface Machine {
  name: string;
  description: string;
  transforms: string[];
}

// ── Current file path ─────────────────────────────────────────────────────────
//
// Tracks the last path used for Save/Load so that Ctrl+S can save in-place
// without opening a dialog when a file is already known.

export const currentFilePath: Writable<string | null> = writable(null);

// ── Theme store ───────────────────────────────────────────────────────────────

export type Theme = 'dark' | 'light';

const THEME_KEY = 'link-mage:theme';

function createThemeStore() {
  // Priority: localStorage → prefers-color-scheme → dark
  function readInitial(): Theme {
    if (typeof localStorage === 'undefined') return 'dark';
    const saved = localStorage.getItem(THEME_KEY);
    if (saved === 'light' || saved === 'dark') return saved;
    if (typeof window !== 'undefined' && window.matchMedia('(prefers-color-scheme: light)').matches) {
      return 'light';
    }
    return 'dark';
  }

  const { subscribe, set, update } = writable<Theme>(readInitial());

  function persist(t: Theme) {
    if (typeof localStorage !== 'undefined') localStorage.setItem(THEME_KEY, t);
  }

  return {
    subscribe,

    /** Explicitly set the theme and persist to localStorage. */
    set(t: Theme) {
      persist(t);
      set(t);
    },

    /** Toggle between dark and light and persist to localStorage. */
    toggle() {
      update(t => {
        const next: Theme = t === 'dark' ? 'light' : 'dark';
        persist(next);
        return next;
      });
    },
  };
}

export const themeStore = createThemeStore();

// ── Status / notification store ───────────────────────────────────────────────

export type StatusKind = 'idle' | 'info' | 'success' | 'error';

export interface StatusMessage {
  kind: StatusKind;
  text: string;
}

export const statusStore: Writable<StatusMessage> = writable({ kind: 'idle', text: '' });

// ── Settings panel ────────────────────────────────────────────────────────────

/** Controls whether the Settings side-sheet is open. */
export const settingsOpenStore: Writable<boolean> = writable(false);

let _statusTimer: ReturnType<typeof setTimeout> | null = null;

export function setStatus(kind: StatusKind, text: string, ttl = 3000) {
  if (_statusTimer !== null) {
    clearTimeout(_statusTimer);
    _statusTimer = null;
  }
  statusStore.set({ kind, text });
  if (kind !== 'idle') {
    _statusTimer = setTimeout(() => {
      statusStore.set({ kind: 'idle', text: '' });
      _statusTimer = null;
    }, ttl);
  }
}

/**
 * Cleanup function to call on app/component unmount to prevent
 * status timer from firing after destruction.
 */
export function cleanupStatus() {
  if (_statusTimer !== null) {
    clearTimeout(_statusTimer);
    _statusTimer = null;
  }
}
