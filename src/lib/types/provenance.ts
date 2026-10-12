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
