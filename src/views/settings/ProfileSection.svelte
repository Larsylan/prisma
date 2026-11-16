<script lang="ts">
  import Section from './Section.svelte';
  import { TEXT_SAVE_DELAY_MS, type Save } from './saving';

  let { name = $bindable(), save }: { name: string; save: Save } = $props();
</script>

<Section title="Profil">
  <label class="field">
    <span>Nom affiché</span>
    <input class="text-input" bind:value={name} oninput={() => save(TEXT_SAVE_DELAY_MS)} />
  </label>
</Section>
