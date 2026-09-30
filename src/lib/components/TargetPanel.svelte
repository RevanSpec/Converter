<script lang="ts">
  import { bytesPerLine, HEXDUMP_LIMIT, hexdump, hexToBytes } from "../hexdump";

  interface Props {
    title: string;
    value: string;
    /** Le résultat n'est pas du texte : ses octets sont affichés en hexdump. */
    binary: boolean;
    chars: number;
    bytes: number;
    status: string;
    copyDisabled: boolean;
    copied: boolean;
    oncopy: () => void;
    oncopybase64: () => void;
  }

  let { title, value, binary, chars, bytes, status, copyDisabled, copied, oncopy, oncopybase64 }: Props =
    $props();

  /** Chasse de JetBrains Mono (0,6 em) à 0,8 rem, et marges intérieures de la zone de texte. */
  const CHAR_WIDTH = 0.6 * 12.8;
  const PADDING = 2 * 18 + 6;

  let bodyWidth = $state(0);

  // Décalage, octets en hexadécimal et caractères ASCII, comme `hexdump -C`, avec autant
  // d'octets par ligne que la largeur du panneau en permet.
  const dump = $derived(
    binary
      ? hexdump(hexToBytes(value), bytesPerLine(Math.floor((bodyWidth - PADDING) / CHAR_WIDTH)))
      : ""
  );
</script>

<div class="panel-card glass-card">
  <div class="panel-header">
    <div class="panel-title-group">
      <span class="panel-indicator target-indicator"></span>
      <h2 class="panel-title">{title}</h2>
      <span class="badge-role">Résultat</span>
      {#if binary}
        <span class="badge-binary" title="Le résultat n'est pas du texte UTF-8 : ses octets sont affichés en hexdump.">
          Octets
        </span>
      {/if}
    </div>
    <div class="panel-actions">
      {#if binary}
        <button
          class="btn-panel-action"
          title="Copier les octets en Base64"
          disabled={copyDisabled}
          onclick={oncopybase64}
        >
          Base64
        </button>
      {/if}
      <button
        class="btn-panel-action btn-highlight"
        class:btn-copied={copied}
        title={binary ? "Copier les octets en hexadécimal" : "Copier le résultat"}
        disabled={copyDisabled}
        onclick={oncopy}
      >
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
        </svg>
        <span>{copied ? "Copié !" : binary ? "Hex" : "Copier"}</span>
      </button>
    </div>
  </div>

  <div class="panel-body" bind:clientWidth={bodyWidth}>
    <textarea
      class="editor-textarea readonly"
      class:hexdump={binary}
      readonly
      wrap={binary ? "off" : "soft"}
      value={binary ? dump : value}
      placeholder="Le résultat de la conversion apparaîtra ici..."
      spellcheck="false"
      aria-label={binary
        ? "Résultat binaire en hexdump : décalage, octets en hexadécimal, caractères ASCII"
        : "Résultat de la conversion"}
    ></textarea>
  </div>

  <div class="panel-footer">
    <div class="stats-group">
      {#if binary}
        <span>{bytes}</span> octets
        <span class="stats-separator">•</span>
        {bytes > HEXDUMP_LIMIT
          ? `${HEXDUMP_LIMIT.toLocaleString("fr-FR")} premiers affichés, copie complète`
          : "pas du texte UTF-8"}
      {:else}
        <span>{chars}</span> car.
        <span class="stats-separator">•</span>
        <span>{bytes}</span> octets
      {/if}
    </div>
    <div class="conversion-status">
      <span class="status-pulse"></span>
      {status}
    </div>
  </div>
</div>
