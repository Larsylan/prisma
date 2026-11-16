<script lang="ts">
  import Icon from '../../components/Icon.svelte';
  import IconButton from '../../components/IconButton.svelte';
  import LevelBadge from '../../components/LevelBadge.svelte';
  import ProgressBar from '../../components/ProgressBar.svelte';
  import TierCounts from '../../components/TierCounts.svelte';
  import { api } from '../../lib/api';
  import { tierGradient } from '../../lib/colors';
  import { formatNumber, percent } from '../../lib/format';
  import type { Profile, SyncStatus } from '../../lib/types';

  let {
    profile,
    status,
    onAddSwitchGame,
    onOpenSettings,
  }: { profile: Profile; status: SyncStatus | null; onAddSwitchGame: () => void; onOpenSettings: () => void } = $props();

  let isSyncing = $derived(Boolean(status?.syncing || status?.steamWebImporting));
</script>

<header data-tauri-drag-region>
  <LevelBadge level={profile.level} progress={profile.levelProgress} tier={profile.levelTier} />
  <div class="identity" data-tauri-drag-region>
    <h1>{profile.name}</h1>
    <span class="secondary level">Niveau de trophée {profile.level}</span>
    <div class="level-progress"><ProgressBar value={profile.levelProgress} colors={tierGradient(profile.levelTier)} /></div>
    <span class="secondary small">
      {percent(profile.levelProgress)} % · niveau {profile.nextLevel} · {formatNumber(profile.points)} points
    </span>
  </div>
  <div class="totals" data-tauri-drag-region>
    <span class="secondary">{formatNumber(profile.earnedTotal)} trophées</span>
    <TierCounts counts={profile.earnedByTier.map(({ tier, count }) => ({ tier, text: formatNumber(count) }))} size={18} />
  </div>
  <div class="actions">
    <IconButton help="Ajouter un jeu Switch" onclick={onAddSwitchGame}><Icon name="plus" /></IconButton>
    <IconButton help="Synchroniser" spinning={isSyncing} onclick={() => api.syncAll()}><Icon name="sync" /></IconButton>
    <IconButton help="Réglages" onclick={onOpenSettings}><Icon name="gear" /></IconButton>
  </div>
</header>

<style>
  header {
    display: flex;
    align-items: center;
    gap: 22px;
    padding: var(--title-bar-inset) var(--gutter) 18px;
  }

  .identity {
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex: 1;
  }

  h1 {
    margin: 0;
    font-size: 24px;
  }

  .level {
    font-weight: 500;
  }

  .level-progress {
    display: flex;
    width: 240px;
  }

  .small {
    font-size: 11px;
  }

  .totals {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 10px;
    font-weight: 500;
  }

  .actions {
    display: flex;
    gap: 8px;
  }
</style>
