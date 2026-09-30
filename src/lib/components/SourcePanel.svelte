<script lang="ts">
  import type { Span } from "../../bindings/Span";
  import { spanToUtf16 } from "../positions";

  interface Props {
    title: string;
    value: string;
    /** Plage fautive à surligner, en caractères Unicode. */
    highlight: Span | null;
    chars: number;
    bytes: number;
    oninput: (value: string) => void;
    onpaste: () => void;
    onclear: () => void;
    textarea?: HTMLTextAreaElement;
  }

  let {
    title,
    value,
    highlight,
    chars,
    bytes,
    oninput,
    onpaste,
    onclear,
    textarea = $bindable(),
  }: Props = $props();

  let backdrop: HTMLDivElement | undefined = $state();

  // Texte découpé autour de la plage fautive ; le calque est derrière la zone de saisie,
  // le curseur de l'utilisateur n'est donc jamais déplacé.
  const parts = $derived.by(() => {
    if (!highlight) return null;
    const { start, end } = spanToUtf16(value, highlight);
    return { before: value.slice(0, start), marked: value.slice(start, end), after: value.slice(end) };
  });

  function syncScroll() {
    if (backdrop && textarea) {
      backdrop.scrollTop = textarea.scrollTop;
      backdrop.scrollLeft = textarea.scrollLeft;
    }
  }

  $effect(() => {
    if (parts && backdrop) syncScroll();
  });
</script>

<div class="panel-card glass-card">
  <div class="panel-header">
    <div class="panel-title-group">
      <span class="panel-indicator source-indicator"></span>
      <h2 class="panel-title">{title}</h2>
      <span class="badge-role">Entrée</span>
    </div>
    <div class="panel-actions">
      <button class="btn-panel-action" title="Coller depuis le presse-papier" onclick={onpaste}>
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"></path>
          <rect x="8" y="2" width="8" height="4" rx="1" ry="1"></rect>
        </svg>
        <span>Coller</span>
      </button>
      <button
        class="btn-panel-action btn-danger-hover"
        title="Effacer la saisie"
        aria-label="Effacer la saisie"
        onclick={onclear}
      >
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    </div>
  </div>

  <div class="panel-body">
    {#if parts}
      <div class="editor-backdrop" bind:this={backdrop} aria-hidden="true">{parts.before}<mark
          >{parts.marked || " "}</mark
        >{parts.after}{"\n"}</div>
    {/if}
    <textarea
      bind:this={textarea}
      class="editor-textarea"
      {value}
      oninput={(event) => oninput(event.currentTarget.value)}
      onscroll={syncScroll}
      placeholder="Tapez ou collez votre texte ici..."
      spellcheck="false"
      aria-label="Texte à convertir"
    ></textarea>
  </div>

  <div class="panel-footer">
    <div class="stats-group">
      <span>{chars}</span> car.
      <span class="stats-separator">•</span>
      <span>{bytes}</span> octets UTF-8
    </div>
    <div class="editor-hint">Saisie en direct</div>
  </div>
</div>
