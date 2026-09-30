<script lang="ts">
  import type { CodecMeta } from "../../bindings/CodecMeta";
  import type { Direction } from "../../bindings/Direction";
  import type { Step } from "../../bindings/Step";
  import type { StepReport } from "../../bindings/StepReport";
  import { formatDuration, formatSize, type Layer } from "../chain";
  import { defaultOptions } from "../options";
  import FormatPicker from "./FormatPicker.svelte";
  import OptionField from "./OptionField.svelte";

  interface Props {
    layer: Layer;
    index: number;
    codecs: CodecMeta[];
    report: StepReport | undefined;
    /** Rien n'est saisi : la chaîne n'a pas tourné. */
    idle: boolean;
    dragging: boolean;
    onchange: (step: Step) => void;
    onremove: () => void;
    onmove: (delta: number) => void;
    ongrab: (event: PointerEvent) => void;
  }

  let { layer, index, codecs, report, idle, dragging, onchange, onremove, onmove, ongrab }: Props =
    $props();

  const step = $derived(layer.step);
  const meta = $derived(codecs.find((codec) => codec.id === step.codec));
  const number = $derived(index + 1);
  const preview = $derived(
    report?.preview?.kind === "text" ? report.preview.text : (report?.preview?.hex ?? "")
  );

  function selectCodec(id: string) {
    const next = codecs.find((codec) => codec.id === id);
    if (next && id !== step.codec) onchange({ ...step, codec: id, options: defaultOptions(next) });
  }

  function setDirection(direction: Direction) {
    onchange({ ...step, direction });
  }

  /** Sur la poignée, les flèches seules déplacent la couche ; Alt+flèche est géré par la liste. */
  function onhandlekeydown(event: KeyboardEvent) {
    if (event.altKey) return;
    if (event.key === "ArrowUp" || event.key === "ArrowDown") {
      event.preventDefault();
      onmove(event.key === "ArrowUp" ? -1 : 1);
    }
  }
</script>

<div
  class="layer-card"
  class:failed={report?.status === "failed"}
  class:off={!step.enabled}
  class:skipped={report?.status === "skipped"}
  class:dragging
>
  <div class="layer-head">
    <button
      class="layer-handle"
      title="Glisser pour déplacer (ou Alt+↑ / Alt+↓)"
      aria-label="Déplacer la couche {number} : flèches haut et bas"
      onpointerdown={ongrab}
      onkeydown={onhandlekeydown}
    >
      <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor" aria-hidden="true">
        <circle cx="9" cy="6" r="1.6"></circle>
        <circle cx="15" cy="6" r="1.6"></circle>
        <circle cx="9" cy="12" r="1.6"></circle>
        <circle cx="15" cy="12" r="1.6"></circle>
        <circle cx="9" cy="18" r="1.6"></circle>
        <circle cx="15" cy="18" r="1.6"></circle>
      </svg>
    </button>
    <span class="layer-index" aria-hidden="true">{number}</span>
    <FormatPicker
      {codecs}
      value={step.codec}
      id="layer-{layer.key}-format"
      label="Format de la couche {number}"
      onselect={selectCodec}
    />
    <label class="layer-toggle" title={step.enabled ? "Désactiver la couche" : "Activer la couche"}>
      <input
        type="checkbox"
        checked={step.enabled}
        aria-label="Couche {number} active"
        onchange={(event) => onchange({ ...step, enabled: event.currentTarget.checked })}
      />
      <span class="layer-toggle-track" aria-hidden="true"></span>
    </label>
    <button
      class="layer-remove btn-danger-hover"
      title="Supprimer la couche"
      aria-label="Supprimer la couche {number}"
      onclick={onremove}
    >
      <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2.2" aria-hidden="true">
        <line x1="18" y1="6" x2="6" y2="18"></line>
        <line x1="6" y1="6" x2="18" y2="18"></line>
      </svg>
    </button>
  </div>

  <div class="layer-settings">
    <div class="segmented-control layer-direction" role="group" aria-label="Sens de la couche {number}">
      <button
        class="segment"
        class:active={step.direction === "encode"}
        aria-pressed={step.direction === "encode"}
        onclick={() => setDirection("encode")}
      >
        Encoder
      </button>
      <button
        class="segment"
        class:active={step.direction === "decode"}
        aria-pressed={step.direction === "decode"}
        onclick={() => setDirection("decode")}
      >
        Décoder
      </button>
    </div>
    {#each meta?.options ?? [] as spec (spec.id)}
      <OptionField
        {spec}
        value={step.options[spec.id]}
        idPrefix="layer-{layer.key}"
        codecId={step.codec}
        compact
        onchange={(value) => onchange({ ...step, options: { ...step.options, [spec.id]: value } })}
      />
    {/each}
  </div>

  <div class="layer-result">
    {#if idle}
      <p class="layer-status">En attente d'une saisie</p>
    {:else if !report}
      <p class="layer-status">Conversion…</p>
    {:else if report.status === "done"}
      <p class="layer-status ok">
        {formatSize(report.bytes)} · {report.is_text ? "texte" : "octets"} · {formatDuration(report.micros)}{report.truncated
          ? " · aperçu tronqué"
          : ""}
      </p>
      <pre class="layer-preview" class:binary={!report.is_text}>{preview || "(vide)"}</pre>
    {:else if report.status === "disabled"}
      <p class="layer-status">Désactivée : les données passent telles quelles</p>
    {:else if report.status === "failed"}
      <p class="layer-status error">{report.error?.message ?? "Erreur"}</p>
    {:else}
      <p class="layer-status">Non exécutée : une couche précédente a échoué</p>
    {/if}
  </div>
</div>
