<script lang="ts">
  import { onMount } from 'svelte';

  import { api, events } from './lib/api';
  import { appState } from './lib/state.svelte';
  import type { GameDetail as GameDetailData } from './lib/types';
  import GameDetail from './views/game/GameDetail.svelte';
  import Library from './views/library/Library.svelte';
  import Settings from './views/settings/Settings.svelte';
  import SwitchPicker from './views/SwitchPicker.svelte';

  type View = { name: 'library' } | { name: 'game'; id: string } | { name: 'settings' };

  let view = $state<View>(viewFromAddress());
  let selectedGame = $state<GameDetailData | null>(null);
  let isAddingSwitchGame = $state(false);

  onMount(() => {
    appState.start();
    events.onNavigate((target) => {
      if (target === 'settings') view = { name: 'settings' };
    });
  });

  /** La fiche ouverte est rechargée à chaque modification de la bibliothèque. */
  $effect(() => {
    if (view.name !== 'game') return;
    const id = view.id;
    void appState.revision;
    api.gameDetail(id).then((detail) => (selectedGame = detail));
  });

  const openLibrary = () => (view = { name: 'library' });

  /** `?view=settings` ou `?view=game&id=…` ouvre directement un écran, pratique pour le mode démo. */
  function viewFromAddress(): View {
    const parameters = new URLSearchParams(location.search);
    const id = parameters.get('id');
    switch (parameters.get('view')) {
      case 'settings':
        return { name: 'settings' };
      case 'game':
        return id ? { name: 'game', id } : { name: 'library' };
      default:
        return { name: 'library' };
    }
  }
</script>

<main class:mac={navigator.userAgent.includes('Mac')}>
  {#if view.name === 'game' && selectedGame?.id === view.id}
    <GameDetail game={selectedGame} onBack={openLibrary} />
  {:else if view.name === 'settings'}
    <Settings onBack={openLibrary} />
  {:else if appState.overview}
    <Library
      overview={appState.overview}
      onSelectGame={(id) => (view = { name: 'game', id })}
      onAddSwitchGame={() => (isAddingSwitchGame = true)}
      onOpenSettings={() => (view = { name: 'settings' })}
    />
  {/if}
</main>

{#if isAddingSwitchGame}
  <SwitchPicker onClose={() => (isAddingSwitchGame = false)} />
{/if}

<style>
  main {
    height: 100vh;
    --title-bar-inset: 26px;
  }


  /** Sur macOS, le contenu passe sous la barre de titre : les boutons de fenêtre occupent le coin. */
  main.mac {
    --title-bar-inset: 40px;
  }
</style>
