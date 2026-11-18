<script lang="ts">
  import Crystal from '../../components/Crystal.svelte';
  import PillPicker from '../../components/PillPicker.svelte';
  import SearchField from '../../components/SearchField.svelte';
  import { compareTitles, matchesSearch } from '../../lib/format';
  import { t } from '../../lib/i18n/index.svelte';
  import { appState } from '../../lib/state.svelte';
  import { playstationText, steamLocalText } from '../../lib/status';
  import type { GameSummary, LibraryOverview, Platform } from '../../lib/types';
  import GameCard from './GameCard.svelte';
  import LibraryHeader from './LibraryHeader.svelte';

  let {
    overview,
    onSelectGame,
    onAddSwitchGame,
    onOpenSettings,
  }: { overview: LibraryOverview; onSelectGame: (id: string) => void; onAddSwitchGame: () => void; onOpenSettings: () => void } = $props();

  type Sort = keyof typeof t.library.sorts;
  const PLATFORMS: Platform[] = ['playstation', 'steam', 'nintendo'];

  let platformOptions = $derived([
    { value: null, label: t.all },
    ...PLATFORMS.map((platform) => ({ value: platform, label: t.platforms[platform] })),
  ]);

  let platform = $state<Platform | null>(null);
  let sort = $state<Sort>('recent');
  let search = $state('');

  let visibleGames = $derived(sorted(overview.games.filter(isVisible)));
  let steamLocal = $derived(appState.status ? steamLocalText(appState.status.steamLocal) : '');
  let playstation = $derived(appState.status ? playstationText(appState.status.playstation) : '');

  function isVisible(game: GameSummary): boolean {
    const isOnPlatform = platform === null || game.platforms.includes(platform);
    return isOnPlatform && matchesSearch(game.title, search);
  }

  /** Les jeux arrivent déjà triés par date d'obtention. */
  function sorted(games: GameSummary[]): GameSummary[] {
    switch (sort) {
      case 'recent':
        return games;
      case 'progress':
        return [...games].sort((lhs, rhs) => rhs.progress - lhs.progress);
      case 'alphabetical':
        return [...games].sort((lhs, rhs) => compareTitles(lhs.title, rhs.title));
    }
  }
</script>

<div class="library">
  <LibraryHeader profile={overview.profile} status={appState.status} {onAddSwitchGame} {onOpenSettings} />

  <div class="controls">
    <PillPicker options={platformOptions} bind:value={platform} />
    <div class="spacer"></div>
    <div class="search"><SearchField placeholder={t.library.search} bind:value={search} /></div>
    <select class="select" bind:value={sort} aria-label={t.sortBy}>
      {#each Object.entries(t.library.sorts) as [value, label] (value)}
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
        <h2>{t.library.emptyTitle}</h2>
        <p class="secondary">{t.library.emptyHint}</p>
        <div class="empty-actions">
          <button class="button primary" onclick={onOpenSettings}>{t.library.openSettings}</button>
          <button class="button" onclick={onAddSwitchGame}>{t.library.addSwitchGame}</button>
        </div>
      {:else}
        <h2>{t.library.noMatch}</h2>
      {/if}
    </div>
  {/if}

  <footer>
    <span class="dot" class:active={appState.status?.steamWatching}></span>
    <span>{appState.status?.steamWatching ? t.library.steamWatching : t.library.steamNotWatching}</span>
    {#if steamLocal}<span>· {steamLocal}</span>{/if}
    <span class="spacer"></span>
    {#if playstation}<span>{playstation}</span>{/if}
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
