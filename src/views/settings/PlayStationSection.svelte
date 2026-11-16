<script lang="ts">
  import ExternalLink from '../../components/ExternalLink.svelte';
  import { api } from '../../lib/api';
  import { appState } from '../../lib/state.svelte';
  import Section from './Section.svelte';

  let { polling = $bindable(), onPollingChange }: { polling: boolean; onPollingChange: () => void } = $props();

  const MINIMUM_TOKEN_LENGTH = 20;

  let npsso = $state('');
  let removesTrophiesOnDisconnect = $state(false);
  let isConnected = $derived(appState.status?.playstationConnected ?? false);

  function connect() {
    const token = npsso;
    npsso = '';
    api.connectPlaystation(token).catch(() => {
      // L'erreur est affichée par l'état de synchronisation.
    });
  }
</script>

<Section
  title="PlayStation Network"
  footer="Le jeton n'est utilisé qu'une fois pour obtenir un accès stocké dans le coffre de votre système. Il expire après environ 2 mois."
>
  {#if isConnected}
    <div class="value"><span>Compte</span><span class="secondary">Connecté</span></div>
    <span class="secondary small">{appState.status?.playstation}</span>
    <label class="toggle">
      <input type="checkbox" bind:checked={polling} onchange={onPollingChange} />
      Notifier les trophées gagnés sur console
    </label>
    <div class="buttons">
      <button class="button" onclick={() => api.syncPlaystation()}>Synchroniser maintenant</button>
      <span class="spacer"></span>
      <label class="toggle">
        <input type="checkbox" bind:checked={removesTrophiesOnDisconnect} />
        Supprimer les trophées importés
      </label>
      <button class="button danger" onclick={() => api.disconnectPlaystation(removesTrophiesOnDisconnect)}>Déconnecter</button>
    </div>
  {:else}
    <ol>
      <li>Connectez-vous sur playstation.com dans votre navigateur.</li>
      <li>
        Ouvrez <ExternalLink href="https://ca.account.sony.com/api/v1/ssocookie">ca.account.sony.com/api/v1/ssocookie</ExternalLink>
        et copiez la valeur « npsso ».
      </li>
    </ol>
    <input class="text-input" type="password" placeholder="Jeton NPSSO (64 caractères)" bind:value={npsso} />
    <div class="buttons">
      <button class="button primary" disabled={npsso.trim().length < MINIMUM_TOKEN_LENGTH} onclick={connect}>Connecter</button>
      <span class="secondary small">{appState.status?.playstation}</span>
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
