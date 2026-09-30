<script lang="ts">
  import { tick } from "svelte";
  import type { CodecMeta } from "../../bindings/CodecMeta";
  import { CATEGORY_LABELS, groupByCategory, searchCodecs } from "../formats";

  interface Props {
    codecs: CodecMeta[];
    value: string;
    /** Préfixe des identifiants HTML, unique par couche. */
    id: string;
    /** Nom accessible du bouton, par exemple « Format de la couche 2 ». */
    label: string;
    onselect: (id: string) => void;
  }

  let { codecs, value, id, label, onselect }: Props = $props();

  let open = $state(false);
  let query = $state("");
  let active = $state(0);
  let root: HTMLDivElement | undefined = $state();
  let button: HTMLButtonElement | undefined = $state();
  let panel: HTMLDivElement | undefined = $state();
  let search: HTMLInputElement | undefined = $state();

  const current = $derived(codecs.find((codec) => codec.id === value));
  const groups = $derived(groupByCategory(searchCodecs(codecs, query)));
  // Ordre de navigation au clavier : celui de l'affichage, groupes compris.
  const ordered = $derived(groups.flatMap((group) => group.codecs));

  async function show() {
    query = "";
    open = true;
    active = Math.max(0, ordered.findIndex((codec) => codec.id === value));
    await tick();
    // La liste s'ouvre sous la couche : la colonne défile pour la montrer, ou au moins
    // son champ de recherche si elle est plus haute que la colonne.
    const column = root?.closest<HTMLElement>(".layer-list");
    const tall = panel && column && panel.offsetHeight > column.clientHeight;
    panel?.scrollIntoView({ block: tall ? "start" : "nearest" });
    search?.focus({ preventScroll: true });
    scrollToActive();
  }

  function hide(focusButton: boolean) {
    open = false;
    if (focusButton) button?.focus();
  }

  function choose(codec: CodecMeta) {
    onselect(codec.id);
    hide(true);
  }

  /** Fait défiler la liste seule (pas la colonne) jusqu'à l'option active. */
  function scrollToActive() {
    const option = document.getElementById(`${id}-option-${active}`);
    const list = option?.closest<HTMLElement>(".picker-list");
    if (!option || !list) return;
    const top = option.offsetTop - list.offsetTop;
    const bottom = top + option.offsetHeight;
    if (top < list.scrollTop) list.scrollTop = top;
    else if (bottom > list.scrollTop + list.clientHeight) list.scrollTop = bottom - list.clientHeight;
  }

  function onkeydown(event: KeyboardEvent) {
    const count = ordered.length;
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      if (count === 0) return;
      active = (active + (event.key === "ArrowDown" ? 1 : -1) + count) % count;
      scrollToActive();
    } else if (event.key === "Enter") {
      event.preventDefault();
      if (ordered[active]) choose(ordered[active]);
    } else if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      hide(true);
    } else if (event.key === "Tab") {
      hide(false);
    }
  }

  /** Un clic ailleurs referme la liste. */
  function onwindowpointerdown(event: PointerEvent) {
    if (open && root && !root.contains(event.target as Node)) hide(false);
  }
</script>

<svelte:window onpointerdown={onwindowpointerdown} />

<div class="format-picker" bind:this={root}>
  <button
    bind:this={button}
    class="picker-button"
    aria-haspopup="listbox"
    aria-expanded={open}
    aria-label="{label} : {current?.label ?? value}"
    onclick={() => (open ? hide(false) : show())}
  >
    <span class="tab-icon" aria-hidden="true">{current?.icon ?? "?"}</span>
    <span class="picker-current">{current?.label ?? value}</span>
    <svg class="picker-chevron" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
      <polyline points="6 9 12 15 18 9"></polyline>
    </svg>
  </button>

  {#if open}
    <div class="picker-panel" bind:this={panel}>
      <input
        bind:this={search}
        bind:value={query}
        class="picker-search"
        type="text"
        role="combobox"
        aria-label="Rechercher un format"
        aria-expanded="true"
        aria-controls="{id}-list"
        aria-autocomplete="list"
        aria-activedescendant={ordered.length > 0 ? `${id}-option-${active}` : undefined}
        placeholder="Rechercher : base64, gzip, url…"
        autocomplete="off"
        spellcheck="false"
        oninput={() => (active = 0)}
        {onkeydown}
      />
      <div class="picker-list" id="{id}-list" role="listbox" aria-label="Formats">
        {#each groups as group (group.category)}
          <div role="group" aria-labelledby="{id}-group-{group.category}">
            <div class="picker-group" id="{id}-group-{group.category}">
              {CATEGORY_LABELS[group.category]}
            </div>
            {#each group.codecs as codec (codec.id)}
              {@const index = ordered.indexOf(codec)}
              <div
                id="{id}-option-{index}"
                class="picker-option"
                class:active={index === active}
                class:selected={codec.id === value}
                role="option"
                tabindex="-1"
                aria-selected={codec.id === value}
                onclick={() => choose(codec)}
                onkeydown={(event) => event.key === "Enter" && choose(codec)}
                onpointermove={() => (active = index)}
              >
                <span class="tab-icon" aria-hidden="true">{codec.icon}</span>
                <span>{codec.label}</span>
              </div>
            {/each}
          </div>
        {:else}
          <div class="picker-empty">Aucun format ne correspond à « {query} ».</div>
        {/each}
      </div>
    </div>
  {/if}
</div>
