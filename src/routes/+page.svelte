<script lang="ts">
  import Sidebar from '$lib/components/Sidebar.svelte';
  import Graph from '$lib/components/Graph.svelte';
  import KeyboardShortcuts from '$lib/components/KeyboardShortcuts.svelte';
  import { sidebarOpenStore } from '$lib/stores';

  let graphRef: Graph | null = $state(null);
</script>

{#if $sidebarOpenStore}
  <Sidebar
    onFit={()    => graphRef?.fit()}
    onLayout={()  => graphRef?.runLayout()}
    onZoomIn={()  => graphRef?.zoomIn()}
    onZoomOut={()  => graphRef?.zoomOut()}
  />
{/if}

<main class="canvas-area">
  <Graph bind:this={graphRef} />
</main>

<!-- Keyboard shortcuts live outside the sidebar/canvas split so they
     capture events from anywhere in the window. -->
<KeyboardShortcuts onFit={() => graphRef?.fit()} />

<style>
  .canvas-area {
    flex: 1;
    position: relative;
    overflow: hidden;
    min-width: 0;
  }
</style>
