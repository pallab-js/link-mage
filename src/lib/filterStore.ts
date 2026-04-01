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
      
      // Build match flags
      let matches = true;

      // Filter by type
      if (nodeTypes.length > 0) {
        const matchesType = nodeTypes.includes(node.type);
        matches = invertMode ? matchesType : !matchesType;
      }

      // Filter by degree
      if (degree < minDegree || degree > maxDegree) {
        matches = invertMode;
      }

      // Hide isolated nodes
      if (hideIsolated && degree === 0) {
        matches = invertMode;
      }

      // Filter by search query
      if (searchQuery) {
        const q = searchQuery.toLowerCase();
        const matchesType = node.type.toLowerCase().includes(q);
        const matchesLabel = (node.properties['label'] || '').toLowerCase().includes(q);
        const matchesValue = (node.properties['value'] || '').toLowerCase().includes(q);
        const matchesAnyProp = Object.values(node.properties).some(v => 
          v.toLowerCase().includes(q)
        );
        const matchesSearch = matchesType || matchesLabel || matchesValue || matchesAnyProp;
        matches = invertMode ? matchesSearch : matchesSearch;
      }

      // Filter by provenance source
      if (provenanceSources.length > 0 && node.provenance) {
        const matchesSource = provenanceSources.includes(node.provenance.source);
        matches = matches && (invertMode ? matchesSource : !matchesSource);
      }

      // Filter by confidence
      if (minConfidence > 0 && node.provenance) {
        const matchesConfidence = node.provenance.confidence * 100 >= minConfidence;
        matches = matches && (invertMode ? matchesConfidence : !matchesConfidence);
      }

      // Filter by date range
      if ((dateFrom || dateTo) && node.provenance) {
        const nodeDate = new Date(node.provenance.createdAt).getTime();
        const from = dateFrom ? new Date(dateFrom).getTime() : 0;
        const to = dateTo ? new Date(dateTo).getTime() : Date.now();
        const matchesDate = nodeDate >= from && nodeDate <= to;
        matches = matches && (invertMode ? matchesDate : !matchesDate);
      }

      return matches;
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
