<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { confirm } from '@tauri-apps/plugin-dialog';
  import {
    graphStore,
    selectionStore,
    selectedNode,
    selectedNodes,
    selectedEdge,
    selectedEdges,
    setStatus,
    type Graph,
  } from '$lib/stores';
  import type { Node, Edge } from '$lib/stores';
  import { NODE_TYPES, type NodeType } from '$lib/nodeTypes';
  import ThemeToggle from './ThemeToggle.svelte';
  import { newGraph, saveGraph, loadGraph, basename } from '$lib/graphActions';
  import FilterPanel from './FilterPanel.svelte';
  import ProvenancePanel from './ProvenancePanel.svelte';
  import TagManager from './TagManager.svelte';

  // ── Props (viewport actions forwarded from the page) ──────────────────────


  interface Props {
    onFit?: () => void;
    onLayout?: () => void;
    onZoomIn?: () => void;
    onZoomOut?: () => void;
  }

  let { onFit, onLayout, onZoomIn, onZoomOut }: Props = $props();

  // ── Svelte action: focus element on mount ────────────────────────────────
  function focusMounted(el: HTMLElement) { el.focus(); }

  // ── Section collapse state ────────────────────────────────────────────────

  let openSections = $state({
    file: true,
    filters: false,
    tags: false,
    addNode: true,
    addEdge: false,
    stats: true,
  });

  function toggle(key: keyof typeof openSections) {
    openSections[key] = !openSections[key];
  }

  // ── Loading state ─────────────────────────────────────────────────────────
  //
  // `busy` keys mirror action names so multiple independent operations don't
  // share the same spinner.  Components test `busy.save`, `busy.load`, etc.

  type BusyKey =
    | 'addNode' | 'addEdge'
    | 'save' | 'load' | 'exportCsv' | 'exportGraphml' | 'importCsv'
    | 'removeNode' | 'removeEdge'
    | 'updateProp' | 'deleteProp';

  let busy = $state<Partial<Record<BusyKey, boolean>>>({});

  function setBusy(key: BusyKey, val: boolean) { busy = { ...busy, [key]: val }; }

  // ── Add-node form ─────────────────────────────────────────────────────────

  // NODE_TYPES is imported from $lib/nodeTypes.ts

  let newNodeType  = $state<NodeType>('IP');
  let newNodeValue = $state('');
  // Extra properties the user can add before submitting
  let extraProps   = $state<{ key: string; val: string }[]>([]);

  function addExtraProp() {
    extraProps = [...extraProps, { key: '', val: '' }];
  }
  function removeExtraProp(i: number) {
    extraProps = extraProps.filter((_, idx) => idx !== i);
  }

  async function handleAddNode() {
    const trimmed = newNodeValue.trim();
    if (!trimmed || busy.addNode) return;

    const properties: Record<string, string> = { value: trimmed, label: trimmed };
    for (const { key, val } of extraProps) {
      if (key.trim()) properties[key.trim()] = val;
    }

    const node: Node = { id: '', type: newNodeType, properties };
    setBusy('addNode', true);
    try {
      const id: string = await invoke('add_node', { node });
      graphStore.addNode({ ...node, id });
      newNodeValue = '';
      extraProps = [];
      setStatus('success', `Added ${newNodeType}: ${trimmed}`);
    } catch (e) {
      setStatus('error', String(e));
    } finally {
      setBusy('addNode', false);
    }
  }

  // ── Add-edge form ─────────────────────────────────────────────────────────

  let edgeSource = $state('');
  let edgeTarget = $state('');
  let edgeLabel  = $state('resolves-to');

  // Display label helper for node dropdowns
  function nodeDisplayName(n: Node): string {
    return `${n.type}: ${n.properties['value'] ?? n.properties['label'] ?? n.id.slice(0, 8)}`;
  }

  async function handleAddEdge() {
    if (!edgeSource || !edgeTarget || !edgeLabel.trim() || busy.addEdge) return;
    if (edgeSource === edgeTarget) {
      setStatus('error', 'Source and target must be different nodes');
      return;
    }

    const edge: Edge = {
      id: '',
      source: edgeSource,
      target: edgeTarget,
      label: edgeLabel.trim(),
      properties: {},
    };

    setBusy('addEdge', true);
    try {
      const id: string = await invoke('add_edge', { edge });
      graphStore.addEdge({ ...edge, id });
      const addedLabel = edge.label;
      edgeLabel = 'resolves-to';
      setStatus('success', `Edge added: ${addedLabel}`);
    } catch (e) {
      setStatus('error', String(e));
    } finally {
      setBusy('addEdge', false);
    }
  }

  // ── Remove selected element ───────────────────────────────────────────────

  async function handleRemoveNode() {
    const nodes = $selectedNodes;
    if (nodes.length === 0 || busy.removeNode) return;
    
    setBusy('removeNode', true);
    try {
      if (nodes.length === 1) {
        const node = nodes[0];
        await invoke('remove_node', { id: node.id });
        graphStore.removeNode(node.id);
        setStatus('info', `Removed node: ${nodeDisplayName(node)}`);
      } else {
        if (!(await confirm(`Delete ${nodes.length} nodes and their edges?`))) return;
        for (const n of nodes) {
          await invoke('remove_node', { id: n.id });
          graphStore.removeNode(n.id);
        }
        setStatus('info', `Removed ${nodes.length} nodes`);
      }
      selectionStore.set(null);
    } catch (e) {
      setStatus('error', String(e));
    } finally {
      setBusy('removeNode', false);
    }
  }

  let collapseLabel = $state('');

  async function handleCollapse() {
    const nodes = $selectedNodes;
    if (nodes.length < 2 || busy.save) return;

    // window.prompt is dead in Tauri v2 — the label comes from the inline
    // input next to the Collapse button.
    const label = collapseLabel.trim() || `${nodes.length} entities`;

    setBusy('save', true); // Re-use save busy state for now or add a new one
    try {
      const col: Node = await invoke('collapse_nodes', { 
        ids: nodes.map((n: Node) => n.id), 
        label 
      });
      
      // Update store: remove original nodes, add collection
      for (const n of nodes) graphStore.removeNode(n.id);
      graphStore.addNode(col);
      
      // We also need to refresh edges because collapse_nodes re-routed them in Rust.
      // For simplicity, just fetch the whole graph or update edges.
      const fullGraph: { nodes: Node[], edges: Edge[] } = await invoke('get_graph');
      graphStore.setGraph(fullGraph);
      
      selectionStore.set({ kind: 'node', ids: [col.id] });
      collapseLabel = '';
      setStatus('success', `Collapsed ${nodes.length} nodes into '${label}'`);
    } catch (e) {
      setStatus('error', String(e));
    } finally {
      setBusy('save', false);
    }
  }

  async function handleRemoveEdge() {
    const edges = $selectedEdges;
    if (edges.length === 0 || busy.removeEdge) return;
    
    setBusy('removeEdge', true);
    try {
      if (edges.length === 1) {
        await invoke('remove_edge', { id: edges[0].id });
        graphStore.removeEdge(edges[0].id);
        setStatus('info', 'Edge removed');
      } else {
        if (!(await confirm(`Delete ${edges.length} edges?`))) return;
        for (const e of edges) {
          await invoke('remove_edge', { id: e.id });
          graphStore.removeEdge(e.id);
        }
        setStatus('info', `Removed ${edges.length} edges`);
      }
      selectionStore.set(null);
    } catch (e) {
      setStatus('error', String(e));
    } finally {
      setBusy('removeEdge', false);
    }
  }

  // ── Node property editing ─────────────────────────────────────────────────
  //
  // Each property row has a local edit buffer keyed by property name.  When
  // the user blurs or presses Enter, the change is committed.

  // Tracks which key is being edited and its draft value
  let editingPropKey  = $state<string | null>(null);
  let editingPropDraft = $state('');

  function startEditProp(key: string, current: string) {
    editingPropKey   = key;
    editingPropDraft = current;
  }

  async function commitPropEdit(node: Node) {
    if (editingPropKey === null) return;
    const updated = { ...node.properties, [editingPropKey]: editingPropDraft };
    setBusy('updateProp', true);
    try {
      await invoke('update_node', { id: node.id, properties: updated });
      graphStore.updateNode(node.id, updated);
    } catch (e) {
      setStatus('error', String(e));
    } finally {
      editingPropKey  = null;
      editingPropDraft = '';
      setBusy('updateProp', false);
    }
  }

  function cancelPropEdit() {
    editingPropKey   = null;
    editingPropDraft = '';
  }

  async function handleDeleteProp(node: Node, key: string) {
    const updated = { ...node.properties };
    delete updated[key];
    setBusy('deleteProp', true);
    try {
      await invoke('update_node', { id: node.id, properties: updated });
      graphStore.updateNode(node.id, updated);
    } catch (e) {
      setStatus('error', String(e));
    } finally {
      setBusy('deleteProp', false);
    }
  }

  // ── Add property to selected node ─────────────────────────────────────────

  let newPropKey = $state('');
  let newPropVal = $state('');

  async function handleAddProp(node: Node) {
    const k = newPropKey.trim();
    if (!k || busy.updateProp) return;
    const updated = { ...node.properties, [k]: newPropVal };
    setBusy('updateProp', true);
    try {
      await invoke('update_node', { id: node.id, properties: updated });
      graphStore.updateNode(node.id, updated);
      newPropKey = '';
      newPropVal = '';
    } catch (e) {
      setStatus('error', String(e));
    } finally {
      setBusy('updateProp', false);
    }
  }

  // ── File operations (delegated to graphActions) ───────────────────────────

  async function handleSave() {
    if (busy.save) return;
    setBusy('save', true);
    try { await saveGraph(); } finally { setBusy('save', false); }
  }

  async function handleLoad() {
    if (busy.load) return;
    setBusy('load', true);
    try { await loadGraph(); } finally { setBusy('load', false); }
  }

  async function handleNewGraph() {
    await newGraph();
  }

  async function handleExportCsv() {
    if (busy.exportCsv) return;
    setBusy('exportCsv', true);
    try {
      const dir: string | null = await invoke('export_csv_dialog');
      if (dir) setStatus('success', `CSV exported → ${basename(dir)}`);
    } catch (e) {
      setStatus('error', String(e));
    } finally {
      setBusy('exportCsv', false);
    }
  }

  async function handleImportCsv() {
    if (busy.importCsv) return;
    if (!(await confirm('Import CSV will replace the current graph. Continue?'))) return;
    setBusy('importCsv', true);
    try {
      const g: Graph = await invoke('import_csv_dialog');
      if (g) {
        graphStore.setGraph(g);
        selectionStore.set(null);
        setStatus('success', `CSV imported — ${g.nodes.length} nodes, ${g.edges.length} edges`);
      }
    } catch (e) {
      setStatus('error', String(e));
    } finally {
      setBusy('importCsv', false);
    }
  }



  async function handleExportGraphml() {
    if (busy.exportGraphml) return;
    setBusy('exportGraphml', true);
    try {
      const path: string | null = await invoke('export_graphml_dialog');
      if (path) setStatus('success', `GraphML exported → ${basename(path)}`);
    } catch (e) {
      setStatus('error', String(e));
    } finally {
      setBusy('exportGraphml', false);
    }
  }
