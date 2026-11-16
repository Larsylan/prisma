<script lang="ts">
  import Icon from './Icon.svelte';

  let { placeholder, value = $bindable() }: { placeholder: string; value: string } = $props();
</script>

<label class="search">
  <Icon name="search" />
  <input type="search" {placeholder} bind:value spellcheck="false" />
</label>

<style>
  .search {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    border-radius: 999px;
    background: var(--field);
    color: var(--secondary);
  }

  .search :global(svg) {
    width: 14px;
    height: 14px;
    flex-shrink: 0;
  }

  input {
    flex: 1;
    min-width: 0;
    border: none;
    outline: none;
    background: none;
    color: var(--text);
  }
</style>
