<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import cytoscape from 'cytoscape';
  import type { Core, ElementDefinition } from 'cytoscape';
  import coseBilkent from 'cytoscape-cose-bilkent';
  import dagre from 'cytoscape-dagre';
  import { invoke } from '@tauri-apps/api/core';
  import {
    graphStore,
    selectionStore,
    themeStore,
    setStatus,
    cleanupStatus,
  } from '$lib/stores';
  import { filteredGraph, filterCriteria } from '$lib/filterStore';
  import type { Node as GNode, Edge as GEdge, Graph, TransformInfo, TransformResult, Machine, EntityDefinition } from '$lib/stores';
  import ContextMenu from '$lib/components/ContextMenu.svelte';

  // Register extensions
  cytoscape.use(coseBilkent);
  cytoscape.use(dagre);

  // ── Cytoscape instance ────────────────────────────────────────────────────

  let container: HTMLDivElement;
  let cy: Core | null = null;
  let mounted = false;

  // ── Registries (loaded once on mount) ───────────────────────────────────

  let allTransforms: TransformInfo[] = $state([]);
  let allMachines: Machine[] = $state([]);
  let allEntities: EntityDefinition[] = $state([]);

  /** Transforms applicable to a given node type. */
  function transformsFor(nodeType: string): TransformInfo[] {
    return allTransforms.filter(t =>
      t.applicable_types.length === 0 || t.applicable_types.includes(nodeType)
    );
  }

  /** All machines (automation sequences) are offered for every node type;
   *  each machine internally skips transforms that don't apply. */

  // ── Context menu state ────────────────────────────────────────────────────

  interface ContextMenuState {
    x: number; y: number;
    nodeId: string; nodeType: string; nodeLabel: string;
    transforms: TransformInfo[];
    machines: Machine[];
  }

  let ctxMenu: ContextMenuState | null = $state(null);

  function openContextMenu(
    x: number, y: number,
    nodeId: string, nodeType: string, nodeLabel: string,
  ) {
    ctxMenu = { 
      x, y, 
      nodeId, nodeType, nodeLabel, 
      transforms: transformsFor(nodeType),
      machines: allMachines
    };
  }

  function closeContextMenu() { ctxMenu = null; }

  // ── Running a transform ───────────────────────────────────────────────────

  async function handleRunTransform(nodeId: string, transformName: string) {
    setStatus('info', `Running ${transformName}…`);
    try {
      const result = await invoke<TransformResult>('run_transform', {
        nodeId,
        transformName,
      });
      // Merge new elements into the store.
      for (const n of result.new_nodes) graphStore.addNode(n);
      for (const e of result.new_edges) graphStore.addEdge(e);
      setStatus('success',
        `${transformName}: +${result.new_nodes.length} node(s), +${result.new_edges.length} edge(s)`
      );
    } catch (e) {
      setStatus('error', String(e));
    } finally {
      closeContextMenu();
    }
  }

  async function handleRunMachine(nodeId: string, machineName: string) {
    setStatus('info', `Executing Machine: ${machineName}…`);
    try {
      const result = await invoke<TransformResult & { errors?: string[] }>('run_machine', {
        nodeId,
        machineName,
      });
      // Merge new elements into the store.
      for (const n of result.new_nodes) graphStore.addNode(n);
      for (const e of result.new_edges) graphStore.addEdge(e);
      if (result.errors?.length) {
        setStatus('error',
          `Machine '${machineName}': ${result.errors.join('; ')}`
        );
      } else {
        setStatus('success',
          `Machine '${machineName}': +${result.new_nodes.length} node(s), +${result.new_edges.length} edge(s)`
        );
      }
    } catch (e) {
      setStatus('error', String(e));
    } finally {
      closeContextMenu();
    }
  }

  // ── Graph-delta tracking ──────────────────────────────────────────────────
  //
  // We maintain shadow maps of what is currently rendered so that each store
  // emission is processed as a precise delta (add / update / remove) instead
  // of a full tear-down and rebuild.  This keeps Cytoscape animations smooth
  // when only one element changes.

  let renderedNodes = new Map<string, GNode>();
  let renderedEdges = new Map<string, GEdge>();

  function nodeLabel(n: GNode): string {
    return n.properties['label'] ?? n.properties['value'] ?? n.type;
  }

  function nodeData(n: GNode) {
    const degree = cy?.$id(n.id).degree() ?? 0;

    // Select main icon from registry
    const entity = allEntities.find(e => e.node_type === n.type);
    let icon = '';
    if (entity) {
      switch (entity.icon) {
        case 'ip': icon = iconIP; break;
        case 'domain': icon = iconDomain; break;
        case 'email': icon = iconEmail; break;
        case 'person': icon = iconPerson; break;
        case 'url': icon = iconURL; break;
        case 'org': icon = iconOrg; break;
        case 'hash': icon = iconHash; break;
        case 'location': icon = iconLocation; break;
        case 'phone': icon = iconPhone; break;
        case 'crypto': icon = iconCrypto; break;
        case 'social': icon = iconSocial; break;
        case 'image': icon = iconImage; break;
        case 'vt': icon = iconVT; break;
        case 'collection': icon = iconCollection; break;
        default: 
          // Fallback: use a generic icon if entity icon is unknown
          console.warn(`[Graph] Unknown icon type '${entity.icon}' for node type '${n.type}'`);
          icon = iconCollection;
      }
    }

    const badge1 = n.properties['country_code'] ? iconFlag : 'none';
    const badge2 = degree > 10 ? iconHub : 'none';

    return {
      id: n.id,
      label: nodeLabel(n),
      type: n.type,
      degree,
      icon,
      badge1,
      badge2,
      ...n.properties
    };
  }

  function edgeData(e: GEdge) {
    return {
      id: e.id,
      source: e.source,
      target: e.target,
      label: e.label,
      ...e.properties,
    };
  }

  /**
   * Apply a precise delta to the Cytoscape canvas inside a single batch.
   *
   * `cy.batch()` defers all style recalculations until the callback returns,
   * so N element changes cost one repaint instead of N.  Returns counts so
   * the caller can decide whether to trigger a layout pass.
   */
  function applyDelta(graph: Graph): { addedNodes: number; addedEdges: number } {
    if (!cy) return { addedNodes: 0, addedEdges: 0 };

    const newNodeMap = new Map(graph.nodes.map(n => [n.id, n]));
    const newEdgeMap = new Map(graph.edges.map(e => [e.id, e]));

    let addedNodes = 0;
    let addedEdges = 0;

    cy.batch(() => {
      // ── Removals (edges first to avoid dangling references) ─────────────
      for (const [id] of renderedEdges) {
        if (!newEdgeMap.has(id)) cy!.$id(id).remove();
      }
      for (const [id] of renderedNodes) {
        if (!newNodeMap.has(id)) cy!.$id(id).remove();
      }

      // ── Node adds / updates ───────────────────────────────────────────
      for (const [id, node] of newNodeMap) {
        const data = nodeData(node);
        if (renderedNodes.has(id)) {
          cy!.$id(id).data(data);
        } else {
          cy!.add({ group: 'nodes', data } as ElementDefinition);
          addedNodes++;
        }
      }

      // ── Edge adds / updates ───────────────────────────────────────────
      for (const [id, edge] of newEdgeMap) {
        const data = edgeData(edge);
        if (renderedEdges.has(id)) {
          cy!.$id(id).data(data);
        } else {
          cy!.add({ group: 'edges', data } as ElementDefinition);
          addedEdges++;
        }
      }
    });

    renderedNodes = newNodeMap;
    renderedEdges = newEdgeMap;

    return { addedNodes, addedEdges };
  }

  // ── rAF-debounced apply ───────────────────────────────────────────────────
  //
  // When multiple store writes arrive in the same JS task (e.g. a transform
  // that adds many nodes), we coalesce them into a single applyDelta call on
  // the next animation frame.  This prevents redundant Cytoscape work while
  // still keeping latency below one frame (~16 ms at 60 fps).

  const LAYOUT_AUTO_THRESHOLD  = 200; // auto-layout only for small graphs
  const LAYOUT_GRID_THRESHOLD  = 500; // use fast grid layout above this count

  let pendingGraph: Graph | null = null;
  let applyRafId: number | null  = null;

  function scheduleApply(graph: Graph) {
    pendingGraph = graph;
    if (applyRafId !== null) return; // already scheduled for this frame
    applyRafId = requestAnimationFrame(() => {
      applyRafId = null;
      if (!pendingGraph || !mounted) return;
      const g = pendingGraph;
      pendingGraph = null;

      const { addedNodes } = applyDelta(g);

      if (addedNodes === 0) return;
      const total = cy?.nodes().length ?? 0;

      if (total <= 1) {
        cy?.fit(undefined, 48);
      } else if (total <= LAYOUT_AUTO_THRESHOLD) {
        // Small graph — auto-layout after a tick so all adds are flushed.
        setTimeout(() => runLayout(), 0);
      } else {
        // Large graph — skip auto-layout; notify the user.
        setStatus(
          'info',
          `${total} nodes — press L to run layout (may take a moment)`,
          5000,
        );
      }
    });
  }

  // ── Stylesheet factory ────────────────────────────────────────────────────
  //
  // All colours are read from live CSS custom properties so that the stylesheet
  // automatically respects whichever theme is active on document.documentElement
  // at call time.  The caller must set data-theme on the html element *before*
  // invoking this function (done in the theme subscription below).

  function svgDataUri(svg: string) {
    return 'data:image/svg+xml;utf8,' + encodeURIComponent(svg);
  }

  const iconDomain = svgDataUri(`<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"/><path d="M2 12h20"/></svg>`);
  const iconIP = svgDataUri(`<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="2" width="20" height="8" rx="2" ry="2"/><rect x="2" y="14" width="20" height="8" rx="2" ry="2"/><line x1="6" y1="6" x2="6.01" y2="6"/><line x1="6" y1="18" x2="6.01" y2="18"/></svg>`);
  const iconPerson = svgDataUri(`<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>`);
  const iconEmail = svgDataUri(`<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="20" height="16" x="2" y="4" rx="2"/><path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7"/></svg>`);
  const iconURL = svgDataUri(`<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>`);
  const iconOrg = svgDataUri(`<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="16" height="20" x="4" y="2" rx="2" ry="2"/><path d="M9 22v-4h6v4"/><path d="M8 6h.01"/><path d="M16 6h.01"/><path d="M12 6h.01"/><path d="M12 10h.01"/><path d="M12 14h.01"/><path d="M16 10h.01"/><path d="M16 14h.01"/><path d="M8 10h.01"/><path d="M8 14h.01"/></svg>`);
  const iconLocation = svgDataUri(`<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 10c0 6-8 12-8 12s-8-6-8-12a8 8 0 0 1 16 0Z"/><circle cx="12" cy="10" r="3"/></svg>`);
  const iconPhone = svgDataUri(`<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z"/></svg>`);
  const iconCrypto = svgDataUri(`<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="8" cy="8" r="6"/><path d="M18.09 10.37A6 6 0 1 1 10.34 18"/><path d="M7 6h1v4"/><path d="M16.71 13.88.49 10.22"/></svg>`);
  const iconSocial = svgDataUri(`<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>`);
  const iconImage = svgDataUri(`<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2" ry="2"/><circle cx="9" cy="9" r="2"/><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/></svg>`);
  const iconFlag = svgDataUri(`<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 15s1-1 4-1 5 2 8 2 4-1 4-1V3s-1 1-4 1-5-2-8-2-4 1-4 1z"/><line x1="4" x2="4" y1="22" y2="15"/></svg>`);
  const iconHub = svgDataUri(`<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z"/><path d="m9 12 2 2 4-4"/></svg>`);
  const iconCollection = svgDataUri(`<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M7 7h10"/><path d="M7 12h10"/><path d="M7 17h10"/></svg>`);
  const iconVT = svgDataUri(`<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>`);
  const iconHash = svgDataUri(`<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="4" x2="20" y1="9" y2="9"/><line x1="4" x2="20" y1="15" y2="15"/><line x1="10" x2="8" y1="3" y2="21"/><line x1="16" x2="14" y1="3" y2="21"/></svg>`);

  function buildStylesheet(): cytoscape.StylesheetStyle[] {
    const root = document.documentElement;
    const v = (name: string) =>
      getComputedStyle(root).getPropertyValue(name).trim();

    const canvas   = v('--canvas-bg');
    const surface  = v('--bg-surface');
    const accent   = v('--accent');
    const text     = v('--text-primary');
    const muted    = v('--text-secondary');
    const edgeLine = v('--cyto-edge-line');
    const selBorder = v('--cyto-selected-border');
    const selBg    = v('--cyto-selected-bg');
    const ghost    = v('--cyto-ghost');

    return [
      // ── Base node ──────────────────────────────────────────────────────
      {
        selector: 'node',
        style: {
          width: 44,
          height: 44,
          'background-color': surface,
          'border-color': accent,
          'border-width': 2,
          label: 'data(label)',
          color: text,
          'font-size': 11,
          'font-family': '-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
          'text-valign': 'bottom',
          'text-halign': 'center',
          'text-margin-y': 5,
          'text-outline-color': canvas,
          'text-outline-width': 2,
          // LOD: Cytoscape hides the label automatically when the rendered
          // font size drops below this threshold (zoomed far out).
          'min-zoomed-font-size': 7,
          shape: 'ellipse',
          'transition-property': 'background-color, border-color, border-width, opacity',
          'transition-duration': 150,
          
          // Multiple background images
          'background-image': ['data(icon)', 'data(badge1)', 'data(badge2)'],
          'background-width': ['24px', '14px', '14px'],
          'background-height': ['24px', '14px', '14px'],
          'background-position-x': ['50%', '100%', '0%'],
          'background-position-y': ['50%', '0%', '0%'],
          'background-clip': 'node',
        } as cytoscape.Css.Node,
      },

      // ── Per-type overrides (generated from entity registry) ────────────
      ...allEntities.map(e => ({
        selector: `node[type = "${e.node_type}"]`,
        style: {
          'background-color': v(e.color),
          'border-color': v(e.color.replace('-bg', '-border')),
          shape: e.shape,
          // Custom style for Collections
          ...(e.node_type === 'Collection' ? { 'border-width': 3, 'border-style': 'double' } : {})
        } as cytoscape.Css.Node,
      })),

      // ── States ─────────────────────────────────────────────────────────
      {
        selector: 'node:selected',
        style: {
          'border-color': selBorder,
          'border-width': 3,
          'background-color': selBg,
          'overlay-color': selBorder,
          'overlay-opacity': 0.08,
          'overlay-padding': 6,
        } as cytoscape.Css.Node,
      },
      {
        selector: 'node:grabbed',
        style: { opacity: 0.82 } as cytoscape.Css.Node,
      },
      {
        selector: 'node.highlighted',
        style: {
          'border-width': 3,
          'border-color': accent,
          'background-color': ghost,
        } as cytoscape.Css.Node,
      },
      {
        selector: 'node.dimmed',
        style: { opacity: 0.25 } as cytoscape.Css.Node,
      },

      // ── Base edge ──────────────────────────────────────────────────────
      {
        selector: 'edge',
        style: {
          width: 1.5,
          'line-color': edgeLine,
          'target-arrow-color': edgeLine,
          'target-arrow-shape': 'triangle',
          'curve-style': 'bezier',
          label: 'data(label)',
          'font-size': 10,
          color: muted,
          'text-background-color': canvas,
          'text-background-opacity': 0.85,
          'text-background-padding': '2px',
          'text-rotation': 'autorotate',
          // Edge labels vanish sooner than node labels when zooming out.
          'min-zoomed-font-size': 10,
          'transition-property': 'line-color, target-arrow-color, width, opacity',
          'transition-duration': 150,
        } as cytoscape.Css.Edge,
      },
      {
        selector: 'edge:selected',
        style: {
          'line-color': selBorder,
          'target-arrow-color': selBorder,
          width: 2.5,
          'overlay-color': selBorder,
          'overlay-opacity': 0.08,
          'overlay-padding': 4,
        } as cytoscape.Css.Edge,
      },
      {
        selector: 'edge.dimmed',
        style: { opacity: 0.15 } as cytoscape.Css.Edge,
      },
    ];
  }

  // ── Layout ────────────────────────────────────────────────────────────────

  let activeLayout: 'cose-bilkent' | 'dagre' = $state('cose-bilkent');

  const COSE_BILKENT_OPTIONS = {
    name: 'cose-bilkent',
    animate: 'end' as const,
    animationEasing: 'ease-out' as const,
    animationDuration: 500,
    fit: true,
    padding: 48,
    // forces
    nodeRepulsion: 6500,
    idealEdgeLength: 120,
    edgeElasticity: 0.45,
    nestingFactor: 0.1,
    gravity: 0.25,
    numIter: 2500,
    tile: true,
    tilingPaddingVertical: 10,
    tilingPaddingHorizontal: 10,
    // convergence
    gravityRangeCompound: 1.5,
    gravityCompound: 1.0,
    gravityRange: 3.8,
    initialEnergyOnIncremental: 0.5,
  };

  const DAGRE_OPTIONS = {
    name: 'dagre',
    animate: 'end' as const,
    animationDuration: 500,
    fit: true,
    padding: 48,
    nodeSep: 60,
    edgeSep: 40,
    rankSep: 80,
    rankDir: 'TB', // Top to Bottom
  };

  /**
   * Public: run the best layout for the current graph size.
   *
   * - ≤ LAYOUT_GRID_THRESHOLD nodes: current active layout (cose-bilkent or dagre)
   * - >  LAYOUT_GRID_THRESHOLD nodes: grid (O(n), instant)
   */
  export function runLayout() {
    if (!cy || cy.nodes().length === 0) return;
    const n = cy.nodes().length;
    if (n > LAYOUT_GRID_THRESHOLD) {
      setStatus('info', `Grid layout (${n} nodes)`, 3000);
      cy.layout({
        name: 'grid',
        fit: true,
        padding: 48,
        avoidOverlap: true,
        condense: false,
      } as cytoscape.LayoutOptions).run();
    } else {
      const opts = activeLayout === 'dagre' ? DAGRE_OPTIONS : COSE_BILKENT_OPTIONS;
      cy.layout(opts as cytoscape.LayoutOptions).run();
    }
  }

  export function cycleLayout() {
    activeLayout = activeLayout === 'cose-bilkent' ? 'dagre' : 'cose-bilkent';
    setStatus('info', `Switched layout to ${activeLayout === 'dagre' ? 'Hierarchical' : 'Force-Directed'}`);
    runLayout();
  }

  // ── Viewport helpers (exported for parent use) ────────────────────────────

  export function fit() {
    cy?.fit(undefined, 48);
  }

  export function zoomIn() {
    if (!cy) return;
    cy.zoom({
      level: Math.min(cy.zoom() * 1.25, cy.maxZoom()),
      renderedPosition: { x: cy.width() / 2, y: cy.height() / 2 },
    });
  }

  export function zoomOut() {
    if (!cy) return;
    cy.zoom({
      level: Math.max(cy.zoom() * 0.8, cy.minZoom()),
      renderedPosition: { x: cy.width() / 2, y: cy.height() / 2 },
    });
  }

  // ── Lifecycle ─────────────────────────────────────────────────────────────

  let unsubGraph: (() => void) | null = null;
  let unsubTheme: (() => void) | null = null;

  // Listen for keyboard shortcut events
  function handleKeyboardEvents(e: Event) {
    if (!cy || !mounted) return;
    const target = e.target as HTMLElement;
    if (
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement ||
      target.isContentEditable
    ) return;

    switch (e.type) {
      case 'graph-layout-auto':
        runLayout();
        break;
      case 'graph-layout-cycle':
        cycleLayout();
        break;
      case 'graph-zoom-in':
        zoomIn();
        break;
      case 'graph-zoom-out':
        zoomOut();
        break;
      case 'graph-select-all':
        // Select all visible nodes
        const allNodeIds = cy.nodes().map(n => n.id());
        if (allNodeIds.length > 0) {
          selectionStore.set({ kind: 'node', ids: allNodeIds });
        }
        break;
      case 'graph-deselect-all':
        selectionStore.set(null);
        break;
    }
  }

  onMount(async () => {
    // 1. Create Cytoscape instance (empty canvas).
    // Ensure data-theme is on <html> before reading CSS vars for the stylesheet.
    document.documentElement.setAttribute('data-theme', $themeStore);
    cy = cytoscape({
      container,
      elements: [],
      style: buildStylesheet(),
      layout: { name: 'preset' },
      minZoom: 0.05,
      maxZoom: 10,
      wheelSensitivity: 0.25,
      boxSelectionEnabled: true,
      selectionType: 'single',
      // ── Performance options ────────────────────────────────────────────
      // textureOnViewport: render to a bitmap texture while the user pans /
      // zooms, then re-render at full quality when movement stops.  Cuts
      // per-frame cost from O(E+N) to O(1) during interaction.
      textureOnViewport: true,
      // hideEdgesOnViewport: skip edge drawing entirely during pan / zoom.
      // Visually seamless because the texture covers the transition period.
      hideEdgesOnViewport: true,
      // motionBlurEnabled: false avoids an extra off-screen canvas that
      // Cytoscape normally maintains for motion-blur compositing.
      // This is an experimental Cytoscape option not in the type definitions.
      // Using spread operator to bypass TypeScript strict checking.
      ...({ motionBlurEnabled: false } as any),
      // pixelRatio: 'auto' uses devicePixelRatio; we clamp it to 2 to avoid
      // 4× canvas memory cost on high-DPI displays with large graphs.
      pixelRatio: Math.min(window.devicePixelRatio, 2),
    });

    // 2. Wire up pointer events → stores.

    cy.on('tap', 'node', evt => {
      selectionStore.set({ kind: 'node', ids: [evt.target.id()] });
    });

    cy.on('tap', 'edge', evt => {
      selectionStore.set({ kind: 'edge', ids: [evt.target.id()] });
    });

    // Tap on empty canvas → deselect.
    cy.on('tap', evt => {
      if (evt.target === cy) selectionStore.set(null);
    });

    // Mirror Cytoscape's own box/click-select to the store.
    cy.on('select', 'node', _evt => {
      const ids = cy!.$('node:selected').map(n => n.id());
      selectionStore.set({ kind: 'node', ids });
    });
    cy.on('select', 'edge', _evt => {
      const ids = cy!.$('edge:selected').map(e => e.id());
      selectionStore.set({ kind: 'edge', ids });
    });
    cy.on('unselect', () => {
      if (!cy) return;
      const nodes = cy.$('node:selected');
      const edges = cy.$('edge:selected');
      if (nodes.length > 0) {
        selectionStore.set({ kind: 'node', ids: nodes.map(n => n.id()) });
      } else if (edges.length > 0) {
        selectionStore.set({ kind: 'edge', ids: edges.map(e => e.id()) });
      } else {
        selectionStore.set(null);
      }
    });

    // Right-click on node → context menu.
    cy.on('cxttap', 'node', evt => {
      // Prevent the browser's native context menu over the canvas.
      evt.originalEvent?.preventDefault();

      const node   = evt.target;
      const nodeId = node.id() as string;
      const nodeType  = (node.data('type')  as string) ?? 'Unknown';
      const nodeLabel = (node.data('label') as string) ?? nodeId;

      // Convert Cytoscape rendered position to viewport coordinates.
      const canvasRect = container.getBoundingClientRect();
      const rp = evt.renderedPosition ?? { x: 0, y: 0 };
      openContextMenu(
        canvasRect.left + rp.x,
        canvasRect.top  + rp.y,
        nodeId, nodeType, nodeLabel,
      );

      // Also select the node so the sidebar updates.
      selectionStore.set({ kind: 'node', ids: [nodeId] });
    });

    // Right-click on background → close menu.
    cy.on('cxttap', evt => {
      if (evt.target === cy) closeContextMenu();
    });

    // 3. Subscribe to theme changes → set data-theme on <html> first (so
    //    getComputedStyle returns correct CSS var values), then re-apply stylesheet.
    unsubTheme = themeStore.subscribe(t => {
      document.documentElement.setAttribute('data-theme', t);
      cy?.style(buildStylesheet());
    });

    // 4. Fetch the persisted graph from the backend.
    let initialGraph: Graph = { nodes: [], edges: [] };
    try {
      initialGraph = await invoke<Graph>('get_graph');
    } catch (e) {
      // Backend may be empty on first run — not an error worth surfacing.
      console.warn('[Graph] get_graph failed (expected on first run):', e);
    }

    // Seed the store (triggers the subscription below immediately).
    if (initialGraph.nodes.length > 0) {
      graphStore.setGraph(initialGraph);
      setStatus('info', `Loaded ${initialGraph.nodes.length} nodes from backend`);
    }

    // 4.5. Fetch the transform registry, machines, and entities once.
    try {
      const [t, m, e] = await Promise.all([
        invoke<TransformInfo[]>('list_transforms'),
        invoke<Machine[]>('list_machines'),
        invoke<EntityDefinition[]>('list_entities'),
      ]);
      allTransforms = t;
      allMachines = m;
      allEntities = e;
    } catch (e) {
      console.warn('[Graph] Registry fetch failed:', e);
    }

    mounted = true;

    // 5. Subscribe to store mutations → incremental canvas updates.
    //    We hold off subscribing until after the initial backend load so the
    //    first applyDelta includes the full set of loaded nodes.
    //    Use scheduleApply so rapid consecutive writes (e.g. transform adding
    //    many nodes) are coalesced into a single rAF-batched repaint.
    unsubGraph = filteredGraph.subscribe(graph => {
      if (!mounted) return;
      scheduleApply(graph);
    });

    // 6. Register keyboard shortcut event listeners
    window.addEventListener('graph-layout-auto', handleKeyboardEvents);
    window.addEventListener('graph-layout-cycle', handleKeyboardEvents);
    window.addEventListener('graph-zoom-in', handleKeyboardEvents);
    window.addEventListener('graph-zoom-out', handleKeyboardEvents);
    window.addEventListener('graph-select-all', handleKeyboardEvents);
    window.addEventListener('graph-deselect-all', handleKeyboardEvents);
  });

  onDestroy(() => {
    mounted = false;
    if (applyRafId !== null) cancelAnimationFrame(applyRafId);
    cleanupStatus();
    unsubGraph?.();
    unsubTheme?.();
    cy?.destroy();
    cy = null;
    // Cleanup keyboard event listeners
    window.removeEventListener('graph-layout-auto', handleKeyboardEvents);
    window.removeEventListener('graph-layout-cycle', handleKeyboardEvents);
    window.removeEventListener('graph-zoom-in', handleKeyboardEvents);
    window.removeEventListener('graph-zoom-out', handleKeyboardEvents);
    window.removeEventListener('graph-select-all', handleKeyboardEvents);
    window.removeEventListener('graph-deselect-all', handleKeyboardEvents);
  });
