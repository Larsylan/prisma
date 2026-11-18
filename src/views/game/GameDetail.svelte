<script lang="ts">
  import GameCover from '../../components/GameCover.svelte';
  import Icon from '../../components/Icon.svelte';
  import PillPicker from '../../components/PillPicker.svelte';
  import PlatformTag from '../../components/PlatformTag.svelte';
  import ProgressBar from '../../components/ProgressBar.svelte';
  import TierCounts from '../../components/TierCounts.svelte';
  import ConfirmDialog from '../../components/ConfirmDialog.svelte';
  import { api } from '../../lib/api';
  import { percent } from '../../lib/format';
  import { t } from '../../lib/i18n/index.svelte';
  import type { GameDetail, TrophyTier, TrophyView } from '../../lib/types';
  import TrophyRow from './TrophyRow.svelte';

  let { game, onBack }: { game: GameDetail; onBack: () => void } = $props();

  type Filter = 'all' | 'earned' | 'locked';
  type Sort = keyof typeof t.game.sorts;
  const TIER_RANK: Record<TrophyTier, number> = { bronze: 0, silver: 1, gold: 2, platinum: 3 };

  let filter = $state<Filter>('all');
  let sort = $state<Sort>('original');
  let isConfirmingRemoval = $state(false);

  let filterOptions = $derived<{ value: Filter; label: string }[]>([
    { value: 'all', label: t.all },
    { value: 'earned', label: t.game.filters.earned },
    { value: 'locked', label: t.game.filters.locked },
  ]);
  let trophies = $derived(sorted(game.trophies.filter(matchesFilter)));
  let summary = $derived(
    [
      t.game.earnedSummary(game.earnedCount, game.trophyCount),
      game.isGroup ? t.game.mergedVersions(game.systems.join(', ')) : null,
      game.platforms.includes('nintendo') ? t.game.switchHint : null,
    ]
      .filter(Boolean)
      .join(' · '),
  );

  function matchesFilter(trophy: TrophyView): boolean {
    return filter === 'all' || (filter === 'earned') === trophy.isEarned;
  }

  function sorted(list: TrophyView[]): TrophyView[] {
    switch (sort) {
      case 'original':
        return list;
      case 'earnedDate':
        return [...list].sort((lhs, rhs) => (rhs.earnedAt ?? '').localeCompare(lhs.earnedAt ?? ''));
      case 'tier':
        return [...list].sort((lhs, rhs) => TIER_RANK[rhs.tier] - TIER_RANK[lhs.tier]);
    }
  }

  async function remove() {
    await api.removeGame(game.id);
    onBack();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && !event.defaultPrevented && !isConfirmingRemoval) onBack();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="detail">
  <div class="toolbar" data-tauri-drag-region>
    <button class="back" onclick={onBack}><Icon name="back" /> {t.backToTrophies}</button>
    {#if game.isRemovable}
      <button class="remove secondary" onclick={() => (isConfirmingRemoval = true)}>{t.game.remove}</button>
    {/if}
  </div>

  <div class="scroll">
    <section class="banner">
      <div class="cover">
        <GameCover title={game.title} coverUrl={game.coverUrl} platform={game.platform} width={game.platform === 'steam' ? 300 : 170} height={140} />
      </div>
      <div class="info">
        <PlatformTag label={game.systemLabel} platforms={game.platforms} />
        <h1>{game.title}</h1>
        <div class="progress">
          <div class="bar"><ProgressBar value={game.progress} height={7} /></div>
          <strong>{t.percent(percent(game.progress))}</strong>
        </div>
        <TierCounts counts={game.tierCounts.map(({ tier, earned, total }) => ({ tier, text: `${earned}/${total}` }))} size={15} />
        <span class="secondary summary">{summary}</span>
      </div>
    </section>

    <div class="controls">
      <PillPicker options={filterOptions} bind:value={filter} />
      <select class="select" bind:value={sort} aria-label={t.sortBy}>
        {#each Object.entries(t.game.sorts) as [value, label] (value)}
          <option {value}>{label}</option>
        {/each}
      </select>
    </div>

    <div class="trophies" role="list">
      {#each trophies as trophy (trophy.id)}
        <TrophyRow {trophy} gameTitle={game.title} isGrayscaleWhenLocked={game.platform === 'playstation'} />
      {/each}
    </div>
  </div>
</div>

{#if isConfirmingRemoval}
  <ConfirmDialog
    title={t.game.removeTitle(game.title)}
    message={t.game.removeMessage}
    confirmLabel={t.game.removeConfirm}
    onConfirm={remove}
    onCancel={() => (isConfirmingRemoval = false)}
  />
{/if}

<style>
  .detail {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--title-bar-inset) var(--gutter) 0;
  }

  .back {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 14px;
    font-weight: 600;
  }

  .back :global(svg) {
    width: 14px;
    height: 14px;
  }

  .remove {
    font-size: 12px;
    font-weight: 600;
  }

  .scroll {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 18px;
    padding: 16px var(--gutter);
  }

  .banner {
    display: flex;
    align-items: center;
    gap: 22px;
  }

  .cover {
    border-radius: 12px;
    box-shadow: 0 6px 14px rgba(0, 0, 0, 0.5);
  }

  .cover :global(.cover) {
    border-radius: 12px;
  }

  .info {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }

  h1 {
    margin: 0;
    font-size: 28px;
  }

  .progress {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 15px;
  }

  .bar {
    display: flex;
    width: 260px;
  }

  .summary {
    font-size: 12px;
  }

  .controls {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .trophies {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
</style>
