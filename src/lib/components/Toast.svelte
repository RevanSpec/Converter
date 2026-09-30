<script lang="ts">
  interface Props {
    message: string;
    error: boolean;
    visible: boolean;
  }

  let { message, error, visible }: Props = $props();
</script>

<div class="toast-container" role="status" aria-live="polite">
  <div class="toast glass-card" class:show={visible} class:toast-error={error}>
    <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.5" aria-hidden="true">
      <polyline points="20 6 9 17 4 12"></polyline>
    </svg>
    <span>{message}</span>
  </div>
</div>
