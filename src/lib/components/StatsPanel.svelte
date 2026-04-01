<script lang="ts">
  import { graphStore, type Node } from '$lib/stores';
  import { calculateStats, getTopHubs, formatStats, type GraphStats } from '$lib/utils/graphStats';
  import { onMount, onDestroy } from 'svelte';

  let stats = $state<GraphStats | null>(null);
  let topHubs = $state<Array<{ node: Node; degree: number }>>([]);

  function updateStats() {
    const graph = graphStore.snapshot();
    stats = calculateStats(graph);
    topHubs = getTopHubs(graph, 5);
  }

  let unsub: (() => void) | null = null;

  onMount(() => {
    updateStats();
    unsub = graphStore.subscribe(() => {
      updateStats();
    });
  });

  onDestroy(() => {
    unsub?.();
  });

  function nodeLabel(n: Node): string {
    return n.properties?.label || n.properties?.value || n.type;
  }

  function copyStats() {
    if (!stats) return;
    navigator.clipboard.writeText(formatStats(stats));
  }
</script>

<div class="stats-panel">
  <div class="stats-header">
    <h3 class="stats-title">📊 Graph Statistics</h3>
    <button class="btn-icon" onclick={copyStats} title="Copy statistics">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
        <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
      </svg>
    </button>
  </div>

  {#if stats}
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-value">{stats.totalNodes}</div>
        <div class="stat-label">Nodes</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{stats.totalEdges}</div>
        <div class="stat-label">Edges</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{(stats.density * 100).toFixed(1)}%</div>
        <div class="stat-label">Density</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{stats.connectedComponents}</div>
        <div class="stat-label">Components</div>
      </div>
    </div>

    <div class="stats-section">
      <h4 class="section-label">Degree Distribution</h4>
      <div class="degree-stats">
        <div class="degree-stat">
          <span class="stat-name">Average:</span>
          <span class="stat-num">{stats.averageDegree}</span>
        </div>
        <div class="degree-stat">
          <span class="stat-name">Max:</span>
          <span class="stat-num">{stats.maxDegree}</span>
        </div>
        <div class="degree-stat">
          <span class="stat-name">Min:</span>
          <span class="stat-num">{stats.minDegree}</span>
        </div>
      </div>
    </div>

    <div class="stats-section">
      <h4 class="section-label">Special Nodes</h4>
      <div class="special-nodes">
        <div class="special-item">
          <span class="special-icon">🔍</span>
          <div class="special-info">
            <div class="special-value">{stats.isolatedNodes}</div>
            <div class="special-label">Isolated</div>
          </div>
        </div>
        <div class="special-item">
          <span class="special-icon">⭐</span>
          <div class="special-info">
            <div class="special-value">{stats.hubNodes}</div>
            <div class="special-label">Hubs (5+)</div>
          </div>
        </div>
      </div>
    </div>

    {#if stats.nodesByType && Object.keys(stats.nodesByType).length > 0}
      <div class="stats-section">
        <h4 class="section-label">Nodes by Type</h4>
        <div class="type-breakdown">
          {#each Object.entries(stats.nodesByType) as [type, count]}
            <div class="type-item">
              <span class="type-name">{type}</span>
              <div class="type-bar">
                <div 
                  class="type-fill" 
                  style="width: {(count / stats!.totalNodes * 100)}%"
                ></div>
              </div>
              <span class="type-count">{count}</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    {#if topHubs.length > 0}
      <div class="stats-section">
        <h4 class="section-label">Top Hubs</h4>
        <div class="hubs-list">
          {#each topHubs as hub}
            <div class="hub-item">
              <span class="hub-degree">{hub.degree}</span>
              <span class="hub-label">{nodeLabel(hub.node)}</span>
              <span class="hub-type">{hub.node.type}</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {:else}
    <div class="empty-state">
      <p>No graph data available</p>
    </div>
  {/if}
</div>

<style>
  .stats-panel {
    padding: 12px;
    background: var(--bg-base);
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
  }

  .stats-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border);
  }

  .stats-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .btn-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-icon:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  /* ── Stats grid ─────────────────────────────────────────────────────── */
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
    margin-bottom: 16px;
  }

  .stat-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 10px;
    background: var(--bg-elevated);
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
  }

  .stat-value {
    font-size: 20px;
    font-weight: 700;
    color: var(--text-primary);
    line-height: 1;
  }

  .stat-label {
    font-size: 10px;
    color: var(--text-muted);
    margin-top: 4px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  /* ── Stats sections ─────────────────────────────────────────────────── */
  .stats-section {
    margin-top: 16px;
    padding-top: 12px;
    border-top: 1px solid var(--border);
  }

  .section-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    margin: 0 0 8px 0;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  /* ── Degree stats ───────────────────────────────────────────────────── */
  .degree-stats {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .degree-stat {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px 8px;
    background: var(--bg-elevated);
    border-radius: var(--radius-sm);
  }

  .stat-name {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .stat-num {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
    font-family: var(--font-mono);
  }

  /* ── Special nodes ──────────────────────────────────────────────────── */
  .special-nodes {
    display: flex;
    gap: 12px;
  }

  .special-item {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    padding: 8px;
    background: var(--bg-elevated);
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
  }

  .special-icon {
    font-size: 20px;
  }

  .special-info {
    display: flex;
    flex-direction: column;
  }

  .special-value {
    font-size: 16px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .special-label {
    font-size: 10px;
    color: var(--text-muted);
  }

  /* ── Type breakdown ─────────────────────────────────────────────────── */
  .type-breakdown {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .type-item {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .type-name {
    font-size: 11px;
    color: var(--text-secondary);
    min-width: 80px;
  }

  .type-bar {
    flex: 1;
    height: 6px;
    background: var(--bg-elevated);
    border-radius: 3px;
    overflow: hidden;
  }

  .type-fill {
    height: 100%;
    background: var(--accent);
    border-radius: 3px;
    transition: width 0.3s ease;
  }

  .type-count {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-primary);
    min-width: 24px;
    text-align: right;
    font-family: var(--font-mono);
  }

  /* ── Hubs list ──────────────────────────────────────────────────────── */
  .hubs-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .hub-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    background: var(--bg-elevated);
    border-radius: var(--radius-sm);
  }

  .hub-degree {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: var(--accent);
    color: white;
    border-radius: 50%;
    font-size: 10px;
    font-weight: 700;
  }

  .hub-label {
    flex: 1;
    font-size: 11px;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .hub-type {
    font-size: 10px;
    color: var(--text-muted);
    padding: 2px 6px;
    background: var(--bg-base);
    border-radius: 999px;
  }

  /* ── Empty state ────────────────────────────────────────────────────── */
  .empty-state {
    text-align: center;
    padding: 20px;
    color: var(--text-muted);
    font-size: 12px;
  }
</style>
