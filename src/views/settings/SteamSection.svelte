<script lang="ts">
  import { onMount } from 'svelte';

  import ExternalLink from '../../components/ExternalLink.svelte';
  import { api } from '../../lib/api';
  import { appState } from '../../lib/state.svelte';
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

  onMount(async () => {
    apiKey = await api.steamApiKey();
  });
</script>

<Section
  title="Steam"
  footer="Sans clé, les succès des jeux lancés sur cet ordinateur sont lus directement depuis Steam. La clé permet d'importer l'historique de tous vos jeux (profil public requis) ; elle est rangée dans le coffre de votre système. La rareté sert uniquement à attribuer bronze/argent/or : elle n'est jamais affichée."
>
  <div class="value"><span>État</span><span class="secondary">{appState.status?.steamLocal || '—'}</span></div>
  {#if appState.status?.steamWeb}
    <div class="value"><span>Historique</span><span class="secondary">{appState.status.steamWeb}</span></div>
  {/if}
  <div class="value">
    <span>Détection</span>
    <span class="secondary">{appState.status?.steamWatching ? 'Temps réel (cache local du client Steam)' : 'Inactive'}</span>
  </div>
  <label class="field">
    <span>Clé Steam Web API (optionnelle)</span>
    <input class="text-input" type="password" bind:value={apiKey} oninput={() => saveApiKey(TEXT_SAVE_DELAY_MS)} />
  </label>
  <label class="field">
    <span>SteamID64</span>
    <input class="text-input" placeholder={detectedSteamId64 ?? '7656119…'} bind:value={steamId64} oninput={() => save(TEXT_SAVE_DELAY_MS)} />
  </label>
  <label class="toggle">
    <input type="checkbox" bind:checked={useRarity} onchange={() => save()} />
    Calculer les grades via la rareté Steam
  </label>
  <div class="buttons">
    <button class="button" disabled={!apiKey.trim() || appState.status?.steamWebImporting} onclick={() => api.importSteamHistory()}>
      Importer l'historique complet
    </button>
    <ExternalLink href="https://steamcommunity.com/dev/apikey">Obtenir une clé</ExternalLink>
  </div>
</Section>
