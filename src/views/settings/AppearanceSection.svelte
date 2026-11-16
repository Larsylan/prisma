<script lang="ts">
  import { onMount } from 'svelte';

  import ThemePreview from '../../components/ThemePreview.svelte';
  import { api } from '../../lib/api';
  import type { Preset, ThemeOption } from '../../lib/types';
  import Section from './Section.svelte';

  let { theme = $bindable(), onChange }: { theme: Preset; onChange: () => void } = $props();

  let options = $state<ThemeOption[]>([]);

  onMount(async () => {
    options = await api.themeGallery();
  });

  function select(preset: Preset) {
    theme = preset;
    onChange();
  }
</script>

<Section title="Apparence">
  <div class="grid">
    {#each options as option (option.preset)}
      <button class="theme" class:selected={option.preset === theme} aria-pressed={option.preset === theme} onclick={() => select(option.preset)}>
        <ThemePreview palette={option.palette} />
        <span class="name">{option.name}</span>
      </button>
    {/each}
  </div>
</Section>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 12px;
  }

  .theme {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 6px;
    border-radius: 10px;
    text-align: left;
    border: 2px solid transparent;
  }

  .theme:hover {
    background: var(--card-hover);
  }

  .theme.selected {
    border-color: var(--accent);
  }

  .name {
    font-weight: 600;
  }
</style>
