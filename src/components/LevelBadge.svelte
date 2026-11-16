<script lang="ts">
  import { tierGradient } from '../lib/colors';
  import type { TrophyTier } from '../lib/types';
  import Crystal from './Crystal.svelte';

  let { level, progress, tier, size = 88 }: { level: number; progress: number; tier: TrophyTier; size?: number } = $props();

  const STROKE = 5;
  let radius = $derived(size / 2 - STROKE / 2);
  let circumference = $derived(2 * Math.PI * radius);
  let gradient = $derived(tierGradient(tier));
</script>

<div class="badge" style:width="{size}px" style:height="{size}px">
  <svg width={size} height={size} viewBox="0 0 {size} {size}" aria-hidden="true">
    <defs>
      <linearGradient id="level-ring" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color={gradient[0]} />
        <stop offset="1" stop-color={gradient[1]} />
      </linearGradient>
    </defs>
    <circle cx={size / 2} cy={size / 2} r={radius} fill="none" style:stroke="rgb(var(--ink) / 0.1)" stroke-width={STROKE} />
    <circle
      cx={size / 2}
      cy={size / 2}
      r={radius}
      fill="none"
      stroke="url(#level-ring)"
      stroke-width={STROKE}
      stroke-linecap="round"
      stroke-dasharray="{circumference * progress} {circumference}"
      transform="rotate(-90 {size / 2} {size / 2})"
    />
  </svg>
  <div class="content">
    <Crystal {tier} size={size * 0.24} />
    <span style:font-size="{size * 0.27}px">{level}</span>
  </div>
</div>

<style>
  .badge {
    position: relative;
    flex-shrink: 0;
  }

  .content {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
  }

  span {
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
</style>
