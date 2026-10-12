<script lang="ts">
  import { tagStore, annotationStore, DEFAULT_TAG_COLORS, ANNOTATION_COLORS, type Tag, type Annotation, type AnnotationColor } from '$lib/stores/annotations';
  import { selectedNode, selectedEdge } from '$lib/stores';
  import { confirm } from '@tauri-apps/plugin-dialog';

  let newTagName = $state('');
  let selectedColor = $state(DEFAULT_TAG_COLORS[0]);
  let editingTag: Tag | null = $state(null);
  let editName = $state('');
  let editColor = $state('');

  let tags = $state<Tag[]>([]);
  let entityTags = $state<Tag[]>([]);
  let annotations = $state<Annotation[]>([]);

  // One effect tracks the selection *and* the stores. Reading $selectedNode
  // inside a plain .subscribe() callback was not reactive: switching
  // selection kept showing (and deleting!) the previous entity's data.
  $effect(() => {
    const node = $selectedNode;
    const edge = $selectedEdge;
    const entityId = node?.id || edge?.id || null;

    const unsubs = [
      tagStore.subscribe(t => (tags = t)),
      tagStore.entityTags.subscribe(() => {
        entityTags = entityId ? tagStore.getForEntity(entityId) : [];
      }),
      annotationStore.subscribe(() => {
        annotations = entityId ? annotationStore.getByEntity(entityId) : [];
      }),
    ];
    return () => unsubs.forEach(u => u());
  });

  function handleCreateTag() {
    if (!newTagName.trim()) return;
    
    tagStore.add({
      name: newTagName.trim(),
      color: selectedColor,
    });
    
    newTagName = '';
    selectedColor = DEFAULT_TAG_COLORS[0];
  }

  function handleStartEdit(tag: Tag) {
    editingTag = tag;
    editName = tag.name;
    editColor = tag.color;
  }

  function handleSaveEdit() {
    if (!editingTag || !editName.trim()) return;
    
    tagStore.update(editingTag.id, {
      name: editName.trim(),
      color: editColor,
    });
    
    editingTag = null;
  }

  function handleCancelEdit() {
    editingTag = null;
  }

  async function handleDeleteTag(tag: Tag) {
    if (!(await confirm(`Delete tag "${tag.name}"? This will remove it from all entities.`))) return;
    tagStore.remove(tag.id);
  }

  function handleApplyTag(tag: Tag) {
    const node = $selectedNode;
    const edge = $selectedEdge;
    const entity = node || edge;
    
    if (!entity) return;
    
    tagStore.applyToEntity(
      tag.id,
      entity.id,
      node ? 'node' : 'edge'
    );
  }

  function handleRemoveTag(tag: Tag) {
    const node = $selectedNode;
    const edge = $selectedEdge;
    const entity = node || edge;
    
    if (!entity) return;
    
    tagStore.removeFromEntity(tag.id, entity.id);
  }

  function getColorName(color: string): string {
    const names: Record<string, string> = {
      '#ef4444': 'Red',
      '#f97316': 'Orange',
      '#fbbf24': 'Yellow',
      '#22c55e': 'Green',
      '#14b8a6': 'Teal',
      '#3b82f6': 'Blue',
      '#6366f1': 'Indigo',
      '#a855f7': 'Purple',
      '#ec4899': 'Pink',
      '#6b7280': 'Gray',
    };
    return names[color] || color;
  }

  // Annotation state
  let annotationText = $state('');
  let annotationColor: AnnotationColor = $state('yellow');

  function handleAddAnnotation() {
    if (!annotationText.trim()) return;
    
    const node = $selectedNode;
    const edge = $selectedEdge;
    const entity = node || edge;
    
    if (!entity) return;
    
    annotationStore.add({
      entityId: entity.id,
      entityType: node ? 'node' : 'edge',
      text: annotationText.trim(),
      color: annotationColor,
      createdBy: 'user',
      pinned: false,
    });
    
    annotationText = '';
  }

  function handleDeleteAnnotation(annotation: Annotation) {
    annotationStore.remove(annotation.id);
  }

  function handleTogglePin(annotation: Annotation) {
    annotationStore.update(annotation.id, {
      pinned: !annotation.pinned,
    });
  }
</script>

