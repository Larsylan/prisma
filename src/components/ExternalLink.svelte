<script lang="ts">
  import { openUrl } from '@tauri-apps/plugin-opener';
  import type { Snippet } from 'svelte';

  /** Ouvre l'adresse dans le navigateur de l'utilisateur plutôt que dans l'app. */
  let { href, children }: { href: string; children: Snippet } = $props();

  function open(event: MouseEvent) {
    event.preventDefault();
    openUrl(href);
  }
</script>

<a {href} onclick={open}>{@render children()}</a>
