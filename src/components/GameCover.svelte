<script lang="ts">
  import { coverGradients } from '../lib/colors';
  import type { Platform } from '../lib/types';
  import Crystal from './Crystal.svelte';

  /**
   * Jaquette d'un jeu ; un jeu sans image, comme tous les jeux Switch, reçoit une jaquette générée
   * à partir de son titre et du cristal, sans visuel de marque.
   */
  let { title, coverUrl, platform, width, height }: { title: string; coverUrl: string | null; platform: Platform; width: number; height: number } =
    $props();

  /** En dessous, le titre ne tiendrait pas : la vignette garde seulement le dégradé et le cristal. */
  const MINIMUM_TITLE_HEIGHT = 40;

  let failed = $state(false);
  let gradient = $derived(coverGradients[platform]);
</script>

<div class="cover" style:width="{width}px" style:height="{height}px">
  {#if coverUrl && !failed}
    <img src={coverUrl} alt="" loading="lazy" onerror={() => (failed = true)} />
  {:else}
    <div class="generated" style:background="linear-gradient(135deg, {gradient[0]}, {gradient[1]})" style:padding="{height * 0.09}px">
      <div class="emblem" style:top="{height * 0.08}px" style:right="{height * 0.08}px">
        <Crystal tier="platinum" size={height * 0.5} />
      </div>
      {#if height >= MINIMUM_TITLE_HEIGHT}
        <span style:font-size="{Math.max(9, height * 0.13)}px">{title}</span>
      {/if}
    </div>
  {/if}
</div>

<style>
  .cover {
    flex-shrink: 0;
    border-radius: 8px;
    overflow: hidden;
    background: rgba(255, 255, 255, 0.05);
  }

  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .generated {
    position: relative;
    width: 100%;
    height: 100%;
    display: flex;
    align-items: flex-end;
  }

  .emblem {
    position: absolute;
    opacity: 0.22;
  }

  span {
    position: relative;
    font-weight: 800;
    line-height: 1.15;
    color: rgba(255, 255, 255, 0.92);
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
