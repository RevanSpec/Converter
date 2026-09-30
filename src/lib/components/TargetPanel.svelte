<script lang="ts">
  interface Props {
    title: string;
    value: string;
    /** Le résultat n'est pas du texte : ses octets sont affichés en hexadécimal. */
    binary: boolean;
    chars: number;
    bytes: number;
    status: string;
    copyDisabled: boolean;
    copied: boolean;
    oncopy: () => void;
  }

  let { title, value, binary, chars, bytes, status, copyDisabled, copied, oncopy }: Props = $props();
</script>

<div class="panel-card glass-card">
  <div class="panel-header">
    <div class="panel-title-group">
      <span class="panel-indicator target-indicator"></span>
      <h2 class="panel-title">{title}</h2>
      <span class="badge-role">Résultat</span>
      {#if binary}
        <span class="badge-binary" title="Le résultat n'est pas du texte UTF-8 : ses octets sont affichés en hexadécimal.">
          Octets (hex)
        </span>
      {/if}
    </div>
    <div class="panel-actions">
      <button
        class="btn-panel-action btn-highlight"
        class:btn-copied={copied}
        title="Copier le résultat"
        disabled={copyDisabled}
        onclick={oncopy}
      >
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
        </svg>
        <span>{copied ? "Copié !" : "Copier"}</span>
      </button>
    </div>
  </div>

  <div class="panel-body">
    <textarea
      class="editor-textarea readonly"
      readonly
      {value}
      placeholder="Le résultat de la conversion apparaîtra ici..."
      spellcheck="false"
      aria-label="Résultat de la conversion"
    ></textarea>
  </div>

  <div class="panel-footer">
    <div class="stats-group">
      <span>{chars}</span> {binary ? "octets affichés" : "car."}
      <span class="stats-separator">•</span>
      <span>{bytes}</span> octets
    </div>
    <div class="conversion-status">
      <span class="status-pulse"></span>
      {status}
    </div>
  </div>
</div>
