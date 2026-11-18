<script lang="ts">
  import { api } from '../../lib/api';
  import { t } from '../../lib/i18n/index.svelte';
  import type { SystemInfo } from '../../lib/types';
  import Section from './Section.svelte';

  let { system = $bindable() }: { system: SystemInfo } = $props();

  async function toggleLaunchAtLogin(enabled: boolean) {
    await api.setLaunchAtLogin(enabled);
    system = await api.systemInfo();
  }
</script>

<Section title={t.settings.system}>
  <label class="toggle">
    <input type="checkbox" checked={system.launchAtLogin} onchange={(event) => toggleLaunchAtLogin(event.currentTarget.checked)} />
    {t.settings.launchAtLogin}
  </label>
  <div class="value"><span>{t.settings.dataDirectory}</span><span class="secondary selectable">{system.dataDirectory}</span></div>
</Section>

<style>
  .selectable {
    user-select: text;
  }
</style>
