<script lang="ts">
  import { onMount } from 'svelte';

  import Crystal from './components/Crystal.svelte';
  import { events } from './lib/api';
  import { api } from './lib/api';
  import { tierGradient } from './lib/colors';
  import { applyPalette } from './lib/palette';
  import type { Announcement } from './lib/types';

  let announcement = $state<Announcement | null>(null);
  let isVisible = $state(false);
  let iconFailed = $state(false);
  let border = $derived(announcement ? tierGradient(announcement.tier) : null);

  onMount(() => {
    api.activeTheme().then(applyPalette);
    events.onThemeChanged(applyPalette);
    events.onToastShow((next) => {
      announcement = next;
      iconFailed = false;
      isVisible = true;
    });
    events.onToastHide(() => (isVisible = false));
  });
</script>

{#if announcement}
  <div
    class="toast"
    class:visible={isVisible}
    class:from-left={announcement.slidesFromLeft}
    style:--border-start={border?.[0]}
  >

    <div class="icon">
      {#if announcement.iconUrl && !iconFailed}
        <img src={announcement.iconUrl} alt="" onerror={() => (iconFailed = true)} />
      {:else}
        <Crystal tier={announcement.tier} size={30} />
      {/if}
    </div>
    <div class="text">
      <div class="headline">
        <Crystal tier={announcement.tier} size={13} />
        {announcement.tier === 'platinum' ? 'Vous avez obtenu un trophée de platine !' : 'Vous avez obtenu un trophée !'}
      </div>
      <div class="name">{announcement.trophyName}</div>
      <div class="game">{announcement.gameTitle}</div>
    </div>
  </div>
{/if}

<style>
  .toast {
    position: absolute;
    inset: 10px;
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 0 16px;
    border-radius: 14px;
    /* Le fond de la carte reprend discrètement les couleurs du thème actif. */
    background: linear-gradient(
      180deg,
      color-mix(in srgb, color-mix(in srgb, var(--background-top) 45%, var(--toast-tint)) 96%, transparent),
      color-mix(in srgb, color-mix(in srgb, var(--background-bottom) 45%, var(--toast-tint)) 96%, transparent)
    );
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--border-start) 55%, transparent);
    opacity: 0;
    transform: translateX(40px);
    transition:
      opacity 0.35s ease-out,
      transform 0.28s ease-out;
  }

  .toast.from-left {
    transform: translateX(-40px);
  }

  .toast.visible {
    opacity: 1;
    transform: none;
  }

  .icon {
    width: 64px;
    height: 64px;
    border-radius: 8px;
    overflow: hidden;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgb(var(--ink) / 0.08);
  }

  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .text {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .headline {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    font-weight: 500;
    color: rgb(var(--ink) / 0.7);
  }

  .name,
  .game {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .name {
    font-size: 16px;
    font-weight: 600;
  }

  .game {
    font-size: 12px;
    color: rgb(var(--ink) / 0.55);
  }
</style>
