<script lang="ts">
  import type { TrophyTier } from '../lib/types';
  import Crystal from './Crystal.svelte';

  let { counts, size = 14 }: { counts: { tier: TrophyTier; text: string }[]; size?: number } = $props();
</script>

<div class="counts" style:gap="{size * 0.9}px">
  {#each counts as count (count.tier)}
    <span class="count">
      <Crystal tier={count.tier} {size} />
      <span style:font-size="{size}px">{count.text}</span>
    </span>
  {/each}
</div>

<style>
  .counts {
    display: flex;
    align-items: center;
  }

  .count {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    color: rgb(var(--ink) / 0.9);
  }
</style>
