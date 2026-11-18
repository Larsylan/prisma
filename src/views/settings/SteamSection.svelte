<script lang="ts">
  import { onMount } from 'svelte';

  import ExternalLink from '../../components/ExternalLink.svelte';
  import { api } from '../../lib/api';
  import { t } from '../../lib/i18n/index.svelte';
  import { appState } from '../../lib/state.svelte';
  import { steamLocalText, steamWebText } from '../../lib/status';
  import Section from './Section.svelte';
  import { debounced, TEXT_SAVE_DELAY_MS, type Save } from './saving';

  let {
    steamId64 = $bindable(),
    useRarity = $bindable(),
    detectedSteamId64,
    save,
  }: { steamId64: string; useRarity: boolean; detectedSteamId64: string | null; save: Save } = $props();

  /** La clé est rangée dans le coffre du système, pas dans les réglages. */
  let apiKey = $state('');
  const saveApiKey = debounced(() => api.setSteamApiKey(apiKey));

  let steamLocal = $derived(appState.status ? steamLocalText(appState.status.steamLocal) : '');
  let steamWeb = $derived(appState.status ? steamWebText(appState.status.steamWeb) : '');

  onMount(async () => {
    apiKey = await api.steamApiKey();
  });
</script>

<Section title="Steam" footer={t.settings.steamFooter}>
  <div class="value"><span>{t.settings.steamState}</span><span class="secondary">{steamLocal || '—'}</span></div>
  {#if steamWeb}
    <div class="value"><span>{t.settings.steamHistory}</span><span class="secondary">{steamWeb}</span></div>
  {/if}
  <div class="value">
    <span>{t.settings.steamDetection}</span>
    <span class="secondary">{appState.status?.steamWatching ? t.settings.steamRealTime : t.settings.steamInactive}</span>
  </div>
  <label class="field">
    <span>{t.settings.steamApiKey}</span>
    <input class="text-input" type="password" bind:value={apiKey} oninput={() => saveApiKey(TEXT_SAVE_DELAY_MS)} />
  </label>
  <label class="field">
    <span>SteamID64</span>
    <input class="text-input" placeholder={detectedSteamId64 ?? '7656119…'} bind:value={steamId64} oninput={() => save(TEXT_SAVE_DELAY_MS)} />
  </label>
  <label class="toggle">
    <input type="checkbox" bind:checked={useRarity} onchange={() => save()} />
    {t.settings.steamRarity}
  </label>
  <div class="buttons">
    <button class="button" disabled={!apiKey.trim() || appState.status?.steamWeb.state === 'importing'} onclick={() => api.importSteamHistory()}>
      {t.settings.steamImport}
    </button>
    <ExternalLink href="https://steamcommunity.com/dev/apikey">{t.settings.steamGetKey}</ExternalLink>
  </div>
</Section>
