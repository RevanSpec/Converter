<script lang="ts">
  import type { Category } from "../../bindings/Category";
  import type { CodecMeta } from "../../bindings/CodecMeta";

  interface Props {
    codecs: CodecMeta[];
    selected: string;
    onselect: (id: string) => void;
  }

  let { codecs, selected, onselect }: Props = $props();

  const CATEGORY_LABELS: Record<Category, string> = {
    bytes: "Octets",
    text: "Texte",
    web: "Web",
    cipher: "Chiffrement",
  };

  // Catégories dans l'ordre de leur premier format.
  const groups = $derived.by(() => {
    const order: Category[] = [];
    for (const codec of codecs) {
      if (!order.includes(codec.category)) order.push(codec.category);
    }
    return order.map((category) => ({
      category,
      codecs: codecs.filter((codec) => codec.category === category),
    }));
  });

  // Ordre de navigation au clavier : celui de l'affichage.
  const ordered = $derived(groups.flatMap((group) => group.codecs));

  /** Flèches, Début et Fin déplacent la sélection d'onglet en onglet. */
  function onkeydown(event: KeyboardEvent) {
    const index = ordered.findIndex((codec) => codec.id === selected);
    const moves: Record<string, number> = {
      ArrowRight: index + 1,
      ArrowLeft: index - 1,
      Home: 0,
      End: ordered.length - 1,
    };
    const target = moves[event.key];
    if (target === undefined || ordered.length === 0) return;
    event.preventDefault();
    const next = ordered[(target + ordered.length) % ordered.length];
    onselect(next.id);
    document.getElementById(`tab-${next.id}`)?.focus();
  }
</script>

<nav class="formats-bar glass-card" aria-label="Formats de conversion">
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div class="formats-scroll" role="tablist" aria-label="Formats de conversion" {onkeydown}>
    {#each groups as group (group.category)}
      <span class="tab-group-label" aria-hidden="true">{CATEGORY_LABELS[group.category]}</span>
      {#each group.codecs as codec (codec.id)}
        <button
          id="tab-{codec.id}"
          class="format-tab"
          class:active={codec.id === selected}
          role="tab"
          aria-selected={codec.id === selected}
          aria-controls="format-panel"
          tabindex={codec.id === selected ? 0 : -1}
          onclick={() => onselect(codec.id)}
        >
          <span class="tab-icon" aria-hidden="true">{codec.icon}</span>
          <span class="tab-label">{codec.label}</span>
        </button>
      {/each}
    {/each}
  </div>
</nav>
