<script lang="ts">
  import { onMount } from 'svelte';

  import GameCover from '../components/GameCover.svelte';
  import SearchField from '../components/SearchField.svelte';
  import TierCounts from '../components/TierCounts.svelte';
  import { api } from '../lib/api';
  import { appState } from '../lib/state.svelte';
  import type { CatalogItem } from '../lib/types';

  /** Ajout d'un jeu Switch : choix dans le catalogue, ou jeu libre doté d'une liste générique. */
  let { onClose }: { onClose: () => void } = $props();

  let catalog = $state<CatalogItem[]>([]);
  let search = $state('');
  let customTitle = $state('');

  let visibleEntries = $derived(catalog.filter((item) => item.title.toLocaleLowerCase('fr').includes(search.trim().toLocaleLowerCase('fr'))));

  onMount(loadCatalog);

  $effect(() => {
    if (appState.revision > 0) loadCatalog();
  });

  async function loadCatalog() {
    catalog = await api.switchCatalog();
  }

  async function addCustomGame() {
    if (!customTitle.trim()) return;
    await api.addCustomSwitchGame(customTitle);
    customTitle = '';
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="backdrop" role="presentation" onclick={onClose}>
  <div class="sheet" role="dialog" aria-modal="true" aria-label="Ajouter un jeu Switch" tabindex="-1" onclick={(event) => event.stopPropagation()} onkeydown={() => {}}>
    <div class="heading">
      <h2>Ajouter un jeu Switch</h2>
      <button class="button primary" onclick={onClose}>Terminé</button>
    </div>
    <p class="secondary">
      La Switch n'a pas de système de succès. Chaque jeu reçoit donc une liste de trophées sur le modèle PS5 : cochez-les au fil de votre
      progression, le platine s'obtient automatiquement.
    </p>

    <SearchField placeholder="Rechercher dans le catalogue" bind:value={search} />

    <div class="entries">
      {#each visibleEntries as item (item.id)}
        <div class="entry">
          <GameCover title={item.title} coverUrl={null} platform="nintendo" width={64} height={48} />
          <div class="entry-text">
            <span class="title">{item.title}</span>
            <TierCounts counts={item.trophyCounts.map(({ tier, count }) => ({ tier, text: String(count) }))} size={11} />
          </div>
          <button class="button" disabled={item.isAdded} onclick={() => api.addSwitchGame(item.id)}>{item.isAdded ? 'Ajouté' : 'Ajouter'}</button>
        </div>
      {/each}
    </div>

    <h3>Jeu absent du catalogue</h3>
    <form
      class="custom"
      onsubmit={(event) => {
        event.preventDefault();
        addCustomGame();
      }}
    >
      <input class="text-input" placeholder="Titre du jeu" bind:value={customTitle} />
      <button class="button" type="submit" disabled={!customTitle.trim()}>Ajouter</button>
    </form>
    <p class="secondary small">Il recevra une liste générique : premiers pas, boss, fin de l'histoire, 100 %, difficulté maximale…</p>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.55);
    z-index: 10;
  }

  .sheet {
    width: 600px;
    height: min(640px, calc(100vh - 60px));
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 24px;
    border-radius: 16px;
    background: var(--background);
    border: 1px solid var(--stroke);
    box-shadow: 0 30px 80px rgba(0, 0, 0, 0.6);
  }

  .heading {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  h2 {
    margin: 0;
    font-size: 22px;
  }

  h3 {
    margin: 0;
    font-size: 13px;
  }

  p {
    margin: 0;
    font-size: 12px;
    line-height: 1.4;
  }

  .small {
    font-size: 11px;
  }

  .entries {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .entry {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 10px;
    border-radius: 10px;
    background: var(--card);
  }

  .entry-text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .title {
    font-size: 14px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .custom {
    display: flex;
    gap: 8px;
  }
</style>
