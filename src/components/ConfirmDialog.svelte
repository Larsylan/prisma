<script lang="ts">
  let {
    title,
    message,
    confirmLabel,
    onConfirm,
    onCancel,
  }: { title: string; message: string; confirmLabel: string; onConfirm: () => void; onCancel: () => void } = $props();

  function handleKeydown(event: KeyboardEvent) {
    if (event.key !== 'Escape') return;
    event.preventDefault();
    onCancel();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="backdrop" role="presentation" onclick={onCancel}>
  <div class="dialog" role="alertdialog" aria-modal="true" aria-label={title} tabindex="-1" onclick={(event) => event.stopPropagation()} onkeydown={() => {}}>
    <h2>{title}</h2>
    <p class="secondary">{message}</p>
    <div class="actions">
      <button class="button" onclick={onCancel}>Annuler</button>
      <button class="button danger" onclick={onConfirm}>{confirmLabel}</button>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.5);
    z-index: 10;
  }

  .dialog {
    width: 340px;
    padding: 20px;
    border-radius: 14px;
    background: var(--dialog);
    border: 1px solid var(--stroke);
    box-shadow: 0 20px 50px rgba(0, 0, 0, 0.6);
  }

  h2 {
    margin: 0 0 8px;
    font-size: 15px;
  }

  p {
    margin: 0 0 18px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
