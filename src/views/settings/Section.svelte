<script lang="ts">
  import type { Snippet } from 'svelte';

  let { title, footer, children }: { title: string; footer?: string; children: Snippet } = $props();
</script>

<section>
  <h2>{title}</h2>
  <div class="content">{@render children()}</div>
  {#if footer}<p class="secondary">{footer}</p>{/if}
</section>

<style>
  h2 {
    margin: 0 0 8px;
    font-size: 13px;
    font-weight: 600;
    color: var(--secondary);
  }

  .content {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px;
    border-radius: 10px;
    background: var(--card);
    border: 1px solid var(--stroke);
  }

  /* Éléments de formulaire communs à toutes les sections. */
  .content :global(.field) {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .content :global(.field.inline) {
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
  }

  .content :global(.value) {
    display: flex;
    justify-content: space-between;
    gap: 20px;
  }

  .content :global(.value .secondary) {
    text-align: right;
  }

  .content :global(.toggle) {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .content :global(.buttons) {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;
  }

  p {
    margin: 8px 2px 0;
    font-size: 11px;
    line-height: 1.45;
  }
</style>
