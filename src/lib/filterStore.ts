import { writable, derived } from 'svelte/store';
import { graphStore } from './stores';
import type { Node, Edge, Graph } from './stores';

export interface FilterCriteria {
  nodeTypes: string[];
  minDegree: number;
  maxDegree: number;
  searchQuery: string;
  /** Filter by tags */
  tags: string[];
  /** Filter by provenance source */
  provenanceSources: string[];
  /** Filter by confidence level (0-100) */
  minConfidence: number;
  /** Filter by date range */
  dateFrom: string | null;
  dateTo: string | null;
  /** Hide isolated nodes (degree 0) */
  hideIsolated: boolean;
  /** Only show nodes with annotations */
  hasAnnotations: boolean;
  /** Invert filter (show matching instead of hiding) */
  invertMode: boolean;
}

const initialCriteria: FilterCriteria = {
  nodeTypes: [], // Empty means all
  minDegree: 0,
  maxDegree: 999,
  searchQuery: '',
  tags: [],
  provenanceSources: [],
  minConfidence: 0,
  dateFrom: null,
  dateTo: null,
  hideIsolated: false,
  hasAnnotations: false,
  invertMode: false,
};

export const filterCriteria = writable<FilterCriteria>(initialCriteria);

/**
 * Derived store that provides the filtered graph.
 * This can be used by the Graph component to hide elements.
 */
export const filteredGraph = derived(
  [graphStore, filterCriteria],
  ([$graph, $criteria]) => {
    const { 
      nodeTypes, 
      minDegree, 
      maxDegree, 
      searchQuery, 
      hideIsolated,
      hasAnnotations,
      minConfidence,
      provenanceSources,
      dateFrom,
      dateTo,
      invertMode 
    } = $criteria;

    // Calculate degrees for all nodes in the original graph
    const degrees = new Map<string, number>();
    for (const edge of $graph.edges) {
      degrees.set(edge.source, (degrees.get(edge.source) || 0) + 1);
      degrees.set(edge.target, (degrees.get(edge.target) || 0) + 1);
    }

    const filteredNodes = $graph.nodes.filter((node: Node) => {
      const degree = degrees.get(node.id) || 0;

      // Every active criterion must pass (AND-ed); invertMode shows the
      // complement (nodes that fail at least one criterion).
      let pass = true;

      if (nodeTypes.length > 0) {
        pass = pass && nodeTypes.includes(node.type);
      }

      if (degree < minDegree || degree > maxDegree) {
        pass = false;
      }

      if (hideIsolated && degree === 0) {
        pass = false;
      }

      if (searchQuery) {
        const q = searchQuery.toLowerCase();
        const matchesSearch =
          node.type.toLowerCase().includes(q) ||
          (node.properties['label'] || '').toLowerCase().includes(q) ||
          (node.properties['value'] || '').toLowerCase().includes(q) ||
          Object.values(node.properties).some(v => v.toLowerCase().includes(q));
        pass = pass && matchesSearch;
      }

      // Provenance-source filter: nodes without provenance count as
      // 'unknown', so they only appear when 'unknown' is selected.
      if (provenanceSources.length > 0) {
        const source = node.provenance?.source ?? 'unknown';
        pass = pass && provenanceSources.includes(source);
      }

      if (minConfidence > 0 && node.provenance) {
        pass = pass && node.provenance.confidence * 100 >= minConfidence;
      }

      if ((dateFrom || dateTo) && node.provenance) {
        const nodeDate = new Date(node.provenance.createdAt).getTime();
        const from = dateFrom ? new Date(dateFrom).getTime() : 0;
        const to = dateTo ? new Date(dateTo).getTime() : Date.now();
        pass = pass && nodeDate >= from && nodeDate <= to;
      }

      return invertMode ? !pass : pass;
    });

    // Filter edges to only include those connecting visible nodes
    const visibleNodeIds = new Set(filteredNodes.map(n => n.id));
    const filteredEdges = $graph.edges.filter((edge: Edge) => 
      visibleNodeIds.has(edge.source) && visibleNodeIds.has(edge.target)
    );

    return {
      nodes: filteredNodes,
      edges: filteredEdges,
    };
  }
);

/**
 * Reset all filters to initial state
 */
export function resetFilters() {
  filterCriteria.set(initialCriteria);
}

/**
 * Quick filter presets
 */
export const FILTER_PRESETS = {
  highConfidence: { minConfidence: 80, name: 'High Confidence Only' },
  recent: { 
    dateFrom: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
    name: 'Last 7 Days' 
  },
  hubs: { minDegree: 5, name: 'Hub Nodes (5+ connections)' },
  isolated: { hideIsolated: true, name: 'Hide Isolated' },
};
