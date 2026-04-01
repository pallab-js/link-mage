/**
 * Centralized node type definitions.
 * 
 * This is the single source of truth for all node types in the application.
 * The backend entity registry in src-tauri/src/entities.rs should match these.
 */

/**
 * List of all supported node types.
 * Keep this in sync with:
 * - src-tauri/src/entities.rs (EntityRegistry::new)
 * - src/lib/components/Sidebar.svelte (NODE_TYPES)
 * - src/lib/components/FilterPanel.svelte (NODE_TYPES)
 */
export const NODE_TYPES = [
  'IP',
  'IPv6',
  'Domain',
  'Email',
  'Person',
  'URL',
  'Organisation',
  'Hash',
  'Location',
  'VirusTotalReport',
  'Phone',
  'CryptocurrencyAddress',
  'SocialProfile',
  'Image',
  'Collection',
] as const;

/** Type-safe node type (derived from NODE_TYPES array) */
export type NodeType = typeof NODE_TYPES[number];

/**
 * Check if a string is a valid node type
 */
export function isValidNodeType(type: string): type is NodeType {
  return NODE_TYPES.includes(type as NodeType);
}
