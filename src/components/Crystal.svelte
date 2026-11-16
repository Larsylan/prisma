<script lang="ts">
  import { mix, tierGradient, tierPalettes } from '../lib/colors';
  import type { TrophyTier } from '../lib/types';

  /**
   * Emblème des trophées : un cristal hexagonal à six facettes, éclairé depuis le haut à gauche.
   */
  let { tier, size = 18, dimmed = false }: { tier: TrophyTier; size?: number; dimmed?: boolean } = $props();

  const WIDTH = 92;
  const HEIGHT = 110;
  /** Contour en coordonnées unitaires. */
  const OUTLINE: [number, number][] = [
    [0.5, 0.0],
    [0.95, 0.3],
    [0.82, 0.8],
    [0.5, 1.0],
    [0.18, 0.8],
    [0.05, 0.3],
  ];
  const CORE: [number, number] = [0.5, 0.42];
  /** Ombrage de chaque facette, de 0 (teinte claire) à 1 (teinte sombre). */
  const SHADES = [0.3, 0.62, 0.92, 0.72, 0.38, 0.0];
  /** Reflets irisés des facettes du platine. */
  const PLATINUM_TINTS: (number | null)[] = [0xd4ecff, null, 0xb9a9e8, null, 0xe6d6ff, null];

  const point = ([x, y]: [number, number]) => `${x * WIDTH},${y * HEIGHT}`;
  const facets = OUTLINE.map((vertex, index) => [CORE, vertex, OUTLINE[(index + 1) % OUTLINE.length]].map(point).join(' '));

  let colors = $derived(
    SHADES.map((shade, index) => {
      const { light, dark } = tierPalettes[tier];
      const tint = tier === 'platinum' ? PLATINUM_TINTS[index] : null;
      return tint === null ? mix(light, dark, shade) : mix(tint, dark, shade * 0.7);
    }),
  );
  let edgeColor = $derived(mix(tierPalettes[tier].dark, 0x000000, 0.35));
  let filter = $derived(dimmed ? 'saturate(0.2)' : `drop-shadow(0 0 ${size / 6}px ${tierGradient(tier)[1]})`);
</script>

<svg
  class:dimmed
  width={size * 0.92}
  height={size * 1.1}
  viewBox="-2 -2 {WIDTH + 4} {HEIGHT + 4}"
  style:filter={filter}
  aria-hidden="true"
>
  {#each facets as facet, index (index)}
    <polygon points={facet} fill={colors[index]} />
  {/each}
  {#each OUTLINE as vertex, index (index)}
    <line x1={CORE[0] * WIDTH} y1={CORE[1] * HEIGHT} x2={vertex[0] * WIDTH} y2={vertex[1] * HEIGHT} stroke="rgba(255,255,255,0.45)" stroke-width="2" />
  {/each}
  <polygon points={OUTLINE.map(point).join(' ')} fill="none" stroke={edgeColor} stroke-width="3.2" stroke-linejoin="round" />
  {#if tier === 'platinum'}
    <path d="M27.6 6.2Q27.6 26.4 47.8 26.4Q27.6 26.4 27.6 46.6Q27.6 26.4 7.4 26.4Q27.6 26.4 27.6 6.2Z" fill="rgba(255,255,255,0.95)" />
  {/if}
</svg>

<style>
  svg {
    flex-shrink: 0;
    overflow: visible;
  }

  .dimmed {
    opacity: 0.4;
  }
</style>
