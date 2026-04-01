/**
 * Graph Statistics & Analytics
 * 
 * Provides insights into graph structure and composition.
 */

import type { Node, Edge, Graph } from '../stores';

/**
 * Basic graph statistics
 */
export interface GraphStats {
  totalNodes: number;
  totalEdges: number;
  nodesByType: Record<string, number>;
  edgesByLabel: Record<string, number>;
  averageDegree: number;
  maxDegree: number;
  minDegree: number;
  isolatedNodes: number;
  hubNodes: number; // nodes with degree >= 5
  density: number;
  connectedComponents: number;
}

/**
 * Calculate comprehensive graph statistics
 */
export function calculateStats(graph: Graph): GraphStats {
  const { nodes, edges } = graph;
  
  // Nodes by type
  const nodesByType: Record<string, number> = {};
  for (const node of nodes) {
    nodesByType[node.type] = (nodesByType[node.type] || 0) + 1;
  }

  // Edges by label
  const edgesByLabel: Record<string, number> = {};
  for (const edge of edges) {
    edgesByLabel[edge.label] = (edgesByLabel[edge.label] || 0) + 1;
  }

  // Degree calculations
  const degrees = new Map<string, number>();
  for (const edge of edges) {
    degrees.set(edge.source, (degrees.get(edge.source) || 0) + 1);
    degrees.set(edge.target, (degrees.get(edge.target) || 0) + 1);
  }

  let totalDegree = 0;
  let maxDegree = 0;
  let minDegree = Infinity;
  let isolatedNodes = 0;
  let hubNodes = 0;

  for (const node of nodes) {
    const degree = degrees.get(node.id) || 0;
    totalDegree += degree;
    maxDegree = Math.max(maxDegree, degree);
    minDegree = Math.min(minDegree, degree);
    if (degree === 0) isolatedNodes++;
    if (degree >= 5) hubNodes++;
  }

  if (nodes.length === 0) {
    minDegree = 0;
  }

  const averageDegree = nodes.length > 0 ? totalDegree / nodes.length : 0;

  // Graph density: 2E / (V * (V-1)) for undirected graphs
  const density = nodes.length > 1 
    ? (2 * edges.length) / (nodes.length * (nodes.length - 1))
    : 0;

  // Connected components (simplified - uses BFS)
  const connectedComponents = countConnectedComponents(nodes, edges);

  return {
    totalNodes: nodes.length,
    totalEdges: edges.length,
    nodesByType,
    edgesByLabel,
    averageDegree: Math.round(averageDegree * 100) / 100,
    maxDegree,
    minDegree,
    isolatedNodes,
    hubNodes,
    density: Math.round(density * 10000) / 10000,
    connectedComponents,
  };
}

/**
 * Count connected components using BFS
 */
function countConnectedComponents(nodes: Node[], edges: Edge[]): number {
  if (nodes.length === 0) return 0;

  // Build adjacency list
  const adjacency = new Map<string, string[]>();
  for (const node of nodes) {
    adjacency.set(node.id, []);
  }
  for (const edge of edges) {
    const sourceAdj = adjacency.get(edge.source) || [];
    const targetAdj = adjacency.get(edge.target) || [];
    sourceAdj.push(edge.target);
    targetAdj.push(edge.source);
    adjacency.set(edge.source, sourceAdj);
    adjacency.set(edge.target, targetAdj);
  }

  const visited = new Set<string>();
  let components = 0;

  for (const node of nodes) {
    if (!visited.has(node.id)) {
      components++;
      // BFS
      const queue = [node.id];
      while (queue.length > 0) {
        const current = queue.shift()!;
        if (!visited.has(current)) {
          visited.add(current);
          const neighbors = adjacency.get(current) || [];
          for (const neighbor of neighbors) {
            if (!visited.has(neighbor)) {
              queue.push(neighbor);
            }
          }
        }
      }
    }
  }

  return components;
}

/**
 * Get top N nodes by degree (hubs)
 */
export function getTopHubs(graph: Graph, limit = 10): Array<{ node: Node; degree: number }> {
  const degrees = new Map<string, number>();
  for (const edge of graph.edges) {
    degrees.set(edge.source, (degrees.get(edge.source) || 0) + 1);
    degrees.set(edge.target, (degrees.get(edge.target) || 0) + 1);
  }

  const nodesWithDegree = graph.nodes.map(node => ({
    node,
    degree: degrees.get(node.id) || 0,
  }));

  return nodesWithDegree
    .sort((a, b) => b.degree - a.degree)
    .slice(0, limit);
}

/**
 * Get nodes with no connections (isolated)
 */
export function getIsolatedNodes(graph: Graph): Node[] {
  const connectedIds = new Set<string>();
  for (const edge of graph.edges) {
    connectedIds.add(edge.source);
    connectedIds.add(edge.target);
  }

  return graph.nodes.filter(node => !connectedIds.has(node.id));
}

/**
 * Calculate centrality metrics (simplified degree centrality)
 */
export function calculateCentrality(graph: Graph): Map<string, number> {
  const centrality = new Map<string, number>();
  const n = graph.nodes.length;
  
  if (n <= 1) {
    for (const node of graph.nodes) {
      centrality.set(node.id, 0);
    }
    return centrality;
  }

  const degrees = new Map<string, number>();
  for (const edge of graph.edges) {
    degrees.set(edge.source, (degrees.get(edge.source) || 0) + 1);
    degrees.set(edge.target, (degrees.get(edge.target) || 0) + 1);
  }

  // Normalized degree centrality: degree / (n-1)
  for (const node of graph.nodes) {
    const degree = degrees.get(node.id) || 0;
    centrality.set(node.id, degree / (n - 1));
  }

  return centrality;
}

/**
 * Format statistics for display
 */
export function formatStats(stats: GraphStats): string {
  const lines: string[] = [];
  
  lines.push(`📊 Graph Statistics`);
  lines.push(``);
  lines.push(`Nodes: ${stats.totalNodes}`);
  lines.push(`Edges: ${stats.totalEdges}`);
  lines.push(`Density: ${(stats.density * 100).toFixed(2)}%`);
  lines.push(``);
  lines.push(`Degree Statistics:`);
  lines.push(`  Average: ${stats.averageDegree}`);
  lines.push(`  Max: ${stats.maxDegree}`);
  lines.push(`  Min: ${stats.minDegree}`);
  lines.push(``);
  lines.push(`Node Types:`);
  for (const [type, count] of Object.entries(stats.nodesByType)) {
    lines.push(`  ${type}: ${count}`);
  }
  lines.push(``);
  lines.push(`Special Nodes:`);
  lines.push(`  Isolated: ${stats.isolatedNodes}`);
  lines.push(`  Hubs (5+): ${stats.hubNodes}`);
  lines.push(`  Components: ${stats.connectedComponents}`);
  
  return lines.join('\n');
}