</script>

<!-- Canvas fills the parent .canvas-area -->
<div class="graph-wrap">
  <div class="graph-canvas" bind:this={container}></div>

  <!-- Floating toolbar ─ bottom-right corner -->
  <div class="graph-toolbar" role="toolbar" aria-label="Graph viewport controls">
    <button onclick={fit} title="Fit to viewport (F)">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M8 3H5a2 2 0 0 0-2 2v3"/>
        <path d="M21 8V5a2 2 0 0 0-2-2h-3"/>
        <path d="M3 16v3a2 2 0 0 0 2 2h3"/>
        <path d="M16 21h3a2 2 0 0 0 2-2v-3"/>
      </svg>
    </button>

    <button onclick={zoomIn} title="Zoom in (+)">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"/>
        <line x1="21" y1="21" x2="16.65" y2="16.65"/>
        <line x1="11" y1="8" x2="11" y2="14"/>
        <line x1="8"  y1="11" x2="14" y2="11"/>
      </svg>
    </button>

    <button onclick={zoomOut} title="Zoom out (-)">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"/>
        <line x1="21" y1="21" x2="16.65" y2="16.65"/>
        <line x1="8"  y1="11" x2="14" y2="11"/>
      </svg>
    </button>

    <div class="toolbar-divider"></div>

    <button onclick={cycleLayout} title="Toggle Layout (H)">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M4 22h14a2 2 0 0 0 2-2V7l-5-5H6a2 2 0 0 0-2 2v4"/>
        <path d="M14 2v4a2 2 0 0 0 2 2h4"/>
        <path d="M3 15h6"/>
        <path d="M3 18h6"/>
        <path d="M11 15h3"/>
      </svg>
    </button>

    <button onclick={runLayout} title="Auto-layout (L)">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="18" cy="5"  r="3"/>
        <circle cx="6"  cy="12" r="3"/>
        <circle cx="18" cy="19" r="3"/>
        <line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/>
        <line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/>
      </svg>
    </button>
  </div>

  <!-- Keyboard shortcut listener (canvas must be focused or use window) -->

  <!-- Context menu (portal rendered at fixed position in viewport) -->
  {#if ctxMenu}
    <ContextMenu
      x={ctxMenu.x}
      y={ctxMenu.y}
      nodeId={ctxMenu.nodeId}
      nodeType={ctxMenu.nodeType}
      nodeLabel={ctxMenu.nodeLabel}
      transforms={ctxMenu.transforms}
      machines={ctxMenu.machines}
      onrun={handleRunTransform}
      onrunmachine={handleRunMachine}
      onclose={closeContextMenu}
    />
  {/if}
</div>

<svelte:window
  onkeydown={e => {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
    // Modifier combos belong to KeyboardShortcuts.svelte; handling plain keys
    // here as well made ⌘F/⌘L/⌘H fire twice per press.
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    if (e.key === 'f' || e.key === 'F') fit();
    if (e.key === '+' || e.key === '=') zoomIn();
    if (e.key === '-') zoomOut();
    if (e.key === 'l' || e.key === 'L') runLayout();
    if (e.key === 'h' || e.key === 'H') cycleLayout();
  }}
/>

<style>
  .graph-wrap {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .graph-canvas {
    width: 100%;
    height: 100%;
    background: var(--canvas-bg);
  }

  /* ── Floating toolbar ──────────────────────────────────────────────────── */
  .graph-toolbar {
    position: absolute;
    bottom: 18px;
    right: 18px;
    display: flex;
    flex-direction: column;
    gap: 3px;
    z-index: 10;
  }

  .graph-toolbar button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    cursor: pointer;
    transition: background 120ms, color 120ms, border-color 120ms, box-shadow 120ms;
  }

  .graph-toolbar button:hover {
    background: var(--bg-hover);
    border-color: var(--border-focus);
    color: var(--text-primary);
    box-shadow: var(--shadow-sm);
  }

  .graph-toolbar button:active {
    transform: scale(0.94);
  }

  .toolbar-divider {
    height: 1px;
    background: var(--border);
    margin: 2px 4px;
  }
</style>
