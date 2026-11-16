<script lang="ts">
  let { value, height = 6, colors }: { value: number; height?: number; colors?: [string, string] } = $props();

  let fill = $derived(colors ? `linear-gradient(90deg, ${colors[0]}, ${colors[1]})` : 'var(--progress)');
</script>

<div class="track" style:height="{height}px">
  <div class="fill" style:width="max({value > 0 ? height : 0}px, {Math.min(1, value) * 100}%)" style:background={fill}></div>
</div>

<style>
  .track {
    flex: 1;
    border-radius: 999px;
    background: rgb(var(--ink) / 0.12);
  }

  .fill {
    height: 100%;
    border-radius: 999px;
    box-shadow: var(--glow);
  }
</style>
