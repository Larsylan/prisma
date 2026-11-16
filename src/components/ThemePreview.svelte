<script lang="ts">
  import { paletteStyle } from '../lib/palette';
  import type { ThemePalette } from '../lib/types';

  /**
   * Miniature de l'écran principal aux couleurs d'un thème. Toutes les dimensions suivent
   * la largeur de l'aperçu : il garde ses proportions quelle que soit sa taille.
   */
  let { palette }: { palette: ThemePalette } = $props();

  const CARD_PROGRESS = [0.85, 0.4, 0.6, 0.25];
</script>

<div class="frame" aria-hidden="true">
  <div class="preview" style={paletteStyle(palette)}>
    <div class="screen">
      <div class="header">
        <span class="badge"></span>
        <span class="lines">
          <span class="line strong"></span>
          <span class="line"></span>
          <span class="bar"><span class="fill" style:width="45%"></span></span>
        </span>
      </div>
      <div class="cards">
        {#each CARD_PROGRESS as progress, index (index)}
          <span class="card">
            <span class="cover"></span>
            <span class="lines">
              <span class="line strong"></span>
              <span class="bar"><span class="fill" style:width="{progress * 100}%"></span></span>
            </span>
          </span>
        {/each}
      </div>
    </div>
  </div>
</div>

<style>
  .frame {
    container-type: inline-size;
  }

  .preview {
    position: relative;
    aspect-ratio: 16 / 10;
    border-radius: 8px;
    overflow: hidden;
    background: var(--background);
    border: 1px solid rgb(var(--ink) / 0.08);
  }


  .screen {
    position: absolute;
    inset: 0;
    padding: 7cqw;
    display: flex;
    flex-direction: column;
    gap: 6cqw;
  }


  .header {
    display: flex;
    align-items: center;
    gap: 4cqw;
  }

  .badge {
    width: 11cqw;
    height: 11cqw;
    flex-shrink: 0;
    border-radius: 50%;
    border: 1.3cqw solid var(--accent);
  }

  .lines {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 1.8cqw;
    min-width: 0;
  }

  .line,
  .bar {
    height: 1.8cqw;
    border-radius: 1cqw;
  }

  .line {
    width: 50%;
    background: rgb(var(--ink) / 0.35);
  }

  .line.strong {
    width: 70%;
    background: rgb(var(--ink) / 0.85);
  }

  .bar {
    background: rgb(var(--ink) / 0.12);
    overflow: hidden;
  }

  .fill {
    display: block;
    height: 100%;
    background: var(--progress);
    box-shadow: var(--glow);
  }

  .cards {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-auto-rows: 1fr;
    gap: 3cqw;
  }

  .card {
    display: flex;
    align-items: center;
    gap: 2.5cqw;
    min-height: 0;
    padding: 2.5cqw;
    border-radius: 2.5cqw;
    background: rgb(var(--ink) / 0.055);
  }

  .cover {
    width: 13cqw;
    height: 10cqw;
    flex-shrink: 0;
    border-radius: 1.2cqw;
    background: rgb(var(--ink) / 0.12);
  }
</style>
