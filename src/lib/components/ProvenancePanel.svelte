<script lang="ts">
  import type { Provenance } from '$lib/types/provenance';
  import type { Node } from '$lib/stores';

  interface Props {
    node: Node | null;
  }

  let { node }: Props = $props();

  function formatDate(isoString: string): string {
    try {
      const date = new Date(isoString);
      return date.toLocaleString();
    } catch {
      return isoString;
    }
  }

  function formatConfidence(confidence: number): string {
    return `${(confidence * 100).toFixed(0)}%`;
  }

  function getSourceColor(source: string): string {
    const colors: Record<string, string> = {
      manual: 'var(--accent)',
      dns_lookup: 'var(--cyto-ip-bg)',
      whois_lookup: 'var(--cyto-domain-bg)',
      geoip_lookup: 'var(--cyto-location-bg)',
      web_scrape: 'var(--cyto-url-bg)',
      virustotal: 'var(--cyto-vt-bg)',
      remote_transform: 'var(--cyto-hash-bg)',
      plugin: 'var(--cyto-org-bg)',
      import: 'var(--cyto-collection-bg)',
      pipeline: 'var(--cyto-person-bg)',
      unknown: 'var(--text-muted)',
    };
    return colors[source] || 'var(--text-muted)';
  }

  function getSourceIcon(source: string): string {
    const icons: Record<string, string> = {
      manual: '✏️',
      dns_lookup: '🔍',
      whois_lookup: '📋',
      geoip_lookup: '🌍',
      web_scrape: '🕸️',
      virustotal: '🛡️',
      remote_transform: '🌐',
      plugin: '🔌',
      import: '📥',
      pipeline: '⚙️',
      unknown: '❓',
    };
    return icons[source] || '❓';
  }
</script>

{#if node?.provenance}
  <div class="provenance-panel">
    <h3 class="provenance-title">Provenance</h3>

    <div class="provenance-section">
      <div class="provenance-row">
        <span class="provenance-label">Source</span>
        <span 
          class="provenance-value provenance-source"
          style="border-left-color: {getSourceColor(node.provenance.source)}"
        >
          {getSourceIcon(node.provenance.source)} {node.provenance.source}
        </span>
      </div>

      <div class="provenance-row">
        <span class="provenance-label">Created</span>
        <span class="provenance-value">{formatDate(node.provenance.createdAt)}</span>
      </div>

      <div class="provenance-row">
        <span class="provenance-label">By</span>
        <span class="provenance-value">{node.provenance.createdBy || 'Unknown'}</span>
      </div>

      <div class="provenance-row">
        <span class="provenance-label">Confidence</span>
        <div class="confidence-bar">
          <div 
            class="confidence-fill" 
            style="width: {node.provenance.confidence * 100}%"
            class:high={node.provenance.confidence >= 0.8}
            class:medium={node.provenance.confidence >= 0.5 && node.provenance.confidence < 0.8}
            class:low={node.provenance.confidence < 0.5}
          ></div>
          <span class="confidence-text">{formatConfidence(node.provenance.confidence)}</span>
        </div>
      </div>

      {#if node.provenance.externalRef}
        <div class="provenance-row">
          <span class="provenance-label">External Ref</span>
          <span class="provenance-value provenance-ref">{node.provenance.externalRef}</span>
        </div>
      {/if}
    </div>

    {#if node.provenance.transformChain && node.provenance.transformChain.length > 0}
      <div class="provenance-section">
        <h4 class="provenance-subtitle">Transform Chain ({node.provenance.transformChain.length})</h4>
        <ol class="transform-chain">
          {#each node.provenance.transformChain as step, i}
            <li class="transform-step">
              <div class="step-header">
                <span class="step-number">{i + 1}</span>
                <span class="step-status" class:success={step.success} class:error={!step.success}>
                  {step.success ? '✓' : '✗'}
                </span>
                <span class="step-name">{step.transformName}</span>
              </div>
              <div class="step-details">
                <span class="step-time">{formatDate(step.executedAt)}</span>
                {#if step.durationMs}
                  <span class="step-duration">{step.durationMs}ms</span>
                {/if}
              </div>
              {#if step.error}
                <div class="step-error">{step.error}</div>
              {/if}
            </li>
          {/each}
        </ol>
      </div>
    {/if}

    {#if node.provenance.notes}
      <div class="provenance-section">
        <h4 class="provenance-subtitle">Notes</h4>
        <p class="provenance-notes">{node.provenance.notes}</p>
      </div>
    {/if}
  </div>
{:else}
  <div class="provenance-panel provenance-empty">
    <p class="no-provenance">
      No provenance information available.<br/>
      <small>This node may have been created before provenance tracking was implemented.</small>
    </p>
  </div>
{/if}

<style>
  .provenance-panel {
    padding: 12px;
    background: var(--bg-base);
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
  }

  .provenance-empty {
    background: var(--bg-elevated);
    border-style: dashed;
  }

  .provenance-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 12px 0;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border);
  }

  .provenance-subtitle {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    margin: 12px 0 8px 0;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .provenance-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .provenance-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }

  .provenance-label {
    font-size: 11px;
    color: var(--text-muted);
    font-weight: 500;
  }

  .provenance-value {
    font-size: 12px;
    color: var(--text-primary);
    text-align: right;
    max-width: 60%;
  }

  .provenance-source {
    padding-left: 8px;
    border-left: 3px solid;
    font-weight: 500;
  }

  .provenance-ref {
    font-family: var(--font-mono);
    font-size: 10px;
    word-break: break-all;
  }

  .confidence-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    max-width: 150px;
    margin-left: auto;
  }

  .confidence-fill {
    height: 6px;
    border-radius: 3px;
    transition: width 0.3s ease;
  }

  .confidence-fill.high {
    background: var(--success);
  }

  .confidence-fill.medium {
    background: var(--warning);
  }

  .confidence-fill.low {
    background: var(--error);
  }

  .confidence-text {
    font-size: 10px;
    font-weight: 600;
    color: var(--text-secondary);
    min-width: 35px;
    text-align: right;
  }

  .transform-chain {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .transform-step {
    padding: 8px;
    background: var(--bg-elevated);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
  }

  .step-header {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 4px;
  }

  .step-number {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--accent);
    color: white;
    font-size: 10px;
    font-weight: 600;
  }

  .step-status {
    font-size: 12px;
  }

  .step-status.success {
    color: var(--success);
  }

  .step-status.error {
    color: var(--error);
  }

  .step-name {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .step-details {
    display: flex;
    gap: 12px;
    font-size: 10px;
    color: var(--text-muted);
  }

  .step-duration {
    font-family: var(--font-mono);
  }

  .step-error {
    margin-top: 6px;
    padding: 6px;
    background: var(--error-bg);
    border-radius: var(--radius-sm);
    font-size: 10px;
    color: var(--error);
    font-family: var(--font-mono);
  }

  .provenance-notes {
    font-size: 11px;
    color: var(--text-secondary);
    line-height: 1.5;
    margin: 0;
    padding: 8px;
    background: var(--bg-elevated);
    border-radius: var(--radius-sm);
  }

  .no-provenance {
    font-size: 12px;
    color: var(--text-muted);
    text-align: center;
    padding: 20px;
  }

  .no-provenance small {
    display: block;
    margin-top: 8px;
    font-size: 10px;
    opacity: 0.7;
  }
</style>
