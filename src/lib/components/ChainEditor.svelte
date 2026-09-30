<script lang="ts">
  import { tick } from "svelte";
  import { flip } from "svelte/animate";
  import type { CodecMeta } from "../../bindings/CodecMeta";
  import type { StepReport } from "../../bindings/StepReport";
  import { moveLayer, newLayer, type Layer } from "../chain";

  import LayerCard from "./LayerCard.svelte";

  interface Props {
    layers: Layer[];
    codecs: CodecMeta[];
    /** Compte rendu de la dernière exécution, par clé de couche. */
    reports: Map<number, StepReport>;
    idle: boolean;
    invertDisabled: boolean;
    invertTitle: string;
    oninvert: () => void;
  }

  let {
    layers = $bindable(),
    codecs,
    reports,
    idle,
    invertDisabled,
    invertTitle,
    oninvert,
  }: Props = $props();

  let list: HTMLOListElement | undefined = $state();
  let addButton: HTMLButtonElement | undefined = $state();
  let dragging = $state<number | null>(null);

  /** Déplace une couche en gardant le focus là où il était. */
  async function move(key: number, delta: number) {
    const from = layers.findIndex((layer) => layer.key === key);
    const focused = document.activeElement as HTMLElement | null;
    layers = moveLayer(layers, from, from + delta);
    await tick();
    focused?.focus();
  }

  async function add() {
    const meta = codecs.find((codec) => codec.id === "base64") ?? codecs[0];
    if (!meta) return;
    const layer = newLayer(meta, layers.at(-1)?.step.direction ?? "decode");
    layers = [...layers, layer];
    await tick();
    list?.querySelector<HTMLElement>(`[data-layer-key="${layer.key}"] .picker-button`)?.focus();
  }

  async function remove(key: number) {
    const index = layers.findIndex((layer) => layer.key === key);
    layers = layers.filter((layer) => layer.key !== key);
    await tick();
    const next = layers[Math.min(index, layers.length - 1)];
    const target = next
      ? list?.querySelector<HTMLElement>(`[data-layer-key="${next.key}"] .layer-remove`)
      : addButton;
    target?.focus();
  }

  /** Alt+↑ et Alt+↓ déplacent la couche qui contient le focus. */
  function onlistkeydown(event: KeyboardEvent) {
    if (!event.altKey || (event.key !== "ArrowUp" && event.key !== "ArrowDown")) return;
    const slot = (event.target as HTMLElement).closest<HTMLElement>("[data-layer-key]");
    if (!slot) return;
    event.preventDefault();
    move(Number(slot.dataset.layerKey), event.key === "ArrowUp" ? -1 : 1);
  }

  $effect(() => {
    list?.addEventListener("keydown", onlistkeydown);
    return () => list?.removeEventListener("keydown", onlistkeydown);
  });

  /**
   * Glisser-déplacer à la souris par la poignée. Les évènements de pointeur remplacent
   * le glisser-déposer HTML, que Tauri intercepte sous Windows pour les fichiers. Ils sont
   * écoutés sur la fenêtre : la couche déplacée change de place dans le DOM, ce qui ferait
   * perdre à sa poignée la capture du pointeur.
   */
  function grab(event: PointerEvent, key: number) {
    if (event.button !== 0 || !list) return;
    event.preventDefault();
    dragging = key;

    const onpointermove = (moveEvent: PointerEvent) => {
      if (!list) return;
      // Nouvel index : nombre d'autres couches dont le milieu est au-dessus du pointeur.
      const y = moveEvent.clientY - list.getBoundingClientRect().top + list.scrollTop;
      let to = 0;
      for (const slot of list.querySelectorAll<HTMLElement>("[data-layer-key]")) {
        if (Number(slot.dataset.layerKey) === key) continue;
        if (y > slot.offsetTop + slot.offsetHeight / 2) to++;
      }
      const from = layers.findIndex((layer) => layer.key === key);
      if (to !== from) layers = moveLayer(layers, from, to);
    };
    const release = () => {
      dragging = null;
      window.removeEventListener("pointermove", onpointermove);
      window.removeEventListener("pointerup", release);
      window.removeEventListener("pointercancel", release);
    };
    window.addEventListener("pointermove", onpointermove);
    window.addEventListener("pointerup", release);
    window.addEventListener("pointercancel", release);
  }
</script>

<section class="chain-column glass-card" aria-labelledby="chain-title">
  <div class="panel-header chain-header">
    <div class="panel-title-group">
      <span class="panel-indicator chain-indicator"></span>
      <h2 class="panel-title" id="chain-title">Chaîne</h2>
      <span class="badge-role">{layers.length} couche{layers.length > 1 ? "s" : ""}</span>
    </div>
    <button
      class="btn-panel-action"
      disabled={invertDisabled}
      title={invertTitle}
      onclick={oninvert}
    >
      <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
        <polyline points="7 3 3 7 7 11"></polyline>
        <path d="M3 7h13a5 5 0 0 1 0 10h-1"></path>
        <polyline points="17 21 21 17 17 13"></polyline>
        <path d="M21 17H8"></path>
      </svg>
      <span>Inverser</span>
    </button>
  </div>

  <ol class="layer-list" bind:this={list}>
    {#each layers as layer, index (layer.key)}
      <li class="layer-slot" data-layer-key={layer.key} animate:flip={{ duration: 160 }}>
        <LayerCard
          {layer}
          {index}
          {codecs}
          report={reports.get(layer.key)}
          {idle}
          dragging={dragging === layer.key}
          onchange={(step) => (layers[index] = { ...layer, step })}
          onremove={() => remove(layer.key)}
          onmove={(delta) => move(layer.key, delta)}
          ongrab={(event) => grab(event, layer.key)}
        />
      </li>
    {/each}
  </ol>

  {#if layers.length === 0}
    <p class="chain-empty">
      Chaîne vide : la sortie reprend l'entrée. Ajoutez une couche, ou chargez une recette.
    </p>
  {/if}

  <button class="btn-add-layer" bind:this={addButton} onclick={add}>
    <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.2" aria-hidden="true">
      <line x1="12" y1="5" x2="12" y2="19"></line>
      <line x1="5" y1="12" x2="19" y2="12"></line>
    </svg>
    <span>Ajouter une couche</span>
  </button>
</section>
