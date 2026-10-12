<script lang="ts">
  import { filterCriteria, FILTER_PRESETS, resetFilters } from '$lib/filterStore';
  import { graphStore } from '$lib/stores';
  import { NODE_TYPES } from '$lib/nodeTypes';
  import { onDestroy } from 'svelte';

  // Debounce timer for search input
  let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  const SEARCH_DEBOUNCE_MS = 300;

  function toggleType(type: string) {
    filterCriteria.update(c => {
      if (c.nodeTypes.includes(type)) {
        return { ...c, nodeTypes: c.nodeTypes.filter(t => t !== type) };
      } else {
        return { ...c, nodeTypes: [...c.nodeTypes, type] };
      }
    });
  }

  function clearTypes() {
    filterCriteria.update(c => ({ ...c, nodeTypes: [] }));
  }

  function updateMinDegree(e: Event) {
    const val = parseInt((e.target as HTMLInputElement).value);
    filterCriteria.update(c => ({ ...c, minDegree: isNaN(val) ? 0 : val }));
  }

  function updateMaxDegree(e: Event) {
    const val = parseInt((e.target as HTMLInputElement).value);
    filterCriteria.update(c => ({ ...c, maxDegree: isNaN(val) ? 999 : val }));
  }

  function updateSearch(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    // Debounce search updates to avoid excessive re-renders on large graphs
    if (searchDebounceTimer !== null) {
      clearTimeout(searchDebounceTimer);
    }
    searchDebounceTimer = setTimeout(() => {
      filterCriteria.update(c => ({ ...c, searchQuery: val }));
      searchDebounceTimer = null;
    }, SEARCH_DEBOUNCE_MS);
  }

  function toggleHideIsolated() {
    filterCriteria.update(c => ({ ...c, hideIsolated: !c.hideIsolated }));
  }

  function applyPreset(preset: keyof typeof FILTER_PRESETS) {
    const settings = FILTER_PRESETS[preset];
    filterCriteria.update(c => ({ ...c, ...settings }));
  }

  // Cleanup debounce timer on destroy
  onDestroy(() => {
    if (searchDebounceTimer !== null) {
      clearTimeout(searchDebounceTimer);
    }
  });

  // Provenance sources for filtering
  const provenanceSources = [
    'manual',
    'dns_lookup',
    'whois_lookup',
    'geoip_lookup',
    'web_scrape',
    'virustotal',
    'remote_transform',
    'plugin',
    'import',
    'pipeline',
    'unknown',
  ];

  function toggleProvenanceSource(source: string) {
    filterCriteria.update(c => {
      if (c.provenanceSources.includes(source)) {
        return { ...c, provenanceSources: c.provenanceSources.filter(s => s !== source) };
      } else {
        return { ...c, provenanceSources: [...c.provenanceSources, source] };
      }
    });
  }
</script>

