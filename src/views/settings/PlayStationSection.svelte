<script lang="ts">
  import ExternalLink from '../../components/ExternalLink.svelte';
  import { api } from '../../lib/api';
  import { t } from '../../lib/i18n/index.svelte';
  import { appState } from '../../lib/state.svelte';
  import { playstationText } from '../../lib/status';
  import Section from './Section.svelte';

  let { polling = $bindable(), onPollingChange }: { polling: boolean; onPollingChange: () => void } = $props();

  const MINIMUM_TOKEN_LENGTH = 20;

  let npsso = $state('');
  let removesTrophiesOnDisconnect = $state(false);
  let isConnected = $derived(appState.status?.playstationConnected ?? false);
  let message = $derived(appState.status ? playstationText(appState.status.playstation) : '');

  function connect() {
    const token = npsso;
    npsso = '';
    api.connectPlaystation(token).catch(() => {
      // L'erreur est affichée par l'état de synchronisation.
    });
  }
</script>

<Section title="PlayStation Network" footer={t.settings.playstationFooter}>
  {#if isConnected}
    <div class="value"><span>{t.settings.playstationAccount}</span><span class="secondary">{t.settings.playstationConnected}</span></div>
    <span class="secondary small">{message}</span>
    <label class="toggle">
      <input type="checkbox" bind:checked={polling} onchange={onPollingChange} />
      {t.settings.playstationPolling}
    </label>
    <div class="buttons">
      <button class="button" onclick={() => api.syncPlaystation()}>{t.settings.playstationSyncNow}</button>
      <span class="spacer"></span>
      <label class="toggle">
        <input type="checkbox" bind:checked={removesTrophiesOnDisconnect} />
        {t.settings.playstationRemoveTrophies}
      </label>
      <button class="button danger" onclick={() => api.disconnectPlaystation(removesTrophiesOnDisconnect)}>{t.settings.playstationDisconnect}</button>
    </div>
  {:else}
    <ol>
      <li>{t.settings.playstationSignIn}</li>
      <li>
        {t.settings.playstationOpen}
        <ExternalLink href="https://ca.account.sony.com/api/v1/ssocookie">ca.account.sony.com/api/v1/ssocookie</ExternalLink>
        {t.settings.playstationCopy}
      </li>
    </ol>
    <input class="text-input" type="password" placeholder={t.settings.playstationToken} bind:value={npsso} />
    <div class="buttons">
      <button class="button primary" disabled={npsso.trim().length < MINIMUM_TOKEN_LENGTH} onclick={connect}>{t.settings.playstationConnect}</button>
      <span class="secondary small">{message}</span>
    </div>
  {/if}
</Section>

<style>
  .small {
    font-size: 11px;
  }

  .spacer {
    flex: 1;
  }

  ol {
    margin: 0;
    padding-left: 18px;
    line-height: 1.6;
  }
</style>