<div class="tag-manager">
  <!-- ── Tags Section ─────────────────────────────────────────────────── -->
  <div class="tag-section">
    <h3 class="section-title">Tags</h3>
    
    <!-- Create new tag -->
    <div class="create-tag-form">
      <input
        type="text"
        class="form-input"
        placeholder="Tag name..."
        bind:value={newTagName}
        onkeydown={e => e.key === 'Enter' && handleCreateTag()}
      />
      <div class="color-picker">
        {#each DEFAULT_TAG_COLORS as color}
          <button
            class="color-option"
            class:selected={selectedColor === color}
            style="background: {color}"
            onclick={() => selectedColor = color}
            title={getColorName(color)}
            aria-label={getColorName(color)}
          />
        {/each}
      </div>
      <button
        class="btn btn-primary btn-sm"
        onclick={handleCreateTag}
        disabled={!newTagName.trim()}
      >
        + Tag
      </button>
    </div>

    <!-- Tag list -->
    <div class="tag-list">
      {#each tags as tag}
        <div class="tag-item">
          {#if editingTag?.id === tag.id}
            <!-- Edit mode -->
            <div class="tag-edit-form">
              <input
                type="text"
                class="form-input form-input-sm"
                bind:value={editName}
                placeholder="Tag name"
              />
              <div class="color-picker color-picker-sm">
                {#each DEFAULT_TAG_COLORS as color}
                  <button
                    class="color-option"
                    class:selected={editColor === color}
                    style="background: {color}"
                    onclick={() => editColor = color}
                    aria-label="Select color"
                  ></button>
                {/each}
              </div>
              <div class="tag-edit-actions">
                <button class="icon-btn success-icon" onclick={handleSaveEdit} title="Save">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                    <polyline points="20 6 9 17 4 12"/>
                  </svg>
                </button>
                <button class="icon-btn" onclick={handleCancelEdit} title="Cancel">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                    <line x1="18" y1="6" x2="6" y2="18"/>
                    <line x1="6" y1="6" x2="18" y2="18"/>
                  </svg>
                </button>
              </div>
            </div>
          {:else}
            <!-- Display mode -->
            <div class="tag-display">
              <span 
                class="tag-chip" 
                style="background: {tag.color}20; border-color: {tag.color}; color: {tag.color}"
              >
                {tag.name}
                <span class="tag-count">{tag.usageCount}</span>
              </span>
              <div class="tag-actions">
                {#if $selectedNode || $selectedEdge}
                  {#if entityTags.some(t => t.id === tag.id)}
                    <button
                      class="icon-btn danger-icon"
                      onclick={() => handleRemoveTag(tag)}
                      title="Remove from selection"
                    >
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                        <line x1="18" y1="6" x2="6" y2="18"/>
                        <line x1="6" y1="6" x2="18" y2="18"/>
                      </svg>
                    </button>
                  {:else}
                    <button
                      class="icon-btn"
                      onclick={() => handleApplyTag(tag)}
                      title="Apply to selection"
                    >
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                        <path d="M12 5v14M5 12h14"/>
                      </svg>
                    </button>
                  {/if}
                {/if}
                <button
                  class="icon-btn"
                  onclick={() => handleStartEdit(tag)}
                  title="Edit tag"
                >
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                    <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
                  </svg>
                </button>
                <button
                  class="icon-btn danger-icon"
                  onclick={() => handleDeleteTag(tag)}
                  title="Delete tag"
                >
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                    <polyline points="3 6 5 6 21 6"/>
                    <path d="M19 6l-1 14H6L5 6"/>
                  </svg>
                </button>
              </div>
            </div>
          {/if}
        </div>
      {/each}
      {#if tags.length === 0}
        <p class="empty-state">No tags yet. Create one above.</p>
      {/if}
    </div>

    {#if entityTags.length > 0}
      <div class="entity-tags-section">
        <h4 class="subsection-title">Applied to Selection</h4>
        <div class="entity-tags-list">
          {#each entityTags as tag}
            <span 
              class="entity-tag-chip"
              style="background: {tag.color}20; border-color: {tag.color}; color: {tag.color}"
            >
              {tag.name}
            </span>
          {/each}
        </div>
      </div>
    {/if}
  </div>

  <!-- ── Annotations Section ──────────────────────────────────────────── -->
  <div class="annotation-section">
    <h3 class="section-title">Annotations</h3>
    
    <!-- Add annotation -->
    <div class="add-annotation-form">
      <textarea
        class="form-input annotation-input"
        placeholder="Add a note..."
        bind:value={annotationText}
        rows="2"
      ></textarea>
      <div class="annotation-actions-row">
        <div class="color-picker">
          {#each ['yellow', 'green', 'blue', 'purple', 'red', 'orange'] as color}
            <button
              class="color-option"
              class:selected={annotationColor === color}
              style="background: {color === 'yellow' ? '#fbbf24' : color === 'green' ? '#22c55e' : color === 'blue' ? '#3b82f6' : color === 'purple' ? '#a855f7' : color === 'red' ? '#ef4444' : '#f97316'}"
              onclick={() => annotationColor = color as AnnotationColor}
              aria-label={color}
            ></button>
          {/each}
        </div>
        <button
          class="btn btn-primary btn-sm"
          onclick={handleAddAnnotation}
          disabled={!annotationText.trim()}
        >
          Add
        </button>
      </div>
    </div>

    <!-- Annotations list -->
    <div class="annotations-list">
      {#each annotations as annotation}
        <div class="annotation-item" class:pinned={annotation.pinned}>
          <div 
            class="annotation-color-bar" 
            style="background: {ANNOTATION_COLORS[annotation.color]}"
          ></div>
          <div class="annotation-content">
            <p class="annotation-text">{annotation.text}</p>
            <div class="annotation-meta">
              <span class="annotation-time">{new Date(annotation.createdAt).toLocaleString()}</span>
              {#if annotation.pinned}
                <span class="annotation-pinned">📌 Pinned</span>
              {/if}
            </div>
          </div>
          <div class="annotation-actions">
            <button
              class="icon-btn"
              onclick={() => handleTogglePin(annotation)}
              title={annotation.pinned ? 'Unpin' : 'Pin'}
            >
              {annotation.pinned ? '📌' : '📍'}
            </button>
            <button
              class="icon-btn danger-icon"
              onclick={() => handleDeleteAnnotation(annotation)}
              title="Delete"
            >
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <polyline points="3 6 5 6 21 6"/>
                <path d="M19 6l-1 14H6L5 6"/>
              </svg>
            </button>
          </div>
        </div>
      {/each}
      {#if annotations.length === 0}
        <p class="empty-state">No annotations yet.</p>
      {/if}
    </div>
  </div>
</div>

<style>
  .tag-manager {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 12px;
  }

  .section-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 10px 0;
    padding-bottom: 6px;
    border-bottom: 1px solid var(--border);
  }

  .subsection-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    margin: 12px 0 8px 0;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  /* ── Tag creation form ───────────────────────────────────────────── */
  .create-tag-form {
    display: flex;
    gap: 6px;
    align-items: center;
    margin-bottom: 10px;
  }

  .create-tag-form .form-input {
    flex: 1;
    min-width: 0;
  }

  .color-picker {
    display: flex;
    gap: 3px;
    flex-wrap: wrap;
  }

  .color-picker-sm {
    max-width: 120px;
  }

  .color-option {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    transition: transform 0.1s, border-color 0.1s;
  }

  .color-option:hover {
    transform: scale(1.1);
  }

  .color-option.selected {
    border-color: var(--text-primary);
    box-shadow: 0 0 0 2px var(--bg-base);
  }

  /* ── Tag list ─────────────────────────────────────────────────────── */
  .tag-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-height: 200px;
    overflow-y: auto;
  }

  .tag-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 8px;
    background: var(--bg-elevated);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
  }

  .tag-display {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex: 1;
    gap: 8px;
  }

  .tag-chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px;
    border-radius: 999px;
    font-size: 11px;
    font-weight: 500;
    border-width: 1px;
    border-style: solid;
  }

  .tag-count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 4px;
    background: rgba(0, 0, 0, 0.15);
    border-radius: 9px;
    font-size: 10px;
  }

  .tag-actions {
    display: flex;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .tag-item:hover .tag-actions {
    opacity: 1;
  }

  .tag-edit-form {
    display: flex;
    gap: 4px;
    align-items: center;
    flex: 1;
  }

  .form-input-sm {
    padding: 3px 6px;
    font-size: 11px;
  }

  .tag-edit-actions {
    display: flex;
    gap: 2px;
  }

  /* ── Entity tags ──────────────────────────────────────────────────── */
  .entity-tags-section {
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid var(--border);
  }

  .entity-tags-list {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .entity-tag-chip {
    display: inline-flex;
    align-items: center;
    padding: 2px 6px;
    border-radius: 999px;
    font-size: 10px;
    font-weight: 500;
    border-width: 1px;
    border-style: solid;
  }

  /* ── Annotation form ─────────────────────────────────────────────── */
  .add-annotation-form {
    margin-bottom: 10px;
  }

  .annotation-input {
    resize: vertical;
    min-height: 50px;
    font-family: inherit;
    font-size: 12px;
  }

  .annotation-actions-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
    margin-top: 6px;
  }

  /* ── Annotations list ────────────────────────────────────────────── */
  .annotations-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-height: 250px;
    overflow-y: auto;
  }

  .annotation-item {
    display: flex;
    gap: 8px;
    padding: 8px;
    background: var(--bg-elevated);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    position: relative;
  }

  .annotation-item.pinned {
    border-color: var(--accent);
    background: var(--bg-base);
  }

  .annotation-color-bar {
    width: 3px;
    min-width: 3px;
    height: 100%;
    border-radius: 2px;
    align-self: stretch;
  }

  .annotation-content {
    flex: 1;
    min-width: 0;
  }

  .annotation-text {
    font-size: 12px;
    color: var(--text-primary);
    margin: 0 0 6px 0;
    line-height: 1.4;
    word-break: break-word;
  }

  .annotation-meta {
    display: flex;
    gap: 8px;
    font-size: 10px;
    color: var(--text-muted);
  }

  .annotation-pinned {
    color: var(--accent);
    font-weight: 500;
  }

  .annotation-actions {
    display: flex;
    flex-direction: column;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .annotation-item:hover .annotation-actions {
    opacity: 1;
  }

  /* ── Empty state ─────────────────────────────────────────────────── */
  .empty-state {
    font-size: 11px;
    color: var(--text-muted);
    text-align: center;
    padding: 12px;
    font-style: italic;
  }

  /* ── Icon button ─────────────────────────────────────────────────── */
  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.1s;
  }

  .icon-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .icon-btn.danger-icon:hover {
    background: var(--error-bg);
    color: var(--error);
  }

  .icon-btn.success-icon:hover {
    background: var(--success-bg);
    color: var(--success);
  }
</style>
