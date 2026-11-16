<script lang="ts" generics="Value">
  let { options, value = $bindable() }: { options: { value: Value; label: string }[]; value: Value } = $props();
</script>

<div class="pills" role="radiogroup">
  {#each options as option (option.label)}
    <button class="pill" class:selected={option.value === value} role="radio" aria-checked={option.value === value} onclick={() => (value = option.value)}>
      {option.label}
    </button>
  {/each}
</div>

<style>
  .pills {
    display: flex;
    gap: 4px;
  }

  .pill {
    padding: 6px 14px;
    border-radius: 999px;
    background: var(--field);
    color: rgb(var(--ink) / 0.85);
    font-weight: 500;
    transition: background 0.15s ease-out;
  }

  .pill.selected {
    background: rgb(var(--ink));
    color: var(--on-ink);
  }
</style>
