<script lang="ts">
  import type { OptionSpec } from "../../bindings/OptionSpec";
  import type { OptionValue } from "../../bindings/OptionValue";

  interface Props {
    spec: OptionSpec;
    value: OptionValue | undefined;
    /** Préfixe des identifiants HTML, unique pour chaque couche en mode Chaîne. */
    idPrefix: string;
    codecId: string;
    /** Variante des cartes de couche : listes déroulantes et champs numériques. */
    compact?: boolean;
    onchange: (value: OptionValue) => void;
  }

  let { spec, value, idPrefix, codecId, compact = false, onchange }: Props = $props();

  const kind = $derived(spec.kind);
  const current = $derived(value ?? spec.kind.default);
  const fieldId = $derived(`${idPrefix}-${spec.id}`);

  /** Le décalage César s'affiche « +13 (ROT13) », les autres entiers tels quels. */
  function intLabel(n: number): string {
    if (codecId !== "caesar") return String(n);
    return n === 13 ? "+13 (ROT13)" : `+${n}`;
  }

  /** N'envoie un entier que s'il est dans la plage de l'option. */
  function inputInt(input: HTMLInputElement, min: number, max: number) {
    const n = input.valueAsNumber;
    if (Number.isInteger(n) && n >= min && n <= max) onchange(n);
  }
</script>

{#if kind.type === "choice"}
  {#if compact}
    <label class="opt-compact">
      <span class="opt-compact-label">{spec.label}</span>
      <select
        class="opt-select"
        value={String(current)}
        onchange={(event) => onchange(event.currentTarget.value)}
      >
        {#each kind.choices as choice (choice.value)}
          <option value={choice.value}>{choice.label}</option>
        {/each}
      </select>
    </label>
  {:else}
    <span class="opt-label" id={fieldId}>{spec.label} :</span>
    <div class="segmented-control" role="group" aria-labelledby={fieldId}>
      {#each kind.choices as choice (choice.value)}
        <button
          class="segment"
          class:active={current === choice.value}
          aria-pressed={current === choice.value}
          onclick={() => onchange(choice.value)}
        >
          {choice.label}
        </button>
      {/each}
    </div>
  {/if}
{:else if kind.type === "bool"}
  <div class="toggle-control">
    <label class="checkbox-container">
      <input
        type="checkbox"
        checked={current === true}
        onchange={(event) => onchange(event.currentTarget.checked)}
      />
      <span class="checkmark"></span>
      <span class="checkbox-text">{spec.label}</span>
    </label>
  </div>
{:else if kind.type === "int"}
  {#if compact}
    <label class="opt-compact">
      <span class="opt-compact-label">{spec.label}</span>
      <input
        class="opt-number"
        type="number"
        min={kind.min}
        max={kind.max}
        value={Number(current)}
        oninput={(event) => inputInt(event.currentTarget, kind.min, kind.max)}
        onchange={(event) => (event.currentTarget.value = String(current))}
      />
    </label>
  {:else}
    <label class="opt-label" for={fieldId}>{spec.label} :</label>
    <div class="slider-container">
      <input
        type="range"
        id={fieldId}
        min={kind.min}
        max={kind.max}
        value={Number(current)}
        aria-valuetext={intLabel(Number(current))}
        oninput={(event) => onchange(Number(event.currentTarget.value))}
      />
      <span class="slider-badge">{intLabel(Number(current))}</span>
    </div>
    <button
      class="btn-micro"
      disabled={current === kind.default}
      onclick={() => onchange(kind.default)}
    >
      Réinitialiser ({kind.default})
    </button>
  {/if}
{:else if kind.type === "text"}
  <label class={compact ? "opt-compact" : "opt-label"} for={fieldId}>
    {#if compact}<span class="opt-compact-label">{spec.label}</span>{:else}{spec.label} :{/if}
  </label>
  <input
    class="opt-text"
    id={fieldId}
    type={kind.secret ? "password" : "text"}
    autocomplete="off"
    value={String(current)}
    oninput={(event) => onchange(event.currentTarget.value)}
  />
{/if}
