/**
 * Data Provenance Tracking
 * 
 * Tracks the origin and transformation history of each entity.
 * Inspired by forensic integrity controls in professional OSINT tools.
 */

/**
 * Represents a single step in the transform chain
 */
export interface TransformStep {
  /** Name of the transform that was applied */
  transformName: string;
  /** Timestamp when transform was executed (ISO 8601) */
  executedAt: string;
  /** ID of the source node that was transformed */
  sourceNodeId: string;
  /** Transform execution duration in milliseconds */
  durationMs?: number;
  /** Whether transform succeeded */
  success: boolean;
  /** Error message if transform failed */
  error?: string;
}

/**
 * Source types for entity creation
 */
export type ProvenanceSource = 
  | 'manual'           // User manually created the entity
  | 'dns_lookup'       // Created via DNS lookup transform
  | 'whois_lookup'     // Created via WHOIS lookup transform
  | 'geoip_lookup'     // Created via GeoIP transform
  | 'web_scrape'       // Created via web scraping
  | 'virustotal'       // Created via VirusTotal lookup
  | 'remote_transform' // Created via remote transform
  | 'plugin'          // Created via native plugin
  | 'import'          // Imported from file
  | 'pipeline'        // Created via transform pipeline
  | 'unknown';        // Source could not be determined

/**
 * Provenance metadata for an entity
 */
export interface Provenance {
  /** Original source of this entity */
  source: ProvenanceSource;
  /** Timestamp when entity was first created (ISO 8601) */
  createdAt: string;
  /** User or system that created the entity */
  createdBy: string;
  /** Chain of transforms that led to this entity */
  transformChain: TransformStep[];
  /** Confidence score (0.0 - 1.0) based on source reliability */
  confidence: number;
  /** External reference ID (e.g., VirusTotal hash, WHOIS record ID) */
  externalRef?: string;
  /** Notes about the provenance */
  notes?: string;
}

/**
 * Source reliability weights for confidence calculation
 */
export const SOURCE_RELIABILITY: Record<ProvenanceSource, number> = {
  manual: 1.0,           // User-created, highest confidence
  dns_lookup: 0.95,      // Direct DNS query
  whois_lookup: 0.90,    // Registry data
  geoip_lookup: 0.85,    // Geolocation database
  web_scrape: 0.75,      // Scraped from webpage
  virustotal: 0.95,      // Aggregated security vendor data
  remote_transform: 0.70, // Third-party transform
  plugin: 0.80,          // Native plugin
  import: 0.85,          // Imported from file
  unknown: 0.50,         // Unknown source, lowest confidence
  pipeline: 0.75,        // Depends on pipeline steps
};

/**
 * Create provenance for a manually created entity
 */
export function createManualProvenance(createdBy = 'user'): Provenance {
  return {
    source: 'manual',
    createdAt: new Date().toISOString(),
    createdBy,
    transformChain: [],
    confidence: SOURCE_RELIABILITY.manual,
  };
}

/**
 * Create provenance for an entity created by a transform
 */
export function createTransformProvenance(
  source: ProvenanceSource,
  sourceNodeId: string,
  transformName: string,
  createdBy = 'system',
  durationMs?: number,
  externalRef?: string,
): Provenance {
  const step: TransformStep = {
    transformName,
    executedAt: new Date().toISOString(),
    sourceNodeId,
    durationMs,
    success: true,
  };

  return {
    source,
    createdAt: step.executedAt,
    createdBy,
    transformChain: [step],
    confidence: SOURCE_RELIABILITY[source],
    externalRef,
  };
}

/**
 * Add a transform step to existing provenance
 */
export function addTransformStep(
  provenance: Provenance,
  transformName: string,
  sourceNodeId: string,
  durationMs?: number,
  success = true,
  error?: string,
): Provenance {
  const step: TransformStep = {
    transformName,
    executedAt: new Date().toISOString(),
    sourceNodeId,
    durationMs,
    success,
    ...(error && { error }),
  };

  // Recalculate confidence based on transform chain
  const newConfidence = calculateConfidence(provenance.source, [...provenance.transformChain, step]);

  return {
    ...provenance,
    transformChain: [...provenance.transformChain, step],
    confidence: newConfidence,
  };
}

/**
 * Calculate confidence score based on source and transform chain
 * Confidence decreases with each transform step (decay factor)
 */
export function calculateConfidence(
  source: ProvenanceSource,
  transformChain: TransformStep[],
  decayFactor = 0.98,
): number {
  const baseConfidence = SOURCE_RELIABILITY[source];
  // Apply decay for each transform step
  const chainDecay = Math.pow(decayFactor, transformChain.length);
  // Factor in failed transforms
  const successRate = transformChain.length === 0 
    ? 1.0 
    : transformChain.filter(s => s.success).length / transformChain.length;
  
  return Math.max(0.1, baseConfidence * chainDecay * successRate);
}

/**
 * Format provenance for display
 */
export function formatProvenance(provenance: Provenance): string {
  const parts: string[] = [];
  
  // Source
  parts.push(`Source: ${provenance.source}`);
  
  // Created
  const createdDate = new Date(provenance.createdAt);
  parts.push(`Created: ${createdDate.toLocaleString()}`);
  
  // Creator
  if (provenance.createdBy) {
    parts.push(`By: ${provenance.createdBy}`);
  }
  
  // Confidence
  parts.push(`Confidence: ${(provenance.confidence * 100).toFixed(0)}%`);
  
  // External reference
  if (provenance.externalRef) {
    parts.push(`Ref: ${provenance.externalRef}`);
  }
  
  // Transform chain
  if (provenance.transformChain.length > 0) {
    parts.push(`\nTransform Chain (${provenance.transformChain.length} steps):`);
    provenance.transformChain.forEach((step, i) => {
      const stepDate = new Date(step.executedAt);
      const status = step.success ? '✓' : '✗';
      parts.push(`  ${i + 1}. ${status} ${step.transformName} (${stepDate.toLocaleTimeString()})`);
    });
  }
  
  return parts.join('\n');
}

/**
 * Merge provenance when entities are combined
 */
export function mergeProvenance(...provenances: Provenance[]): Provenance {
  if (provenances.length === 0) {
    return createManualProvenance('system');
  }
  
  if (provenances.length === 1) {
    return provenances[0];
  }
  
  // Use the oldest creation date
  const earliest = provenances.reduce((a, b) => 
    new Date(a.createdAt) < new Date(b.createdAt) ? a : b
  );
  
  // Combine all transform chains
  const allSteps = provenances.flatMap(p => p.transformChain);
  allSteps.sort((a, b) => new Date(a.executedAt).getTime() - new Date(b.executedAt).getTime());
  
  // Average confidence
  const avgConfidence = provenances.reduce((sum, p) => sum + p.confidence, 0) / provenances.length;
  
  return {
    source: earliest.source,
    createdAt: earliest.createdAt,
    createdBy: earliest.createdBy,
    transformChain: allSteps,
    confidence: avgConfidence,
    notes: `Merged from ${provenances.length} sources`,
  };
}