</script>

<!-- ═══════════════════════════════════════════════════════════════════════ -->
<aside class="sidebar">

  <!-- ── Header ──────────────────────────────────────────────────────────── -->
  <div class="sidebar-header">
    <span class="logo-mark">⬡</span>
    <span class="app-name">Link Mage</span>
    <ThemeToggle />
  </div>

  <!-- ── View / layout ───────────────────────────────────────────────────── -->
  <section class="sidebar-section">
    <h2 class="section-title">View</h2>
    <div class="btn-grid">
      <button class="btn btn-ghost" onclick={onFit} title="Fit graph to viewport">
        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <path d="M8 3H5a2 2 0 0 0-2 2v3"/><path d="M21 8V5a2 2 0 0 0-2-2h-3"/>
          <path d="M3 16v3a2 2 0 0 0 2 2h3"/><path d="M16 21h3a2 2 0 0 0 2-2v-3"/>
        </svg>
        Fit
      </button>
      <button class="btn btn-ghost" onclick={onLayout} title="Run force-directed layout">
        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/>
          <line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/>
          <line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/>
        </svg>
        Layout
      </button>
      <button class="btn btn-ghost" onclick={onZoomIn} title="Zoom in">
        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
          <line x1="11" y1="8" x2="11" y2="14"/><line x1="8" y1="11" x2="14" y2="11"/>
        </svg>
        Zoom +
      </button>
      <button class="btn btn-ghost" onclick={onZoomOut} title="Zoom out">
        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
          <line x1="8" y1="11" x2="14" y2="11"/>
        </svg>
        Zoom −
      </button>
    </div>
  </section>

  <!-- ── File ────────────────────────────────────────────────────────────── -->
  <section class="sidebar-section">
    <button class="section-toggle" onclick={() => toggle('file')} aria-expanded={openSections.file}>
      <h2 class="section-title">File</h2>
      <svg class="chevron" class:open={openSections.file} width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" aria-hidden="true">
        <polyline points="6 9 12 15 18 9"/>
      </svg>
    </button>

    {#if openSections.file}
      <div class="section-body">
        <button class="btn btn-secondary btn-full" onclick={handleSave} disabled={busy.save}>
          {#if busy.save}<span class="spinner" aria-hidden="true"></span>{:else}
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
              <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
              <polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/>
            </svg>
          {/if}
          {busy.save ? 'Saving…' : 'Save Graph'}
        </button>

        <button class="btn btn-secondary btn-full" onclick={handleLoad} disabled={busy.load}>
          {#if busy.load}<span class="spinner" aria-hidden="true"></span>{:else}
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
          {/if}
          {busy.load ? 'Loading…' : 'Open Graph'}
        </button>

        <button class="btn btn-danger btn-full" onclick={handleNewGraph}>
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
            <polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14H6L5 6"/>
            <path d="M10 11v6"/><path d="M14 11v6"/>
          </svg>
          New Graph
        </button>

        <button class="btn btn-secondary btn-full" onclick={handleExportCsv} disabled={busy.exportCsv}>
          {#if busy.exportCsv}<span class="spinner" aria-hidden="true"></span>{:else}
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="7 10 12 15 17 10"/>
              <line x1="12" y1="15" x2="12" y2="3"/>
            </svg>
          {/if}
          {busy.exportCsv ? 'Exporting…' : 'Export CSV'}
        </button>

        <button class="btn btn-secondary btn-full" onclick={handleExportGraphml} disabled={busy.exportGraphml}>
          {#if busy.exportGraphml}<span class="spinner" aria-hidden="true"></span>{:else}
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
              <polyline points="16 18 22 12 16 6"/>
              <polyline points="8 6 2 12 8 18"/>
            </svg>
          {/if}
          {busy.exportGraphml ? 'Exporting…' : 'Export GraphML'}
        </button>

        <button class="btn btn-secondary btn-full" onclick={handleImportCsv} disabled={busy.importCsv}>
          {#if busy.importCsv}<span class="spinner" aria-hidden="true"></span>{:else}
            <!-- upload arrow: arrow points UP into tray -->
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="17 8 12 3 7 8"/>
              <line x1="12" y1="3" x2="12" y2="15"/>
            </svg>
          {/if}
          {busy.importCsv ? 'Importing…' : 'Import CSV'}
        </button>
      </div>
    {/if}
  </section>

  <!-- ── Filters ────────────────────────────────────────────────────────── -->
  <section class="sidebar-section">
    <button class="section-toggle" onclick={() => toggle('filters')} aria-expanded={openSections.filters}>
      <h2 class="section-title">Filters & Search</h2>
      <svg class="chevron" class:open={openSections.filters} width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" aria-hidden="true">
        <polyline points="6 9 12 15 18 9"/>
      </svg>
    </button>

    {#if openSections.filters}
      <div class="section-body">
        <FilterPanel />
      </div>
    {/if}
  </section>

  <!-- ── Graph stats ──────────────────────────────────────────────────────── -->
  {#if openSections.stats}
    <section class="sidebar-section stats-bar">
      <div class="stat-chip">
        <span class="stat-dot dot-node"></span>
        <span class="stat-num">{$graphStore.nodes.length}</span>
        <span class="stat-lbl">nodes</span>
      </div>
      <div class="stat-chip">
        <span class="stat-dot dot-edge"></span>
        <span class="stat-num">{$graphStore.edges.length}</span>
        <span class="stat-lbl">edges</span>
      </div>
    </section>
  {/if}

  <!-- ── Tags & Annotations ─────────────────────────────────────────────── -->
  <section class="sidebar-section">
    <button class="section-toggle" onclick={() => toggle('tags')} aria-expanded={openSections.tags}>
      <h2 class="section-title">Tags & Notes</h2>
      <svg class="chevron" class:open={openSections.tags} width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" aria-hidden="true">
        <polyline points="6 9 12 15 18 9"/>
      </svg>
    </button>

    {#if openSections.tags}
      <div class="section-body">
        <TagManager />
      </div>
    {/if}
  </section>

  <!-- ── Add Node ─────────────────────────────────────────────────────────── -->
  <section class="sidebar-section">
    <button class="section-toggle" onclick={() => toggle('addNode')} aria-expanded={openSections.addNode}>
      <h2 class="section-title">Add Entity</h2>
      <svg class="chevron" class:open={openSections.addNode} width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" aria-hidden="true">
        <polyline points="6 9 12 15 18 9"/>
      </svg>
    </button>

    {#if openSections.addNode}
      <div class="section-body">
        <div class="form-group">
          <label class="form-label" for="node-type">Type</label>
          <select id="node-type" class="form-select" bind:value={newNodeType}>
            {#each NODE_TYPES as t}
              <option value={t}>{t}</option>
            {/each}
          </select>
        </div>

        <div class="form-group">
          <label class="form-label" for="node-value">Value</label>
          <input
            id="node-value"
            class="form-input"
            type="text"
            placeholder="e.g. 8.8.8.8"
            bind:value={newNodeValue}
            onkeydown={e => e.key === 'Enter' && handleAddNode()}
          />
        </div>

        <!-- Extra properties (optional) -->
        {#if extraProps.length}
          <div class="extra-props">
            {#each extraProps as ep, i}
              <div class="extra-prop-row">
                <input class="form-input" placeholder="key" bind:value={ep.key} />
                <input class="form-input" placeholder="value" bind:value={ep.val} />
                <button class="icon-btn danger-icon" onclick={() => removeExtraProp(i)} aria-label="Remove property">
                  <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                </button>
              </div>
            {/each}
          </div>
        {/if}

        <button class="btn btn-ghost btn-sm" onclick={addExtraProp}>+ Add property</button>

        <button
          class="btn btn-primary btn-full mt-sm"
          onclick={handleAddNode}
          disabled={!newNodeValue.trim() || busy.addNode}
        >
          {#if busy.addNode}<span class="spinner" aria-hidden="true"></span>{/if}
          {busy.addNode ? 'Adding…' : '+ Add Node'}
        </button>
      </div>
    {/if}
  </section>

  <!-- ── Add Edge ──────────────────────────────────────────────────────────── -->
  <section class="sidebar-section">
    <button class="section-toggle" onclick={() => toggle('addEdge')} aria-expanded={openSections.addEdge}>
      <h2 class="section-title">Add Edge</h2>
      <svg class="chevron" class:open={openSections.addEdge} width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" aria-hidden="true">
        <polyline points="6 9 12 15 18 9"/>
      </svg>
    </button>

    {#if openSections.addEdge}
      <div class="section-body">
        {#if $graphStore.nodes.length < 2}
          <p class="hint">Add at least two nodes first.</p>
        {:else}
          <div class="form-group">
            <label class="form-label" for="edge-source">Source</label>
            <select id="edge-source" class="form-select" bind:value={edgeSource}>
              <option value="" disabled>— pick node —</option>
              {#each $graphStore.nodes as n}
                <option value={n.id}>{nodeDisplayName(n)}</option>
              {/each}
            </select>
          </div>

          <div class="edge-arrow">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="var(--text-muted)" stroke-width="2" aria-hidden="true">
              <line x1="12" y1="5" x2="12" y2="19"/><polyline points="19 12 12 19 5 12"/>
            </svg>
          </div>

          <div class="form-group">
            <label class="form-label" for="edge-target">Target</label>
            <select id="edge-target" class="form-select" bind:value={edgeTarget}>
              <option value="" disabled>— pick node —</option>
              {#each $graphStore.nodes.filter((n: Node) => n.id !== edgeSource) as n}
                <option value={n.id}>{nodeDisplayName(n)}</option>
              {/each}
            </select>
          </div>

          <div class="form-group">
            <label class="form-label" for="edge-label">Relationship</label>
            <input
              id="edge-label"
              class="form-input"
              type="text"
              placeholder="resolves-to"
              bind:value={edgeLabel}
              onkeydown={e => e.key === 'Enter' && handleAddEdge()}
            />
          </div>

          <button
            class="btn btn-primary btn-full"
            onclick={handleAddEdge}
            disabled={!edgeSource || !edgeTarget || !edgeLabel.trim() || busy.addEdge}
          >
            {#if busy.addEdge}<span class="spinner" aria-hidden="true"></span>{/if}
            {busy.addEdge ? 'Adding…' : '+ Add Edge'}
          </button>
        {/if}
      </div>
    {/if}
  </section>

  <!-- ── Selection detail ─────────────────────────────────────────────────── -->
  {#if $selectedNodes.length > 0}
    {@const nodes = $selectedNodes}
    {@const node = nodes[0]}
    <section class="sidebar-section detail-panel">
      <!-- Header row -->
      <div class="detail-header">
        <div class="detail-header-left">
          {#if nodes.length === 1}
            <span class="type-badge type-{node.type.toLowerCase()}">{node.type}</span>
            <span class="detail-id" title={node.id}>{node.id.slice(0, 8)}…</span>
          {:else}
            <span class="type-badge type-collection">{nodes.length} Items</span>
            <span class="detail-id">Multiple Selection</span>
          {/if}
        </div>
        <div class="header-actions">
          {#if nodes.length > 1}
            <input
              class="form-input collapse-label"
              bind:value={collapseLabel}
              placeholder="Label"
              title="Collection label (defaults to 'N entities')"
            />
            <button
              class="btn btn-secondary btn-sm"
              onclick={handleCollapse}
              title="Collapse selected nodes into a single collection"
            >Collapse</button>
          {/if}
          <button
            class="btn btn-danger btn-sm"
            onclick={handleRemoveNode}
            disabled={busy.removeNode}
            title="Delete selection"
          >
            {#if busy.removeNode}<span class="spinner spinner-sm" aria-hidden="true"></span>
            {:else}Delete{/if}
          </button>
        </div>
      </div>

      {#if nodes.length === 1}
        <!-- Properties list (only for single selection) -->
        <div class="props-table" role="table" aria-label="Node properties">
          <div class="props-table-head" role="row">
            <span role="columnheader">Key</span>
            <span role="columnheader">Value</span>
            <span role="columnheader" class="sr-only">Actions</span>
          </div>

          {#each Object.entries(node.properties) as [k, v]}
            <div class="props-table-row" role="row">
              <span class="prop-key" role="cell" title={k}>{k}</span>

              <div class="prop-val-cell" role="cell">
                {#if editingPropKey === k}
                  <!-- Inline edit mode -->
                  <input
                    class="form-input prop-input"
                    bind:value={editingPropDraft}
                    onblur={() => commitPropEdit(node)}
                    onkeydown={e => {
                      if (e.key === 'Enter') { e.preventDefault(); commitPropEdit(node); }
                      if (e.key === 'Escape') cancelPropEdit();
                    }}
                    use:focusMounted
                  />
                {:else}
                  <span
                    class="prop-val"
                    role="button"
                    tabindex="0"
                    title="Click to edit"
                    onclick={() => startEditProp(k, v as string)}
                    onkeydown={e => e.key === 'Enter' && startEditProp(k, v as string)}
                  >{v}</span>                {/if}
              </div>

              <div class="prop-actions" role="cell">
                {#if editingPropKey === k}
                  <button class="icon-btn success-icon" onclick={() => commitPropEdit(node)} title="Confirm" aria-label="Confirm edit">
                    <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><polyline points="20 6 9 17 4 12"/></svg>
                  </button>
                  <button class="icon-btn" onclick={cancelPropEdit} title="Cancel" aria-label="Cancel edit">
                    <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                  </button>
                {:else}
                  <button
                    class="icon-btn danger-icon"
                    onclick={() => handleDeleteProp(node, k)}
                    disabled={busy.deleteProp}
                    title="Delete property"
                    aria-label="Delete property {k}"
                  >
                    <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14H6L5 6"/></svg>
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        </div>

        <!-- Add new property -->
        <div class="add-prop-form">
          <input
            class="form-input"
            placeholder="key"
            bind:value={newPropKey}
            onkeydown={e => e.key === 'Enter' && handleAddProp(node)}
          />
          <input
            class="form-input"
            placeholder="value"
            bind:value={newPropVal}
            onkeydown={e => e.key === 'Enter' && handleAddProp(node)}
          />
          <button
            class="btn btn-primary btn-sm"
            onclick={() => handleAddProp(node)}
            disabled={!newPropKey.trim() || busy.updateProp}
            title="Add property"
          >
            {#if busy.updateProp}<span class="spinner spinner-sm" aria-hidden="true"></span>{:else}+{/if}
          </button>
        </div>
      {:else}
        <!-- Multi-select summary -->
        <div class="multi-select-info">
          <p>{nodes.length} nodes selected.</p>
          <div class="nodes-list">
            {#each nodes.slice(0, 10) as n}
              <div class="mini-node">
                <span class="type-dot type-{n.type.toLowerCase()}"></span>
                {nodeDisplayName(n)}
              </div>
            {/each}
            {#if nodes.length > 10}
              <div class="more-hint">…and {nodes.length - 10} more</div>
            {/if}
          </div>
        </div>
      {/if}

      <!-- Provenance information (single node only) -->
      {#if nodes.length === 1}
        <div class="provenance-section-wrapper">
          <ProvenancePanel {node} />
        </div>
      {/if}
    </section>

  {:else if $selectedEdges.length > 0}
    {@const edges = $selectedEdges}
    {@const edge = edges[0]}

    <!-- Resolve node display names for source / target -->
    {@const srcNode = $graphStore.nodes.find((n: Node) => n.id === edge.source)}
    {@const tgtNode = $graphStore.nodes.find((n: Node) => n.id === edge.target)}

    <section class="sidebar-section detail-panel">
      <div class="detail-header">
        <div class="detail-header-left">
          {#if edges.length === 1}
            <span class="type-badge type-edge">Edge</span>
            <span class="detail-id" title={edge.id}>{edge.id.slice(0, 8)}…</span>
          {:else}
            <span class="type-badge type-edge">{edges.length} Edges</span>
            <span class="detail-id">Multiple Selection</span>
          {/if}
        </div>
        <button
          class="btn btn-danger btn-sm"
          onclick={handleRemoveEdge}
          disabled={busy.removeEdge}
          title="Delete edge"
        >
          {#if busy.removeEdge}<span class="spinner spinner-sm" aria-hidden="true"></span>
          {:else}Delete{/if}
        </button>
      </div>

      {#if edges.length === 1}
        <!-- Edge endpoint summary -->
        <div class="edge-summary">
          <div class="edge-ep">
            <span class="ep-label">From</span>
            <span class="ep-value">{srcNode ? nodeDisplayName(srcNode) : edge.source.slice(0,8)}</span>
          </div>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="var(--text-muted)" stroke-width="2" aria-hidden="true">
            <line x1="5" y1="12" x2="19" y2="12"/><polyline points="12 5 19 12 12 19"/>
          </svg>
          <div class="edge-ep">
            <span class="ep-label">To</span>
            <span class="ep-value">{tgtNode ? nodeDisplayName(tgtNode) : edge.target.slice(0,8)}</span>
          </div>
        </div>

        <div class="props-table-row edge-label-row" role="row">
          <span class="prop-key" role="cell">label</span>
          <span class="prop-val" role="cell">{edge.label}</span>
          <span role="cell"></span>
        </div>

        {#each Object.entries(edge.properties) as [k, v]}
          <div class="props-table-row" role="row">
            <span class="prop-key" role="cell">{k}</span>
            <span class="prop-val" role="cell">{v}</span>
            <span role="cell"></span>
          </div>
        {/each}
      {/if}
    </section>
  {/if}

</aside>

<!-- ═══════════════════════════════════════════════════════════════════════ -->
<style>
  /* ── Shell ─────────────────────────────────────────────────────────────── */
  .sidebar {
    width: var(--sidebar-width);
    height: 100%;
    background: var(--bg-surface);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    overflow-x: hidden;
    flex-shrink: 0;
  }

  /* ── Header ────────────────────────────────────────────────────────────── */
  .sidebar-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 12px;
    height: var(--topbar-height);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .logo-mark { font-size: 17px; color: var(--accent); line-height: 1; }
  .app-name  { font-size: 13px; font-weight: 700; color: var(--text-primary); flex: 1; letter-spacing: -.01em; }

  /* ── Section chrome ────────────────────────────────────────────────────── */
  .sidebar-section {
    padding: 10px 12px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .section-toggle {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    margin-bottom: 0;
  }
  .section-toggle:hover .section-title { color: var(--text-secondary); }

  .section-title {
    font-size: var(--font-size-xs);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: .07em;
    color: var(--text-muted);
    margin: 0;
    pointer-events: none;
  }

  /* standalone (non-collapsible) section heading */
  .sidebar-section > h2.section-title {
    margin-bottom: 8px;
  }

  .chevron {
    color: var(--text-muted);
    transition: transform 150ms;
    flex-shrink: 0;
  }
  .chevron.open { transform: rotate(180deg); }

  .section-body {
    margin-top: 10px;
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  /* ── Buttons ───────────────────────────────────────────────────────────── */
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    padding: 5px 10px;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
    font-family: var(--font-sans);
    cursor: pointer;
    border: 1px solid transparent;
    transition: background 120ms, border-color 120ms, color 120ms, opacity 120ms;
    white-space: nowrap;
  }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }

  .btn-primary {
    background: var(--accent); color: #fff; border-color: var(--accent);
  }
  .btn-primary:not(:disabled):hover { background: var(--accent-hover); border-color: var(--accent-hover); }

  .btn-secondary {
    background: var(--bg-elevated); color: var(--text-secondary); border-color: var(--border);
  }
  .btn-secondary:not(:disabled):hover { background: var(--bg-hover); color: var(--text-primary); border-color: var(--border-focus); }

  .btn-ghost {
    background: none; color: var(--text-secondary); border-color: var(--border);
  }
  .btn-ghost:not(:disabled):hover { background: var(--bg-hover); color: var(--text-primary); }

  .btn-danger {
    background: none; color: var(--accent-danger); border-color: color-mix(in srgb, var(--accent-danger) 40%, transparent);
  }
  .btn-danger:not(:disabled):hover { background: var(--accent-danger); color: #fff; border-color: var(--accent-danger); }

  .btn-full  { width: 100%; }
  .btn-sm    { padding: 3px 8px; font-size: var(--font-size-xs); }
  .mt-sm     { margin-top: 8px; }

  .btn-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4px;
  }

  /* ── Icon-only button ──────────────────────────────────────────────────── */
  .icon-btn {
    display: inline-flex; align-items: center; justify-content: center;
    width: 22px; height: 22px; padding: 0; flex-shrink: 0;
    background: none; border: none; cursor: pointer; border-radius: var(--radius-sm);
    color: var(--text-secondary);
    transition: background 100ms, color 100ms;
  }
  .icon-btn:hover           { background: var(--bg-hover); color: var(--text-primary); }
  .icon-btn.danger-icon:hover  { background: var(--accent-danger); color: #fff; }
  .icon-btn.success-icon:hover { background: var(--accent-success); color: #fff; }
  .icon-btn:disabled        { opacity: 0.4; cursor: not-allowed; }

  /* ── Spinner ───────────────────────────────────────────────────────────── */
  .spinner {
    display: inline-block;
    width: 12px; height: 12px;
    border: 2px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
    flex-shrink: 0;
  }
  .spinner-sm { width: 9px; height: 9px; border-width: 1.5px; }
  @keyframes spin { to { transform: rotate(360deg); } }

  /* ── Stats bar ─────────────────────────────────────────────────────────── */
  .stats-bar {
    display: flex;
    gap: 12px;
    padding: 7px 12px;
  }
  .stat-chip {
    display: flex; align-items: center; gap: 5px;
  }
  .stat-dot {
    width: 7px; height: 7px; border-radius: 50%; flex-shrink: 0;
  }
  .dot-node { background: var(--accent); }
  .dot-edge { background: var(--text-muted); }
  .stat-num  { font-size: var(--font-size-sm); font-weight: 600; color: var(--text-primary); font-family: var(--font-mono); }
  .stat-lbl  { font-size: var(--font-size-xs); color: var(--text-muted); }

  /* ── Form elements ─────────────────────────────────────────────────────── */
  .form-group { margin-bottom: 7px; }
  .form-label { display: block; font-size: var(--font-size-xs); color: var(--text-muted); margin-bottom: 3px; }

  .form-input, .form-select {
    width: 100%; padding: 5px 7px;
    background: var(--bg-base); border: 1px solid var(--border);
    border-radius: var(--radius-sm); color: var(--text-primary);
    font-size: var(--font-size-sm); font-family: var(--font-sans);
    outline: none; transition: border-color 120ms;
    box-sizing: border-box;
  }
  .form-input:focus, .form-select:focus { border-color: var(--border-focus); }

  /* ── Extra-props rows (add-node form) ──────────────────────────────────── */
  .extra-props { display: flex; flex-direction: column; gap: 4px; margin-bottom: 6px; }
  .extra-prop-row { display: flex; gap: 4px; align-items: center; }
  .extra-prop-row .form-input { flex: 1; min-width: 0; }

  /* ── Edge arrow ────────────────────────────────────────────────────────── */
  .edge-arrow { display: flex; justify-content: center; margin: -2px 0; }

  /* ── Hint text ─────────────────────────────────────────────────────────── */
  .hint { font-size: var(--font-size-xs); color: var(--text-muted); text-align: center; padding: 4px 0; }

  /* ── Detail panel ──────────────────────────────────────────────────────── */
  .detail-panel { flex: 1; min-height: 0; }

  .detail-header {
    display: flex; align-items: center; justify-content: space-between;
    margin-bottom: 10px; gap: 6px;
  }
  .detail-header-left { display: flex; align-items: center; gap: 6px; min-width: 0; }

  .type-badge {
    display: inline-block; padding: 2px 7px; flex-shrink: 0;
    border-radius: 999px; font-size: var(--font-size-xs); font-weight: 600;
    border: 1px solid currentColor;
  }
  /* per-type colours matching the Graph stylesheet */
  .type-ip           { color: #4a7cf6; border-color: #4a7cf6; background: color-mix(in srgb, #4a7cf6 12%, transparent); }
  .type-domain       { color: #3dba7e; border-color: #3dba7e; background: color-mix(in srgb, #3dba7e 12%, transparent); }
  .type-email        { color: #9f7aea; border-color: #9f7aea; background: color-mix(in srgb, #9f7aea 12%, transparent); }
  .type-person       { color: #e0a33d; border-color: #e0a33d; background: color-mix(in srgb, #e0a33d 12%, transparent); }
  .type-url          { color: #22c55e; border-color: #22c55e; background: color-mix(in srgb, #22c55e 12%, transparent); }
  .type-organisation { color: #a855f7; border-color: #a855f7; background: color-mix(in srgb, #a855f7 12%, transparent); }
  .type-hash              { color: #eab308; border-color: #eab308; background: color-mix(in srgb, #eab308 12%, transparent); }
  .type-location          { color: #14b8a6; border-color: #14b8a6; background: color-mix(in srgb, #14b8a6 12%, transparent); }
  .type-ipv6              { color: #60a5fa; border-color: #60a5fa; background: color-mix(in srgb, #60a5fa 12%, transparent); }
  .type-virustotalreport  { color: #ef4444; border-color: #ef4444; background: color-mix(in srgb, #ef4444 12%, transparent); }
  .type-edge              { color: var(--text-muted); border-color: var(--border); background: var(--bg-elevated); }

  .detail-id {
    font-size: 10px; color: var(--text-muted); font-family: var(--font-mono);
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }

  /* ── Properties table ──────────────────────────────────────────────────── */
  .props-table { margin-bottom: 8px; }

  .props-table-head {
    display: grid; grid-template-columns: 72px 1fr 44px;
    padding: 2px 0 4px;
    font-size: 10px; font-weight: 600; text-transform: uppercase;
    letter-spacing: .05em; color: var(--text-muted);
    border-bottom: 1px solid var(--border);
  }

  .props-table-row {
    display: grid; grid-template-columns: 72px 1fr 44px;
    align-items: center; gap: 4px;
    padding: 4px 0;
    border-bottom: 1px solid color-mix(in srgb, var(--border) 50%, transparent);
    font-size: var(--font-size-xs);
  }
  .edge-label-row { opacity: 0.7; }

  .prop-key {
    color: var(--text-muted); overflow: hidden; text-overflow: ellipsis;
    white-space: nowrap; padding-right: 4px;
  }

  .prop-val-cell { min-width: 0; }
  .prop-val {
    color: var(--text-primary); word-break: break-all;
    cursor: pointer; border-radius: 2px;
    padding: 1px 3px; margin: -1px -3px;
    transition: background 100ms;
    display: block;
  }
  .prop-val:hover { background: var(--bg-hover); }

  .prop-input { padding: 2px 5px; font-size: var(--font-size-xs); }

  .prop-actions {
    display: flex; align-items: center; justify-content: flex-end; gap: 2px;
  }

  /* ── Add-prop form ─────────────────────────────────────────────────────── */
  .add-prop-form {
    display: flex; gap: 4px; align-items: center; margin-top: 6px;
  }
  .add-prop-form .form-input { flex: 1; min-width: 0; }

  /* ── Edge detail ───────────────────────────────────────────────────────── */
  .edge-summary {
    display: flex; align-items: center; gap: 6px;
    padding: 6px 0 8px; margin-bottom: 4px;
    border-bottom: 1px solid var(--border);
  }
  .edge-ep { display: flex; flex-direction: column; flex: 1; min-width: 0; }
  .ep-label { font-size: 10px; color: var(--text-muted); text-transform: uppercase; letter-spacing: .05em; }
  .ep-value { font-size: var(--font-size-xs); color: var(--text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  /* ── Multi-select ──────────────────────────────────────────────────────── */
  .multi-select-info {
    padding: 10px 0;
  }
  .multi-select-info p {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    margin: 0 0 10px;
  }
  .nodes-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 200px;
    overflow-y: auto;
    background: var(--bg-base);
    padding: 6px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
  }
  .mini-node {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .type-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
    border: 1px solid rgba(255,255,255,0.1);
  }
  /* dots match type colors */
  .type-ip           { background: #4a7cf6; }
  .type-domain       { background: #3dba7e; }
  .type-email        { background: #9f7aea; }
  .type-person       { background: #e0a33d; }
  .type-url          { background: #22c55e; }
  .type-organisation { background: #a855f7; }
  .type-hash         { background: #eab308; }
  .type-location     { background: #14b8a6; }
  .type-collection   { background: var(--accent); }

  .more-hint {
    font-size: 10px;
    color: var(--text-muted);
    padding-top: 4px;
    text-align: center;
  }

  .header-actions {
    display: flex;
    gap: 4px;
  }

  /* Inline label input next to the Collapse button (window.prompt is dead
     in Tauri v2). */
  .collapse-label {
    flex: 1;
    min-width: 70px;
    padding: 2px 5px;
    font-size: var(--font-size-xs);
  }

  /* ── Provenance panel ─────────────────────────────────────────────────── */
  .provenance-section-wrapper {
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid var(--border);
  }
</style>
