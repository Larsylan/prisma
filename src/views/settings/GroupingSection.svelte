<script lang="ts">
  import { t } from '../../lib/i18n/index.svelte';
  import type { GroupingMode } from '../../lib/types';
  import Section from './Section.svelte';
  import type { Save } from './saving';

  let { mode = $bindable(), save }: { mode: GroupingMode; save: Save } = $props();
</script>

<Section title={t.settings.grouping} footer={t.settings.groupingFooter}>
  <label class="field inline">
    <span>{t.settings.groupingLabel}</span>
    <select class="select" bind:value={mode} onchange={() => save()}>
      {#each Object.entries(t.settings.groupingModes) as [value, label] (value)}
        <option value={value as GroupingMode}>{label}</option>
      {/each}
    </select>
  </label>
</Section>
