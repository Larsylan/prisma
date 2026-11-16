<script lang="ts">
  import { onMount } from 'svelte';

  import Icon from '../../components/Icon.svelte';
  import { api } from '../../lib/api';
  import type { Settings, SystemInfo } from '../../lib/types';
  import AppearanceSection from './AppearanceSection.svelte';
  import GroupingSection from './GroupingSection.svelte';
  import NotificationsSection from './NotificationsSection.svelte';
  import PlayStationSection from './PlayStationSection.svelte';
  import ProfileSection from './ProfileSection.svelte';
  import { debounced } from './saving';
  import SteamSection from './SteamSection.svelte';
  import SystemSection from './SystemSection.svelte';

  let { onBack }: { onBack: () => void } = $props();

  let settings = $state<Settings | null>(null);
  let system = $state<SystemInfo | null>(null);

  const save = debounced(() => settings && api.updateSettings($state.snapshot(settings)));

  onMount(async () => {
    [settings, system] = await Promise.all([api.settings(), api.systemInfo()]);
  });

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') onBack();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="settings">
  <div class="toolbar" data-tauri-drag-region>
    <button class="back" onclick={onBack}><Icon name="back" /> Trophées</button>
    <h1 data-tauri-drag-region>Réglages</h1>
  </div>

  {#if settings && system}
    <div class="scroll">
      <ProfileSection bind:name={settings.profileName} {save} />
      <AppearanceSection bind:theme={settings.theme} onChange={() => save()} />
      <SteamSection
        bind:steamId64={settings.steamId64}
        bind:useRarity={settings.useSteamRarity}
        detectedSteamId64={system.detectedSteamId64}
        {save}
      />
      <PlayStationSection bind:polling={settings.playstationPolling} onPollingChange={() => save()} />
      <GroupingSection bind:mode={settings.groupingMode} {save} />
      <NotificationsSection
        bind:soundEnabled={settings.soundEnabled}
        bind:position={settings.toastPosition}
        bind:durationSeconds={settings.toastDurationSeconds}
        {save}
      />
      <SystemSection bind:system />
    </div>
  {/if}
</div>

<style>
  .settings {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 20px;
    padding: var(--title-bar-inset) var(--gutter) 12px;
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

  h1 {
    margin: 0;
    font-size: 22px;
  }

  .scroll {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 18px;
    padding: 8px var(--gutter) 28px;
    max-width: 720px;
    width: 100%;
    margin: 0 auto;
  }
</style>
