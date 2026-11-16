<script lang="ts">
  /** Pictogrammes au trait, dessinés pour l'app. */
  const PATHS = {
    plus: 'M12 5v14M5 12h14',
    sync: 'M4 12a8 8 0 0 1 13.7-5.6L20 9M20 4v5h-5M20 12a8 8 0 0 1-13.7 5.6L4 15M4 20v-5h5',
    gear: 'M12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7ZM19.4 13.5l1.6 1.2-2 3.4-1.9-.7a7.6 7.6 0 0 1-2 1.2l-.3 2h-4l-.3-2a7.6 7.6 0 0 1-2-1.2l-1.9.7-2-3.4 1.6-1.2a7.7 7.7 0 0 1 0-3L3 9.3l2-3.4 1.9.7a7.6 7.6 0 0 1 2-1.2l.3-2h4l.3 2a7.6 7.6 0 0 1 2 1.2l1.9-.7 2 3.4-1.6 1.2a7.7 7.7 0 0 1 0 3Z',
    search: 'M10.5 17a6.5 6.5 0 1 0 0-13 6.5 6.5 0 0 0 0 13ZM15.5 15.5 20 20',
    back: 'M15 5l-7 7 7 7',
    lock: 'M7 11V8a5 5 0 0 1 10 0v3M6 11h12v9H6z',
    check: 'M5 12.5l4.5 4.5L19 7.5',
    controller: 'M7 9v4M5 11h4M15 10h.01M18 12h.01M6.5 6h11a4 4 0 0 1 3.9 4.9l-1.2 5.2a2.5 2.5 0 0 1-4.3 1.1L14 15h-4l-1.9 2.2a2.5 2.5 0 0 1-4.3-1.1l-1.2-5.2A4 4 0 0 1 6.5 6Z',
  } as const;

  let { name }: { name: keyof typeof PATHS } = $props();
</script>

<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
  <path d={PATHS[name]} />
</svg>
