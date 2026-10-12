/**
 * Annotation & Tagging System
 * 
 * Allows users to add notes, labels, and classifications to graph entities.
 * Inspired by Maltego's annotation features and professional investigation tools.
 */

import { writable, get } from 'svelte/store';
import type { Writable } from 'svelte/store';

/**
 * Annotation attached to a node or edge
 */
export interface Annotation {
  /** Unique identifier */
  id: string;
  /** ID of the entity this annotation is attached to */
  entityId: string;
  /** Type of entity (node or edge) */
  entityType: 'node' | 'edge';
  /** Annotation text content */
  text: string;
  /** Color for visual indication */
  color: AnnotationColor;
  /** Creation timestamp (ISO 8601) */
  createdAt: string;
  /** Last modified timestamp (ISO 8601) */
  updatedAt: string;
  /** User who created the annotation */
  createdBy: string;
  /** Optional pinned state */
  pinned: boolean;
}

/**
 * Predefined annotation colors
 */
export type AnnotationColor = 
  | 'yellow'
  | 'green'
  | 'blue'
  | 'purple'
  | 'red'
  | 'orange';

export const ANNOTATION_COLORS: Record<AnnotationColor, string> = {
  yellow: '#fbbf24',
  green: '#22c55e',
  blue: '#3b82f6',
  purple: '#a855f7',
  red: '#ef4444',
  orange: '#f97316',
};

/**
 * Tag for classifying and filtering entities
 */
export interface Tag {
  /** Unique identifier */
  id: string;
  /** Tag name/label */
  name: string;
  /** Tag color */
  color: string;
  /** Creation timestamp (ISO 8601) */
  createdAt: string;
  /** Number of entities using this tag */
  usageCount: number;
}

/**
 * Entity tag assignment
 */
export interface EntityTag {
  /** Unique identifier */
  id: string;
  /** ID of the tagged entity */
  entityId: string;
  /** Type of entity */
  entityType: 'node' | 'edge';
  /** Tag ID */
  tagId: string;
  /** Timestamp when tag was applied */
  appliedAt: string;
  /** User who applied the tag */
  appliedBy: string;
}

// ── Annotation Store ──────────────────────────────────────────────────────────

function createAnnotationStore() {
  const { subscribe, set, update }: Writable<Annotation[]> = writable([]);

  return {
    subscribe,

    /** Add a new annotation */
    add(annotation: Omit<Annotation, 'id' | 'createdAt' | 'updatedAt'>): Annotation {
      const now = new Date().toISOString();
      const newAnnotation: Annotation = {
        ...annotation,
        id: generateId(),
        createdAt: now,
        updatedAt: now,
      };
      update(anns => [...anns, newAnnotation]);
      return newAnnotation;
    },

    /** Update an existing annotation */
    update(id: string, updates: Partial<Annotation>): boolean {
      let updated = false;
      update(anns => 
        anns.map(a => {
          if (a.id === id) {
            updated = true;
            return { ...a, ...updates, updatedAt: new Date().toISOString() };
          }
          return a;
        })
      );
      return updated;
    },

    /** Remove an annotation */
    remove(id: string): boolean {
      let removed = false;
      update(anns => {
        const filtered = anns.filter(a => {
          if (a.id === id) {
            removed = true;
            return false;
          }
          return true;
        });
        return filtered;
      });
      return removed;
    },

    /** Get all annotations for an entity */
    getByEntity(entityId: string): Annotation[] {
      const anns = get({ subscribe });
      return anns.filter(a => a.entityId === entityId);
    },
  };
}

export const annotationStore = createAnnotationStore();

// ── Tag Store ─────────────────────────────────────────────────────────────────

function createTagStore() {
  const { subscribe, set, update }: Writable<Tag[]> = writable([]);
  const entityTagsStore: Writable<EntityTag[]> = writable([]);

  return {
    subscribe,

    /** Get entity tags store for subscription */
    get entityTags() {
      return entityTagsStore;
    },

    /** Add a new tag */
    add(tag: Omit<Tag, 'id' | 'createdAt' | 'usageCount'>): Tag {
      const newTag: Tag = {
        ...tag,
        id: generateId(),
        createdAt: new Date().toISOString(),
        usageCount: 0,
      };
      update(tags => [...tags, newTag]);
      return newTag;
    },

    /** Update an existing tag */
    update(id: string, updates: Partial<Tag>): boolean {
      let updated = false;
      update(tags => 
        tags.map(t => {
          if (t.id === id) {
            updated = true;
            return { ...t, ...updates };
          }
          return t;
        })
      );
      return updated;
    },

    /** Remove a tag (and all assignments) */
    remove(id: string): boolean {
      let removed = false;
      update(tags => {
        const filtered = tags.filter(t => {
          if (t.id === id) {
            removed = true;
            return false;
          }
          return true;
        });
        return filtered;
      });
      // Remove all assignments
      entityTagsStore.update(et => et.filter(e => e.tagId !== id));
      return removed;
    },

    /** Apply a tag to an entity */
    applyToEntity(tagId: string, entityId: string, entityType: 'node' | 'edge', appliedBy = 'user'): EntityTag | null {
      const tags = get({ subscribe });
      const tag = tags.find(t => t.id === tagId);
      if (!tag) return null;

      // Check if already applied
      const existing = get(entityTagsStore).find(
        et => et.entityId === entityId && et.tagId === tagId
      );
      if (existing) return existing;

      const entityTag: EntityTag = {
        id: generateId(),
        entityId,
        entityType,
        tagId,
        appliedAt: new Date().toISOString(),
        appliedBy,
      };

      entityTagsStore.update(et => [...et, entityTag]);
      
      // Update usage count
      update(tags => 
        tags.map(t => 
          t.id === tagId ? { ...t, usageCount: t.usageCount + 1 } : t
        )
      );

      return entityTag;
    },

    /** Remove a tag from an entity */
    removeFromEntity(tagId: string, entityId: string): boolean {
      let removed = false;
      entityTagsStore.update(et => {
        const filtered = et.filter(e => {
          if (e.entityId === entityId && e.tagId === tagId) {
            removed = true;
            return false;
          }
          return true;
        });
        return filtered;
      });

      if (removed) {
        // Decrement usage count
        update(tags => 
          tags.map(t => 
            t.id === tagId ? { ...t, usageCount: Math.max(0, t.usageCount - 1) } : t
          )
        );
      }

      return removed;
    },

    /** Get all tags for an entity */
    getForEntity(entityId: string): Tag[] {
      const tags = get({ subscribe });
      const entityTags = get(entityTagsStore);
      const tagIds = new Set(
        entityTags.filter(et => et.entityId === entityId).map(et => et.tagId)
      );
      return tags.filter(t => tagIds.has(t.id));
    },
  };
}

export const tagStore = createTagStore();

// ── Utilities ─────────────────────────────────────────────────────────────────

function generateId(): string {
  return `ann_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

/** Default colors for new tags */
export const DEFAULT_TAG_COLORS = [
  '#ef4444', // red
  '#f97316', // orange
  '#fbbf24', // yellow
  '#22c55e', // green
  '#14b8a6', // teal
  '#3b82f6', // blue
  '#6366f1', // indigo
  '#a855f7', // purple
  '#ec4899', // pink
  '#6b7280', // gray
];
