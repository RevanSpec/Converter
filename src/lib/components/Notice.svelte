<script lang="ts">
  interface Props {
    level: "error" | "warning";
    message: string;
    /** Présent quand l'erreur porte sur une plage du texte saisi. */
    onlocate?: () => void;
  }

  let { level, message, onlocate }: Props = $props();
</script>

<div
  class="error-banner glass-card"
  class:warning={level === "warning"}
  role={level === "error" ? "alert" : "status"}
>
  <div class="error-icon" aria-hidden="true">
    <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="10"></circle>
      <line x1="12" y1="8" x2="12" y2="12"></line>
      <line x1="12" y1="16" x2="12.01" y2="16"></line>
    </svg>
  </div>
  <div class="error-message">{message}</div>
  {#if onlocate}
    <button class="btn-micro" onclick={onlocate}>Voir dans le texte</button>
  {/if}
</div>
