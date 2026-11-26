<script lang="ts">
  import { t } from '../../lib/i18n/index.svelte';
  import type { Language } from '../../lib/types';
  import Section from './Section.svelte';
  import { TEXT_SAVE_DELAY_MS, type Save } from './saving';

  let { name = $bindable(), language = $bindable(), save }: { name: string; language: Language; save: Save } = $props();

  /** Chaque langue garde son propre nom, pour être reconnue quelle que soit la langue affichée. */
  let languageOptions = $derived<{ value: Language; label: string }[]>([
    { value: 'system', label: t.settings.systemLanguage },
    { value: 'french', label: 'Français' },
    { value: 'english', label: 'English' },
    { value: 'italian', label: 'Italiano' },
  ]);
</script>

<Section title={t.settings.profile}>
  <label class="field">
    <span>{t.settings.displayName}</span>
    <input class="text-input" bind:value={name} oninput={() => save(TEXT_SAVE_DELAY_MS)} />
  </label>
  <label class="field inline">
    <span>{t.settings.language}</span>
    <select class="select" bind:value={language} onchange={() => save()}>
      {#each languageOptions as option (option.value)}
        <option value={option.value}>{option.label}</option>
      {/each}
    </select>
  </label>
</Section>
