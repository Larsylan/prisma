<script lang="ts">
  import Crystal from '../../components/Crystal.svelte';
  import PillPicker from '../../components/PillPicker.svelte';
  import SearchField from '../../components/SearchField.svelte';
  import { platformLabels } from '../../lib/format';
  import { appState } from '../../lib/state.svelte';
  import type { GameSummary, LibraryOverview, Platform } from '../../lib/types';
  import GameCard from './GameCard.svelte';
  import LibraryHeader from './LibraryHeader.svelte';

  let {
    overview,
    onSelectGame,
    onAddSwitchGame,
    onOpenSettings,
  }: { overview: LibraryOverview; onSelectGame: (id: string) => void; onAddSwitchGame: () => void; onOpenSettings: () => void } = $props();

  type Sort = 'recent' | 'progress' | 'alphabetical';
  const SORT_LABELS: Record<Sort, string> = { recent: 'Derniers obtenus', progress: 'Progression', alphabetical: 'Alphabétique' };
  const PLATFORM_OPTIONS = [
    { value: null, label: 'Tous' },
    ...(Object.keys(platformLabels) as Platform[]).map((platform) => ({ value: platform, label: platformLabels[platform] })),
  ];

  let platform = $state<Platform | null>(null);
  let sort = $state<Sort>('recent');
  let search = $state('');

  let visibleGames = $derived(sorted(overview.games.filter(isVisible)));

  function isVisible(game: GameSummary): boolean {
    const isOnPlatform = platform === null || game.platforms.includes(platform);
    return isOnPlatform && game.title.toLocaleLowerCase('fr').includes(search.trim().toLocaleLowerCase('fr'));
  }

  /** Les jeux arrivent déjà triés par date d'obtention. */
  function sorted(games: GameSummary[]): GameSummary[] {
    switch (sort) {
      case 'recent':
        return games;
      case 'progress':
        return [...games].sort((lhs, rhs) => rhs.progress - lhs.progress);
      case 'alphabetical':
        return [...games].sort((lhs, rhs) => lhs.title.localeCompare(rhs.title, 'fr'));
    }
  }
</script>

<div class="library">
  <LibraryHeader profile={overview.profile} status={appState.status} {onAddSwitchGame} {onOpenSettings} />

  <div class="controls">
    <PillPicker options={PLATFORM_OPTIONS} bind:value={platform} />
    <div class="spacer"></div>
    <div class="search"><SearchField placeholder="Rechercher un jeu" bind:value={search} /></div>
    <select class="select" bind:value={sort} aria-label="Tri">
      {#each Object.entries(SORT_LABELS) as [value, label] (value)}
        <option {value}>{label}</option>
      {/each}
    </select>
  </div>

  {#if visibleGames.length > 0}
    <div class="grid">
      {#each visibleGames as game (game.id)}
        <GameCard {game} onselect={() => onSelectGame(game.id)} />
      {/each}
    </div>
  {:else}
    <div class="empty">
      <Crystal tier="platinum" size={54} />
      {#if overview.games.length === 0}
        <h2>Aucun trophée pour le moment</h2>
        <p class="secondary">Connectez votre compte PlayStation et/ou Steam dans les réglages, ou ajoutez vos jeux Switch.</p>
        <div class="empty-actions">
          <button class="button primary" onclick={onOpenSettings}>Ouvrir les réglages</button>
          <button class="button" onclick={onAddSwitchGame}>Ajouter un jeu Switch</button>
        </div>
      {:else}
        <h2>Aucun jeu ne correspond</h2>
      {/if}
    </div>
  {/if}

  <footer>
    <span class="dot" class:active={appState.status?.steamWatching}></span>
    <span>{appState.status?.steamWatching ? 'Surveillance Steam en temps réel active' : 'Surveillance Steam inactive'}</span>
    {#if appState.status?.steamLocal}<span>· {appState.status.steamLocal}</span>{/if}
    <span class="spacer"></span>
    {#if appState.status?.playstation}<span>{appState.status.playstation}</span>{/if}
  </footer>
</div>

<style>
  .library {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .controls {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 0 var(--gutter) 16px;
  }

  .spacer {
    flex: 1;
  }

  .search {
    width: 212px;
  }

  .grid {
    flex: 1;
    overflow-y: auto;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(380px, 1fr));
    align-content: start;
    gap: 14px;
    padding: 2px var(--gutter) 20px;
  }

  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 14px;
  }

  .empty h2 {
    margin: 0;
    font-size: 20px;
  }

  .empty p {
    margin: 0;
  }

  .empty-actions {
    display: flex;
    gap: 8px;
  }

  footer {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px var(--gutter);
    font-size: 11px;
    color: var(--secondary);
    background: var(--sunken);
    white-space: nowrap;
    overflow: hidden;
  }

  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: orange;
    flex-shrink: 0;
  }

  .dot.active {
    background: #34c759;
  }
</style>
