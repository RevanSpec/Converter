<script lang="ts">
  import type { CodecMeta } from "../../bindings/CodecMeta";
  import type { OptionValue } from "../../bindings/OptionValue";
  import type { Options } from "../../bindings/Options";
  import OptionField from "./OptionField.svelte";

  interface Props {
    codec: CodecMeta | undefined;
    options: Options;
    onchange: (id: string, value: OptionValue) => void;
    morsePlaying: boolean;
    ontogglemorse: () => void;
  }

  let { codec, options, onchange, morsePlaying, ontogglemorse }: Props = $props();
</script>

<div
  class="options-bar glass-card"
  id="format-panel"
  role="tabpanel"
  aria-labelledby={codec ? `tab-${codec.id}` : undefined}
>
  <div class="options-group">
    {#if codec?.id === "morse"}
      <button
        class="btn-sound"
        aria-pressed={morsePlaying}
        title="Écouter la transcription sonore du code Morse"
        onclick={ontogglemorse}
      >
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
          <path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"></path>
        </svg>
        <span>{morsePlaying ? "Arrêter l'audio" : "Écouter le Morse (Bips audio)"}</span>
      </button>
    {/if}

    {#each codec?.options ?? [] as spec (spec.id)}
      <OptionField
        {spec}
        value={options[spec.id]}
        idPrefix="opt"
        codecId={codec?.id ?? ""}
        onchange={(value) => onchange(spec.id, value)}
      />
    {:else}
      <span class="generic-hint">Conversion instantanée bidirectionnelle en temps réel</span>
    {/each}
  </div>
</div>
