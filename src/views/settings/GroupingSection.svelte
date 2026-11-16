<script lang="ts">
  import { groupingLabels } from '../../lib/format';
  import type { GroupingMode } from '../../lib/types';
  import Section from './Section.svelte';
  import type { Save } from './saving';

  let { mode = $bindable(), save }: { mode: GroupingMode; save: Save } = $props();
</script>

<Section
  title="Regroupement"
  footer="Un jeu présent sur plusieurs systèmes (PS3, PS4, PS5, Vita, Steam, Switch) n'a qu'une entrée : les trophées de même nom sont fusionnés et comptés une seule fois."
>
  <label class="field inline">
    <span>Regrouper un même jeu</span>
    <select class="select" bind:value={mode} onchange={() => save()}>
      {#each Object.entries(groupingLabels) as [value, label] (value)}
        <option value={value as GroupingMode}>{label}</option>
      {/each}
    </select>
  </label>
</Section>
