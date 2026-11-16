<script lang="ts">
  import type { Snippet } from 'svelte';

  let { help, spinning = false, onclick, children }: { help: string; spinning?: boolean; onclick: () => void; children: Snippet } = $props();
</script>

<button class="icon-button" class:spinning title={help} aria-label={help} {onclick}>
  {@render children()}
</button>

<style>
  .icon-button {
    width: 34px;
    height: 34px;
    border-radius: 50%;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: rgb(var(--ink) / 0.08);
  }

  .icon-button:hover {
    background: rgb(var(--ink) / 0.14);
  }

  .icon-button :global(svg) {
    width: 16px;
    height: 16px;
  }

  .spinning :global(svg) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
