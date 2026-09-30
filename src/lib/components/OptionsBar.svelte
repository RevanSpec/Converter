<script lang="ts">
  import type { CodecMeta } from "../../bindings/CodecMeta";
  import type { OptionValue } from "../../bindings/OptionValue";
  import type { Options } from "../../bindings/Options";

  interface Props {
    codec: CodecMeta | undefined;
    options: Options;
    onchange: (id: string, value: OptionValue) => void;
    morsePlaying: boolean;
    ontogglemorse: () => void;
  }

  let { codec, options, onchange, morsePlaying, ontogglemorse }: Props = $props();

  /** Le décalage César s'affiche « +13 (ROT13) », les autres entiers tels quels. */
  function intLabel(value: number): string {
    if (codec?.id !== "caesar") return String(value);
    return value === 13 ? "+13 (ROT13)" : `+${value}`;
  }
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
      {@const kind = spec.kind}
      {@const value = options[spec.id] ?? kind.default}
      {#if kind.type === "choice"}
        <span class="opt-label" id="opt-{spec.id}">{spec.label} :</span>
        <div class="segmented-control" role="group" aria-labelledby="opt-{spec.id}">
          {#each kind.choices as choice (choice.value)}
            <button
              class="segment"
              class:active={value === choice.value}
              aria-pressed={value === choice.value}
              onclick={() => onchange(spec.id, choice.value)}
            >
              {choice.label}
            </button>
          {/each}
        </div>
      {:else if kind.type === "bool"}
        <div class="toggle-control">
          <label class="checkbox-container">
            <input
              type="checkbox"
              checked={value === true}
              onchange={(event) => onchange(spec.id, event.currentTarget.checked)}
            />
            <span class="checkmark"></span>
            <span class="checkbox-text">{spec.label}</span>
          </label>
        </div>
      {:else if kind.type === "int"}
        <label class="opt-label" for="opt-{spec.id}">{spec.label} :</label>
        <div class="slider-container">
          <input
            type="range"
            id="opt-{spec.id}"
            min={kind.min}
            max={kind.max}
            value={Number(value)}
            aria-valuetext={intLabel(Number(value))}
            oninput={(event) => onchange(spec.id, Number(event.currentTarget.value))}
          />
          <span class="slider-badge">{intLabel(Number(value))}</span>
        </div>
        <button
          class="btn-micro"
          disabled={value === kind.default}
          onclick={() => onchange(spec.id, kind.default)}
        >
          Réinitialiser ({kind.default})
        </button>
      {:else if kind.type === "text"}
        <label class="opt-label" for="opt-{spec.id}">{spec.label} :</label>
        <input
          class="opt-text"
          id="opt-{spec.id}"
          type={kind.secret ? "password" : "text"}
          autocomplete="off"
          value={String(value)}
          oninput={(event) => onchange(spec.id, event.currentTarget.value)}
        />
      {/if}
    {:else}
      <span class="generic-hint">Conversion instantanée bidirectionnelle en temps réel</span>
    {/each}
  </div>
</div>
