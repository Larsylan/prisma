<script lang="ts">
  import { platformColors } from '../lib/colors';
  import type { Platform } from '../lib/types';

  let { label, platforms }: { label: string; platforms: Platform[] } = $props();

  /** Un dégradé réunit les couleurs de chaque plateforme d'un jeu regroupé. */
  let background = $derived.by(() => {
    const colors = platforms.map((platform) => platformColors[platform]);
    return `linear-gradient(90deg, ${(colors.length === 1 ? [...colors, ...colors] : colors).join(', ')})`;
  });
</script>

<span class="tag" style:background>{label}</span>

<style>
  .tag {
    display: inline-block;
    padding: 2px 6px;
    border-radius: 4px;
    border: 1px solid rgba(255, 255, 255, 0.15);
    font-size: 10px;
    font-weight: 700;
    color: rgba(255, 255, 255, 0.9);
    white-space: nowrap;
  }
</style>
