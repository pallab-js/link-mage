<script lang="ts">
  import { invoke }          from '@tauri-apps/api/core';
  import { settingsOpenStore } from '$lib/stores';
  import { setStatus }       from '$lib/stores';

  // ── Types ──────────────────────────────────────────────────────────────────

  interface ApiServiceInfo {
    id:          string;
    name:        string;
    description: string;
    key_url:     string;
    has_key:     boolean;
  }

  interface RemoteTransform {
    name:             string;
    description:      string;
    url:              string;
    applicable_types: string[];
  }

  // ── State ──────────────────────────────────────────────────────────────────

  let services     = $state<ApiServiceInfo[]>([]);
  let loadError    = $state<string | null>(null);
  let loading      = $state(false);
  /** Editing state: keyed by service id */
  let editing      = $state<Record<string, boolean>>({});
  let inputValues  = $state<Record<string, string>>({});
  let showValues   = $state<Record<string, boolean>>({});
  let savingKey    = $state<string | null>(null);
  let deletingKey  = $state<string | null>(null);

  let remoteTransforms = $state<RemoteTransform[]>([]);
  let newRemote = $state({
    name:             '',
    description:      '',
    url:              '',
    applicable_types: '',
  });
  let addingRemote = $state(false);

  // ── Load services when the panel opens ────────────────────────────────────

  $effect(() => {
    if ($settingsOpenStore) {
      loadServices();
      loadRemoteTransforms();
    }
  });

  async function loadServices() {
    loading   = true;
    loadError = null;
    try {
      services = await invoke<ApiServiceInfo[]>('list_api_services');
      // Reset editing state
      editing     = {};
      inputValues = {};
      showValues  = {};
    } catch (e) {
      loadError = String(e);
    } finally {
      loading = false;
    }
  }

  async function loadRemoteTransforms() {
    try {
      remoteTransforms = await invoke<RemoteTransform[]>('list_remote_transforms');
    } catch (e) {
      console.error('Failed to load remote transforms:', e);
    }
  }

  // ── Edit helpers ───────────────────────────────────────────────────────────

  function startEdit(id: string) {
    editing     = { ...editing,     [id]: true  };
    inputValues = { ...inputValues, [id]: ''    };
    showValues  = { ...showValues,  [id]: false };
  }

  function cancelEdit(id: string) {
    editing = { ...editing, [id]: false };
  }

  async function revealKey(id: string) {
    try {
      const key = await invoke<string | null>('get_api_key', { service: id });
      if (key) {
        inputValues = { ...inputValues, [id]: key  };
        showValues  = { ...showValues,  [id]: true };
      }
    } catch (e) {
      setStatus('error', `Failed to retrieve key: ${e}`);
    }
  }

  async function saveKey(id: string) {
    const key = (inputValues[id] ?? '').trim();
    if (!key) {
      setStatus('error', 'API key must not be empty');
      return;
    }
    savingKey = id;
    try {
      await invoke('set_api_key', { service: id, key });
      setStatus('success', `${services.find(s => s.id === id)?.name ?? id} API key saved`);
      editing = { ...editing, [id]: false };
      // Refresh list so the "has_key" badge updates.
      await loadServices();
    } catch (e) {
      setStatus('error', `Save failed: ${e}`);
    } finally {
      savingKey = null;
    }
  }

  async function deleteKey(id: string) {
    deletingKey = id;
    try {
      await invoke('delete_api_key', { service: id });
      setStatus('success', `${services.find(s => s.id === id)?.name ?? id} API key removed`);
      await loadServices();
    } catch (e) {
      setStatus('error', `Delete failed: ${e}`);
    } finally {
      deletingKey = null;
    }
  }

  async function addRemoteTransform() {
    if (!newRemote.name || !newRemote.url) {
      setStatus('error', 'Name and URL are required');
      return;
    }
    addingRemote = true;
    try {
      const transform = {
        ...newRemote,
        applicable_types: newRemote.applicable_types.split(',').map(s => s.trim()).filter(Boolean)
      };
      await invoke('add_remote_transform', { transform });
      setStatus('success', `Remote transform '${newRemote.name}' added`);
      newRemote = { name: '', description: '', url: '', applicable_types: '' };
      await loadRemoteTransforms();
    } catch (e) {
      setStatus('error', `Failed to add remote transform: ${e}`);
    } finally {
      addingRemote = false;
    }
  }

  async function removeRemoteTransform(name: string) {
    try {
      await invoke('remove_remote_transform', { name });
      setStatus('success', `Remote transform '${name}' removed`);
      await loadRemoteTransforms();
    } catch (e) {
      setStatus('error', `Failed to remove remote transform: ${e}`);
    }
  }

  function close() {
    settingsOpenStore.set(false);
  }

  function handleBackdropClick(e: MouseEvent) {
    if ((e.target as HTMLElement).classList.contains('settings-backdrop')) close();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if $settingsOpenStore}
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="settings-backdrop" onclick={handleBackdropClick}>
  <div class="settings-panel" role="dialog" aria-modal="true" aria-label="Settings" tabindex="-1">

    <!-- Header -->
    <div class="settings-header">
      <h2>Settings</h2>
      <button class="close-btn" onclick={close} aria-label="Close settings">✕</button>
    </div>

    <!-- Body -->
    <div class="settings-body">

      <!-- ── API Keys section ─────────────────────────────────────────────── -->
      <section>
        <h3 class="section-title">API Keys</h3>
        <p class="section-desc">
          Keys are stored in the OS keychain (macOS Keychain, Linux Secret
          Service, Windows Credential Manager) — never in plain text on disk.
        </p>

        {#if loading}
          <p class="msg-info">Loading…</p>
        {:else if loadError}
          <p class="msg-error">{loadError}</p>
        {:else if services.length === 0}
          <p class="msg-info">No API services configured.</p>
        {:else}
          <ul class="service-list">
            {#each services as svc (svc.id)}
              <li class="service-item">
                <div class="service-row">
                  <div class="service-meta">
                    <span class="service-name">{svc.name}</span>
                    {#if svc.has_key}
                      <span class="badge badge-ok">key stored</span>
                    {:else}
                      <span class="badge badge-missing">no key</span>
                    {/if}
                  </div>
                  <p class="service-desc">{svc.description}</p>
                  <p class="service-link">
                    Get your key at
                    <!-- svelte-ignore a11y_invalid_attribute -->
                    <a href={svc.key_url} target="_blank" rel="noopener noreferrer">{svc.key_url}</a>
                  </p>
                </div>

                {#if editing[svc.id]}
                  <!-- Edit form -->
                  <div class="key-form">
                    <div class="input-row">
                      <input
                        type={showValues[svc.id] ? 'text' : 'password'}
                        placeholder="Paste API key…"
                        bind:value={inputValues[svc.id]}
                        onkeydown={(e) => { if (e.key === 'Enter') saveKey(svc.id); }}
                        class="key-input"
                      />
                      <button
                        class="icon-btn"
                        onclick={() => showValues = { ...showValues, [svc.id]: !showValues[svc.id] }}
                        title={showValues[svc.id] ? 'Hide' : 'Show'}
                        aria-label={showValues[svc.id] ? 'Hide key' : 'Show key'}
                      >
                        {showValues[svc.id] ? '🙈' : '👁'}
                      </button>
                    </div>
                    <div class="form-actions">
                      <button
                        class="btn btn-primary"
                        onclick={() => saveKey(svc.id)}
                        disabled={savingKey === svc.id}
                      >
                        {savingKey === svc.id ? 'Saving…' : 'Save'}
                      </button>
                      <button class="btn btn-ghost" onclick={() => cancelEdit(svc.id)}>
                        Cancel
                      </button>
                    </div>
                  </div>
                {:else}
                  <!-- Action buttons -->
                  <div class="service-actions">
                    {#if svc.has_key}
                      <button class="btn btn-secondary" onclick={() => startEdit(svc.id)}>
                        Update key
                      </button>
                      <button
                        class="btn btn-danger"
                        onclick={() => deleteKey(svc.id)}
                        disabled={deletingKey === svc.id}
                      >
                        {deletingKey === svc.id ? 'Removing…' : 'Remove key'}
                      </button>
                      <button class="btn btn-ghost" onclick={() => revealKey(svc.id)}>
                        View key
                      </button>
                    {:else}
                      <button class="btn btn-primary" onclick={() => startEdit(svc.id)}>
                        Add key
                      </button>
                    {/if}
                  </div>
                {/if}
              </li>
            {/each}
          </ul>
        {/if}
      </section>

      <!-- ── Remote Transforms section ────────────────────────────────────── -->
      <section style="margin-top: 2rem; border-top: 1px solid var(--border); padding-top: 2rem;">
        <h3 class="section-title">Remote Transforms</h3>
        <p class="section-desc">
          Add external HTTP endpoints that implement the Link Mage transform protocol.
        </p>

        <!-- Add Form -->
        <div class="service-item" style="margin-bottom: 1.5rem; background: var(--bg-secondary);">
          <h4 style="font-size: 0.85rem; margin: 0 0 1rem; color: var(--text-primary);">Add Remote Transform</h4>
          <div class="key-form">
            <input
              type="text"
              placeholder="Name (e.g. Reverse DNS)"
              bind:value={newRemote.name}
              class="key-input"
              style="font-family: inherit;"
            />
            <input
              type="text"
              placeholder="Description"
              bind:value={newRemote.description}
              class="key-input"
              style="font-family: inherit;"
            />
            <input
              type="text"
              placeholder="URL (https://...)"
              bind:value={newRemote.url}
              class="key-input"
              style="font-family: inherit;"
            />
            <input
              type="text"
              placeholder="Applicable Types (e.g. domain, ip - comma separated)"
              bind:value={newRemote.applicable_types}
              class="key-input"
              style="font-family: inherit;"
            />

            <div class="form-actions" style="margin-top: 0.5rem;">
              <button
                class="btn btn-primary"
                onclick={addRemoteTransform}
                disabled={addingRemote}
              >
                {addingRemote ? 'Adding…' : 'Add Transform'}
              </button>
            </div>
          </div>
        </div>

        {#if remoteTransforms.length === 0}
          <p class="msg-info">No remote transforms configured.</p>
        {:else}
          <ul class="service-list">
            {#each remoteTransforms as rt (rt.name)}
              <li class="service-item">
                <div class="service-row">
                  <div class="service-meta">
                    <span class="service-name">{rt.name}</span>
                    <span class="badge badge-ok" style="background: rgba(88, 166, 255, .2); color: var(--accent);">remote</span>
                  </div>
                  <p class="service-desc">{rt.description}</p>
                  <p class="service-link">{rt.url}</p>
                  {#if rt.applicable_types.length > 0}
                    <div style="margin-top: 0.5rem; display: flex; gap: 0.25rem; flex-wrap: wrap;">
                      {#each rt.applicable_types as type}
                        <span class="badge" style="background: var(--bg-hover); color: var(--text-secondary); text-transform: none;">{type}</span>
                      {/each}
                    </div>
                  {/if}
                </div>
                <div class="service-actions">
                  <button class="btn btn-danger" onclick={() => removeRemoteTransform(rt.name)}>
                    Remove
                  </button>
                </div>
              </li>
            {/each}
          </ul>
        {/if}
      </section>

    </div><!-- /settings-body -->
  </div>
</div>
{/if}

<style>
  .settings-backdrop {
    position:         fixed;
    inset:            0;
    background:       rgba(0, 0, 0, .45);
    backdrop-filter:  blur(2px);
    z-index:          200;
    display:          flex;
    justify-content:  flex-end;
  }

  .settings-panel {
    width:            min(420px, 95vw);
    height:           100%;
    background:       var(--bg-secondary);
    border-left:      1px solid var(--border);
    display:          flex;
    flex-direction:   column;
    overflow:         hidden;
    box-shadow:       -4px 0 24px rgba(0, 0, 0, .35);
    animation:        slide-in 0.2s ease;
  }

  @keyframes slide-in {
    from { transform: translateX(100%); }
    to   { transform: translateX(0);    }
  }

  .settings-header {
    display:        flex;
    align-items:    center;
    justify-content: space-between;
    padding:        1rem 1.25rem;
    border-bottom:  1px solid var(--border);
    flex-shrink:    0;
  }

  .settings-header h2 {
    margin:      0;
    font-size:   1rem;
    font-weight: 600;
    color:       var(--text-primary);
  }

  .close-btn {
    background:  transparent;
    border:      none;
    color:       var(--text-secondary);
    cursor:      pointer;
    font-size:   1rem;
    line-height: 1;
    padding:     .25rem;
    border-radius: 4px;
    transition:  background 0.15s, color 0.15s;
  }
  .close-btn:hover {
    background: var(--bg-hover);
    color:      var(--text-primary);
  }

  .settings-body {
    flex:     1;
    overflow-y: auto;
    padding:  1.25rem;
  }

  .section-title {
    font-size:   .75rem;
    font-weight: 600;
    letter-spacing: .08em;
    text-transform: uppercase;
    color:       var(--text-secondary);
    margin:      0 0 .5rem;
  }

  .section-desc {
    font-size: .8rem;
    color:     var(--text-secondary);
    margin:    0 0 1rem;
    line-height: 1.5;
  }

  .msg-info  { color: var(--text-secondary); font-size: .85rem; }
  .msg-error { color: var(--error, #e55); font-size: .85rem; }

  .service-list {
    list-style: none;
    margin:     0;
    padding:    0;
    display:    flex;
    flex-direction: column;
    gap:        .75rem;
  }

  .service-item {
    background:    var(--bg-primary);
    border:        1px solid var(--border);
    border-radius: 8px;
    padding:       .9rem 1rem;
  }

  .service-row {
    margin-bottom: .65rem;
  }

  .service-meta {
    display:     flex;
    align-items: center;
    gap:         .5rem;
    margin-bottom: .3rem;
  }

  .service-name {
    font-size:   .9rem;
    font-weight: 600;
    color:       var(--text-primary);
  }

  .badge {
    font-size:   .65rem;
    font-weight: 600;
    letter-spacing: .04em;
    text-transform: uppercase;
    padding:     2px 6px;
    border-radius: 4px;
  }
  .badge-ok      { background: rgba(72, 199, 116, .2); color: #48c774; }
  .badge-missing { background: rgba(255, 180,  0, .2); color: #ffb400; }

  .service-desc {
    font-size: .8rem;
    color:     var(--text-secondary);
    margin:    0 0 .2rem;
  }

  .service-link {
    font-size: .75rem;
    color:     var(--text-secondary);
    margin:    0;
    word-break: break-all;
  }
  .service-link a {
    color: var(--accent, #58a6ff);
    text-decoration: none;
  }
  .service-link a:hover { text-decoration: underline; }

  .service-actions {
    display:   flex;
    flex-wrap: wrap;
    gap:       .5rem;
  }

  .key-form {
    display:        flex;
    flex-direction: column;
    gap:            .5rem;
  }

  .input-row {
    display: flex;
    gap:     .4rem;
  }

  .key-input {
    flex:          1;
    background:    var(--bg-secondary);
    border:        1px solid var(--border);
    border-radius: 6px;
    padding:       .45rem .65rem;
    font-size:     .85rem;
    color:         var(--text-primary);
    font-family:   monospace;
    outline:       none;
    transition:    border-color 0.15s;
  }
  .key-input:focus { border-color: var(--accent, #58a6ff); }

  .icon-btn {
    background:    var(--bg-secondary);
    border:        1px solid var(--border);
    border-radius: 6px;
    padding:       0 .6rem;
    cursor:        pointer;
    font-size:     .9rem;
    color:         var(--text-secondary);
    transition:    background 0.15s;
    line-height:   1;
  }
  .icon-btn:hover { background: var(--bg-hover); }

  .form-actions {
    display: flex;
    gap:     .5rem;
  }

  /* Shared button styles */
  .btn {
    padding:       .4rem .85rem;
    border-radius: 6px;
    font-size:     .82rem;
    font-weight:   600;
    cursor:        pointer;
    border:        1px solid transparent;
    transition:    background 0.15s, border-color 0.15s, opacity 0.15s;
  }
  .btn:disabled { opacity: .5; cursor: not-allowed; }

  .btn-primary   { background: var(--accent, #58a6ff); color: #fff; border-color: var(--accent, #58a6ff); }
  .btn-primary:hover:not(:disabled) { filter: brightness(1.15); }

  .btn-secondary { background: var(--bg-hover); color: var(--text-primary); border-color: var(--border); }
  .btn-secondary:hover:not(:disabled) { border-color: var(--accent, #58a6ff); }

  .btn-danger    { background: transparent; color: #e55; border-color: #e55; }
  .btn-danger:hover:not(:disabled) { background: rgba(220, 80, 80, .15); }

  .btn-ghost     { background: transparent; color: var(--text-secondary); border-color: var(--border); }
  .btn-ghost:hover:not(:disabled) { background: var(--bg-hover); color: var(--text-primary); }
</style>
