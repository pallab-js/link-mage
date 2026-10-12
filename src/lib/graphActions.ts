/**
 * Shared graph file-operation helpers.
 *
 * Both Sidebar.svelte and KeyboardShortcuts.svelte call these so all file
 * operations are defined in exactly one place.
 */

import { invoke } from '@tauri-apps/api/core';
import { confirm } from '@tauri-apps/plugin-dialog';
import { get } from 'svelte/store';
import {
  graphStore,
  selectionStore,
  currentFilePath,
  setStatus,
} from './stores';
import type { Graph, Selection } from './stores';

// ── Concurrency guards ────────────────────────────────────────────────────────

let _saving  = false;
let _loading = false;

// ── Shape of the value returned by the Rust load_graph_dialog command ─────────

interface LoadResult {
  graph: Graph;
  path: string;
}

// ── File operations ───────────────────────────────────────────────────────────

/**
 * Clear the current graph after confirmation.
 * Resets the backend state, frontend store, selection, and known file path.
 */
export async function newGraph(): Promise<boolean> {
  if (!(await confirm('Clear the current graph? Unsaved changes will be lost.'))) return false;
  try {
    await invoke('clear_graph');
    graphStore.setGraph({ nodes: [], edges: [] });
    selectionStore.set(null);
    currentFilePath.set(null);
    setStatus('info', 'New graph');
    return true;
  } catch (e) {
    setStatus('error', String(e));
    return false;
  }
}

/**
 * Open a Save-As dialog, write the graph, and remember the chosen path.
 * Returns the saved path, or null if the user cancelled.
 */
export async function saveGraphAs(): Promise<string | null> {
  if (_saving) return null;
  _saving = true;
  try {
    const path: string | null = await invoke('save_graph_dialog');
    if (path) {
      currentFilePath.set(path);
      setStatus('success', `Saved → ${basename(path)}`);
      return path;
    }
    return null;
  } catch (e) {
    setStatus('error', String(e));
    return null;
  } finally {
    _saving = false;
  }
}

/**
 * Smart save:
 *   - If a file path is already known (previous save / load), write directly.
 *   - Otherwise fall back to Save-As dialog.
 */
export async function saveGraph(): Promise<boolean> {
  const path = get(currentFilePath);
  if (path) {
    if (_saving) return false;
    _saving = true;
    try {
      await invoke('save_graph_to_path', { path });
      setStatus('success', `Saved → ${basename(path)}`);
      return true;
    } catch (e) {
      setStatus('error', String(e));
      return false;
    } finally {
      _saving = false;
    }
  }
  return (await saveGraphAs()) !== null;
}

/**
 * Open a file dialog and load a graph from the selected JSON file.
 * Updates the frontend store and remembers the file path.
 */
export async function loadGraph(): Promise<boolean> {
  if (_loading) return false;
  _loading = true;
  try {
    const result: LoadResult | null = await invoke('load_graph_dialog');
    if (result) {
      graphStore.setGraph(result.graph);
      selectionStore.set(null);
      currentFilePath.set(result.path);
      setStatus(
        'success',
        `Loaded ${result.graph.nodes.length} node(s) — ${basename(result.path)}`,
      );
      return true;
    }
    return false;
  } catch (e) {
    setStatus('error', String(e));
    return false;
  } finally {
    _loading = false;
  }
}

/**
 * Load the bundled demo graph (replaces the current graph, no dialog).
 * Does not touch currentFilePath, so a later Ctrl+S still prompts Save As.
 */
export async function loadDemo(): Promise<boolean> {
  try {
    const g: Graph = await invoke('load_demo_graph');
    graphStore.setGraph(g);
    selectionStore.set(null);
    setStatus('success', `Demo graph loaded — ${g.nodes.length} nodes, ${g.edges.length} edges`);
    return true;
  } catch (e) {
    setStatus('error', String(e));
    return false;
  }
}

/**
 * Delete whichever nodes or edges are currently selected.
 * Updates both the backend state and the frontend store.
 */
export async function deleteSelected(selection: Selection | null): Promise<boolean> {
  if (!selection || selection.ids.length === 0) return false;
  try {
    if (selection.kind === 'node') {
      for (const id of selection.ids) {
        await invoke('remove_node', { id });
        graphStore.removeNode(id);
      }
      setStatus('success', `Deleted ${selection.ids.length} node(s)`);
    } else {
      for (const id of selection.ids) {
        await invoke('remove_edge', { id });
        graphStore.removeEdge(id);
      }
      setStatus('success', `Deleted ${selection.ids.length} edge(s)`);
    }
    selectionStore.set(null);
    return true;
  } catch (e) {
    setStatus('error', String(e));
    return false;
  }
}

// ── Utility ───────────────────────────────────────────────────────────────────

/** Return the final path component, handling both '/' and '\' separators. */
export function basename(p: string): string {
  return p.split(/[\\/]/).pop() ?? p;
}
