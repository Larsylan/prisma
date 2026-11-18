<script lang="ts">
  import Crystal from '../../components/Crystal.svelte';
  import { api } from '../../lib/api';
  import { t } from '../../lib/i18n/index.svelte';
  import type { ToastPosition, TrophyTier } from '../../lib/types';
  import Section from './Section.svelte';
  import type { Save } from './saving';

  let {
    soundEnabled = $bindable(),
    position = $bindable(),
    durationSeconds = $bindable(),
    save,
  }: { soundEnabled: boolean; position: ToastPosition; durationSeconds: number; save: Save } = $props();

  const TIERS: TrophyTier[] = ['bronze', 'silver', 'gold', 'platinum'];
</script>

<Section title={t.settings.notifications}>
  <label class="toggle">
    <input type="checkbox" bind:checked={soundEnabled} onchange={() => save()} />
    {t.settings.sound}
  </label>
  <label class="field inline">
    <span>{t.settings.position}</span>
    <select class="select" bind:value={position} onchange={() => save()}>
      {#each Object.entries(t.settings.positions) as [value, label] (value)}
        <option value={value as ToastPosition}>{label}</option>
      {/each}
    </select>
  </label>
  <label class="field inline">
    <span>{t.settings.duration(durationSeconds)}</span>
    <input type="range" min="3" max="12" step="1" bind:value={durationSeconds} onchange={() => save()} />
  </label>
  <div class="buttons">
    {#each TIERS as tier (tier)}
      <button class="button test" onclick={() => api.testNotification(tier)}><Crystal {tier} size={13} />{t.tiers[tier]}</button>
    {/each}
  </div>
</Section>

<style>
  .test {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
</style>
