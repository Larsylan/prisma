<script lang="ts">
  import Crystal from '../../components/Crystal.svelte';
  import GameCover from '../../components/GameCover.svelte';
  import PlatformTag from '../../components/PlatformTag.svelte';
  import ProgressBar from '../../components/ProgressBar.svelte';
  import TierCounts from '../../components/TierCounts.svelte';
  import { percent } from '../../lib/format';
  import type { GameSummary } from '../../lib/types';

  let { game, onselect }: { game: GameSummary; onselect: () => void } = $props();
</script>

<button class="card" onclick={onselect}>
  <GameCover title={game.title} coverUrl={game.coverUrl} platform={game.platform} width={game.platform === 'steam' ? 150 : 92} height={70} />
  <div class="body">
    <div class="title">
      <span>{game.title}</span>
      {#if game.isPlatinumEarned}<Crystal tier="platinum" size={12} />{/if}
    </div>
    <div class="row">
      <PlatformTag label={game.systemLabel} platforms={game.platforms} />
      <TierCounts counts={game.tierCounts.map(({ tier, earned }) => ({ tier, text: String(earned) }))} size={11} />
    </div>
    <div class="row">
      <ProgressBar value={game.progress} height={5} />
      <span class="percent">{percent(game.progress)} %</span>
    </div>
  </div>
</button>

<style>
  .card {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 12px;
    border-radius: 12px;
    background: var(--card);
    border: 1px solid var(--stroke);
    text-align: left;
    transition:
      transform 0.12s ease-out,
      background 0.12s ease-out,
      border-color 0.12s ease-out;
  }

  .card:hover {
    background: var(--card-hover);
    border-color: rgb(var(--ink) / 0.35);
    transform: scale(1.01);
  }

  .body {
    display: flex;
    flex-direction: column;
    gap: 7px;
    flex: 1;
    min-width: 0;
  }

  .title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 14px;
    font-weight: 600;
  }

  .title span {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .percent {
    min-width: 38px;
    white-space: nowrap;
    text-align: right;
    font-size: 11px;
    font-weight: 600;
    color: rgb(var(--ink) / 0.8);
  }
</style>