<div class="filter-panel">
  <!-- Quick presets -->
  <div class="filter-section">
    <label class="filter-label">Quick Filters</label>
    <div class="preset-buttons">
      <button class="preset-btn" onclick={() => applyPreset('highConfidence')}>
        🎯 High Confidence
      </button>
      <button class="preset-btn" onclick={() => applyPreset('hubs')}>
        ⭐ Hub Nodes
      </button>
      <button class="preset-btn" onclick={() => applyPreset('isolated')}>
        👁️ Hide Isolated
      </button>
      <button class="preset-btn" onclick={() => applyPreset('recent')}>
        🕐 Last 7 Days
      </button>
    </div>
  </div>

  <div class="filter-group">
    <label class="filter-label" for="search-graph">Search Graph</label>
    <input
      id="search-graph"
      type="text"
      class="form-input"
      placeholder="Search properties, labels, types..."
      value={$filterCriteria.searchQuery}
      oninput={updateSearch}
    />
  </div>

  <div class="filter-group">
    <label class="filter-label">Degree Range (Connections)</label>
    <div class="degree-range-inputs">
      <div class="degree-input">
        <span class="degree-label">Min:</span>
        <input
          type="range"
          min="0" max="20"
          value={$filterCriteria.minDegree}
          oninput={updateMinDegree}
        />
        <span class="degree-val">{$filterCriteria.minDegree}+</span>
      </div>
      <div class="degree-input">
        <span class="degree-label">Max:</span>
        <input
          type="range"
          min="0" max="50"
          value={$filterCriteria.maxDegree}
          oninput={updateMaxDegree}
        />
        <span class="degree-val">{$filterCriteria.maxDegree}</span>
      </div>
    </div>
  </div>

  <div class="filter-group">
    <div class="filter-label-row">
      <span class="filter-label">Entity Types</span>
      <button class="text-link" onclick={clearTypes}>Reset</button>
    </div>
    <div class="type-grid">
      {#each NODE_TYPES as type}
        <button
          class="type-chip"
          class:active={$filterCriteria.nodeTypes.includes(type)}
          onclick={() => toggleType(type)}
        >
          {type}
        </button>
      {/each}
    </div>
  </div>

  <div class="filter-group">
    <label class="filter-label">Provenance Source</label>
    <div class="type-grid">
      {#each provenanceSources as source}
        <button
          class="type-chip"
          class:active={$filterCriteria.provenanceSources.includes(source)}
          onclick={() => toggleProvenanceSource(source)}
        >
          {source.replace('_', ' ')}
        </button>
      {/each}
    </div>
  </div>

  <div class="filter-group">
    <label class="filter-label">Advanced Options</label>
    <div class="advanced-options">
      <label class="checkbox-label">
        <input 
          type="checkbox" 
          checked={$filterCriteria.hideIsolated}
          onchange={toggleHideIsolated}
        />
        <span>Hide isolated nodes (no connections)</span>
      </label>
    </div>
  </div>

  <div class="filter-actions">
    <button class="btn btn-secondary" onclick={resetFilters}>
      Reset All Filters
    </button>
    {#if $filterCriteria.nodeTypes.length > 0 || $filterCriteria.searchQuery || $filterCriteria.minDegree > 0}
      <span class="filter-count">
        {$filterCriteria.nodeTypes.length} type(s), 
        degree {$filterCriteria.minDegree}}-{$filterCriteria.maxDegree}
      </span>
    {/if}
  </div>
</div>

<style>
  .filter-panel {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 4px 0;
  }

  .filter-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .filter-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .filter-label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-muted);
    letter-spacing: 0.05em;
  }

  .filter-label-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .text-link {
    background: none;
    border: none;
    color: var(--accent);
    font-size: 10px;
    cursor: pointer;
    padding: 0;
  }

  .form-input {
    width: 100%;
    padding: 6px 8px;
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-primary);
    font-size: 12px;
  }

  /* ── Preset buttons ───────────────────────────────────────────────────── */
  .preset-buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .preset-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 999px;
    font-size: 11px;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s;
  }

  .preset-btn:hover {
    background: var(--bg-hover);
    border-color: var(--accent);
    color: var(--text-primary);
  }

  /* ── Degree range inputs ─────────────────────────────────────────────── */
  .degree-range-inputs {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .degree-input {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .degree-label {
    font-size: 11px;
    color: var(--text-muted);
    min-width: 32px;
  }

  .degree-input input[type="range"] {
    flex: 1;
  }

  .degree-val {
    font-size: 11px;
    font-family: var(--font-mono);
    min-width: 32px;
    text-align: right;
    color: var(--text-primary);
  }

  /* ── Type grid ───────────────────────────────────────────────────────── */
  .type-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .type-chip {
    padding: 4px 10px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 999px;
    font-size: 11px;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.1s;
  }

  .type-chip:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .type-chip.active {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }

  /* ── Advanced options ────────────────────────────────────────────────── */
  .advanced-options {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 8px;
    background: var(--bg-elevated);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .checkbox-label input[type="checkbox"] {
    width: 16px;
    height: 16px;
    cursor: pointer;
  }

  /* ── Filter actions ──────────────────────────────────────────────────── */
  .filter-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: 8px;
    border-top: 1px solid var(--border);
  }

  .btn {
    padding: 6px 12px;
    border-radius: var(--radius-md);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-secondary {
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    color: var(--text-secondary);
  }

  .btn-secondary:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .filter-count {
    font-size: 11px;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }
</style>
