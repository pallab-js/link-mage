import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { filterCriteria, filteredGraph, resetFilters, type FilterCriteria } from '../filterStore';
import { graphStore } from '../stores';
import type { Node, Edge } from '../stores';

/**
 * Regression tests for the filter predicate. The original implementation
 * OR-ed the criteria and inverted the semantics of invertMode — the tests
 * below pin the corrected behaviour:
 *  - every active criterion is AND-ed;
 *  - invertMode shows the *complement* (nodes failing at least one criterion);
 *  - nodes without provenance pass provenance-source filters (treated as
 *    'unknown' rather than silently dropped).
 */

const A: Node = { id: 'a', type: 'IP', properties: { value: '10.0.0.1' } };
const B: Node = { id: 'b', type: 'Domain', properties: { label: 'evil corp' } };
const C: Node = {
  id: 'c',
  type: 'Email',
  properties: {},
  provenance: {
    source: 'manual',
    createdAt: '2026-01-01T00:00:00Z',
    createdBy: 'user',
    transformChain: [],
    confidence: 0.5,
  },
};
const E: Edge = { id: 'e1', source: 'a', target: 'b', label: 'resolves', properties: {} };

function setCriteria(patch: Partial<FilterCriteria>) {
  filterCriteria.update(c => ({ ...c, ...patch }));
}

function visibleIds(): string[] {
  return get(filteredGraph).nodes.map(n => n.id);
}

beforeEach(() => {
  resetFilters();
  graphStore.setGraph({ nodes: [A, B, C], edges: [E] });
});

describe('filterStore', () => {
  it('nodeTypes filter keeps only listed types', () => {
    setCriteria({ nodeTypes: ['IP'] });
    expect(visibleIds()).toEqual(['a']);
  });

  it('criteria are AND-ed, not OR-ed', () => {
    // 'a' passes the type filter but fails the search; 'b' the reverse.
    // The old OR-implementation returned both.
    setCriteria({ nodeTypes: ['IP'], searchQuery: 'evil corp' });
    expect(visibleIds()).toEqual([]);
  });

  it('invertMode shows the complement of the filtered set', () => {
    setCriteria({ nodeTypes: ['IP'], invertMode: true });
    expect(visibleIds()).toEqual(['b', 'c']);
  });

  it('searchQuery matches property values case-insensitively', () => {
    setCriteria({ searchQuery: 'EVIL' });
    expect(visibleIds()).toEqual(['b']);
  });

  it('hideIsolated removes degree-0 nodes', () => {
    setCriteria({ hideIsolated: true });
    expect(visibleIds()).toEqual(['a', 'b']);
  });

  it('minDegree removes nodes below the threshold', () => {
    setCriteria({ minDegree: 1 });
    expect(visibleIds()).toEqual(['a', 'b']);
  });

  it('provenance source filter keeps matching nodes and unknown-source nodes', () => {
    setCriteria({ provenanceSources: ['manual'] });
    expect(visibleIds()).toEqual(['c']);

    setCriteria({ provenanceSources: ['unknown'] });
    // Nodes without provenance pass — they are 'unknown', not invisible.
    expect(visibleIds()).toEqual(['a', 'b']);
  });

  it('minConfidence drops low-confidence nodes but keeps unprovenanced ones', () => {
    setCriteria({ minConfidence: 90 });
    expect(visibleIds()).toEqual(['a', 'b']); // c has confidence 0.5
  });

  it('edges only survive when both endpoints are visible', () => {
    setCriteria({ nodeTypes: ['IP'] });
    expect(get(filteredGraph).edges).toEqual([]);

    setCriteria({ nodeTypes: [] });
    expect(get(filteredGraph).edges).toHaveLength(1);
  });

  it('resetFilters restores the full graph', () => {
    setCriteria({ nodeTypes: ['IP'], hideIsolated: true, invertMode: true });
    resetFilters();
    const g = get(filteredGraph);
    expect(g.nodes).toHaveLength(3);
    expect(g.edges).toHaveLength(1);
    expect(get(filterCriteria).nodeTypes).toEqual([]);
    expect(get(filterCriteria).invertMode).toBe(false);
  });
});
