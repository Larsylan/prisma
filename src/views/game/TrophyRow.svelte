<script lang="ts">
  import { CheckMenuItem, Menu, MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu';

  import Crystal from '../../components/Crystal.svelte';
  import Icon from '../../components/Icon.svelte';
  import { api } from '../../lib/api';
  import { earnedDate, tierLabels } from '../../lib/format';
  import type { TrophyTier, TrophyView } from '../../lib/types';

  let { trophy, isGrayscaleWhenLocked }: { trophy: TrophyView; isGrayscaleWhenLocked: boolean } = $props();

  const REGULAR_TIERS: TrophyTier[] = ['bronze', 'silver', 'gold'];

  let isRevealed = $state(false);
  let iconFailed = $state(false);
  let isConcealed = $derived(trophy.isHidden && !trophy.isEarned && !isRevealed);
  let toggleLabel = $derived(trophy.isEarned ? "Retirer l'obtention" : 'Marquer comme obtenu');
  let earnedText = $derived.by(() => {
    const date = trophy.earnedAt ? `Obtenu le ${earnedDate(trophy.earnedAt)}` : 'Obtenu';
    return trophy.earnedOn.length > 0 ? `${date} · ${trophy.earnedOn.join(', ')}` : date;
  });

  function toggleEarned() {
    api.setTrophyEarned(trophy.reference, !trophy.isEarned);
  }

  async function showContextMenu(event: MouseEvent) {
    if (!trophy.canToggleEarned && !trophy.canChangeTier) return;
    event.preventDefault();
    const items = [];
    if (trophy.canToggleEarned) {
      items.push(await MenuItem.new({ text: toggleLabel, action: toggleEarned }));
    }
    if (trophy.canChangeTier) {
      items.push(await MenuItem.new({ text: 'Grade du trophée', enabled: false }));
      for (const tier of REGULAR_TIERS) {
        items.push(
          await CheckMenuItem.new({
            text: tierLabels[tier],
            checked: trophy.tier === tier,
            action: () => api.setTrophyTier(trophy.reference, tier),
          }),
        );
      }
      if (trophy.hasTierOverride) {
        items.push(await PredefinedMenuItem.new({ item: 'Separator' }));
        items.push(await MenuItem.new({ text: 'Rétablir le grade automatique', action: () => api.setTrophyTier(trophy.reference, null) }));
      }
    }
    await (await Menu.new({ items })).popup();
  }
</script>

<div class="row" oncontextmenu={showContextMenu} role="listitem">
  <div class="artwork">
    {#if isConcealed}
      <Icon name="lock" />
    {:else if trophy.iconUrl && !iconFailed}
      <img src={trophy.iconUrl} alt="" loading="lazy" class:grayscale={!trophy.isEarned && isGrayscaleWhenLocked} onerror={() => (iconFailed = true)} />
    {:else}
      <Crystal tier={trophy.tier} size={32} dimmed={!trophy.isEarned} />
    {/if}
  </div>

  <div class="text">
    <span class="name" class:locked={!trophy.isEarned}>{isConcealed ? 'Trophée caché' : trophy.name}</span>
    <span class="detail secondary">{isConcealed ? 'Continuez à jouer pour découvrir ce trophée.' : trophy.detail}</span>
    {#if trophy.isEarned}<span class="earned">{earnedText}</span>{/if}
  </div>

  {#if trophy.isHidden && !trophy.isEarned}
    <button class="reveal secondary" onclick={() => (isRevealed = !isRevealed)}>{isRevealed ? 'Masquer' : 'Afficher'}</button>
  {/if}

  <div class="tier">
    <Crystal tier={trophy.tier} size={22} dimmed={!trophy.isEarned} />
    <span class="secondary">{tierLabels[trophy.tier]}</span>
  </div>

  {#if trophy.canToggleEarned}
    <button class="check" class:checked={trophy.isEarned} title={toggleLabel} aria-label={toggleLabel} onclick={toggleEarned}>
      {#if trophy.isEarned}<Icon name="check" />{/if}
    </button>
  {/if}
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 10px;
    border-radius: 10px;
    background: var(--card);
  }

  .row:hover {
    background: var(--card-hover);
  }

  .artwork {
    width: 60px;
    height: 60px;
    flex-shrink: 0;
    border-radius: 8px;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--placeholder);
    color: rgb(var(--ink) / 0.4);
  }

  .artwork :global(svg) {
    width: 22px;
  }

  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  img.grayscale {
    filter: grayscale(1);
    opacity: 0.55;
  }

  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .name {
    font-size: 15px;
    font-weight: 600;
  }

  .name.locked {
    color: rgb(var(--ink) / 0.75);
  }

  .detail {
    font-size: 12px;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .earned {
    font-size: 11px;
    font-weight: 500;
    color: var(--earned);
    text-shadow: var(--text-glow);
  }

  .reveal {
    font-size: 11px;
    font-weight: 600;
  }

  .tier {
    width: 54px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 3px;
    font-size: 10px;
    font-weight: 500;
  }

  .check {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    border: 2px solid rgb(var(--ink) / 0.35);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .check.checked {
    border-color: var(--earned);
    background: var(--earned);
    color: var(--on-highlight);
    box-shadow: var(--glow);
  }

  .check :global(svg) {
    width: 14px;
    height: 14px;
    stroke-width: 3;
  }
</style>
