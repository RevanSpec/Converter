<script lang="ts">
  import { tick } from "svelte";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import type { CodecError } from "../../bindings/CodecError";
  import type { CodecMeta } from "../../bindings/CodecMeta";
  import type { Preset } from "../../bindings/Preset";
  import type { SavedRecipe } from "../../bindings/SavedRecipe";
  import type { Step } from "../../bindings/Step";
  import {
    asCodecError,
    deleteSavedRecipe,
    recipeFromShort,
    recipeToShort,
    saveRecipe,
  } from "../api";
  import { spanToUtf16 } from "../positions";

  interface Props {
    steps: Step[];
    codecs: CodecMeta[];
    presets: Preset[];
    saved: SavedRecipe[];
    /** Remplace la chaîne par une recette ; `name` est absent pour une recette collée. */
    onload: (steps: Step[], name: string | null) => void;
    onsaved: (saved: SavedRecipe[]) => void;
    ontoast: (message: string, error?: boolean) => void;
  }

  let { steps, codecs, presets, saved, onload, onsaved, ontoast }: Props = $props();

  // Forme courte de la chaîne, modifiable pour importer une recette collée.
  let text = $state("");
  let edited = $state(false);
  let error = $state<CodecError | null>(null);
  let field: HTMLInputElement | undefined = $state();

  let menuOpen = $state(false);
  let confirming = $state<string | null>(null);
  let menu: HTMLDivElement | undefined = $state();
  let menuButton: HTMLButtonElement | undefined = $state();

  let naming = $state(false);
  let name = $state("");
  let nameField: HTMLInputElement | undefined = $state();

  // La forme courte suit la chaîne, sauf pendant que l'utilisateur la modifie.
  $effect(() => {
    const current = $state.snapshot(steps);
    if (edited) return;
    let cancelled = false;
    const timer = setTimeout(async () => {
      try {
        const short = await recipeToShort(current);
        if (!cancelled) {
          text = short;
          error = null;
        }
      } catch (reason) {
        if (!cancelled) error = asCodecError(reason);
      }
    }, 60);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  });

  async function importText() {
    try {
      const imported = await recipeFromShort(text);
      edited = false;
      error = null;
      onload(imported, null);
    } catch (reason) {
      error = asCodecError(reason);
      if (error.span && field) {
        const { start, end } = spanToUtf16(text, error.span);
        field.focus();
        field.setSelectionRange(start, end);
      }
    }
  }

  function revert() {
    edited = false;
    error = null;
  }

  async function copyText() {
    try {
      await writeText(text);
      ontoast("Recette copiée : collez-la dans ce champ pour la réimporter");
    } catch (reason) {
      ontoast(`Copie impossible : ${String(reason)}`, true);
    }
  }

  async function loadPreset(preset: Preset) {
    closeMenu(false);
    try {
      onload(await recipeFromShort(preset.recipe), preset.name);
    } catch (reason) {
      ontoast(asCodecError(reason).message, true);
    }
  }

  function loadSaved(recipe: SavedRecipe) {
    closeMenu(false);
    onload(recipe.recipe.steps, recipe.name);
  }

  async function remove(recipeName: string) {
    try {
      onsaved(await deleteSavedRecipe(recipeName));
      ontoast(`Recette « ${recipeName} » supprimée`);
    } catch (reason) {
      ontoast(asCodecError(reason).message, true);
    }
    confirming = null;
  }

  async function startNaming() {
    naming = true;
    name = "";
    await tick();
    nameField?.focus();
  }

  async function save() {
    const trimmed = name.trim();
    try {
      onsaved(await saveRecipe(trimmed, $state.snapshot(steps)));
      ontoast(`Recette « ${trimmed} » enregistrée dans Mes recettes`);
      naming = false;
    } catch (reason) {
      ontoast(asCodecError(reason).message, true);
    }
  }

  async function toggleMenu() {
    if (menuOpen) {
      closeMenu(false);
      return;
    }
    menuOpen = true;
    confirming = null;
    await tick();
    menu?.querySelector<HTMLElement>("button")?.focus();
  }

  function closeMenu(focusButton: boolean) {
    menuOpen = false;
    confirming = null;
    if (focusButton) menuButton?.focus();
  }

  function onwindowpointerdown(event: PointerEvent) {
    const target = event.target as Node;
    if (menuOpen && !menu?.contains(target) && !menuButton?.contains(target)) closeMenu(false);
  }

  function onwindowkeydown(event: KeyboardEvent) {
    if (menuOpen && event.key === "Escape") {
      event.preventDefault();
      closeMenu(true);
    }
  }

  /** « URL (déc.) → Base64 (déc.) » : le résumé d'une recette enregistrée. */
  function summary(recipe: SavedRecipe): string {
    return recipe.recipe.steps
      .map((step) => {
        const label = codecs.find((codec) => codec.id === step.codec)?.label ?? step.codec;
        return `${label} (${step.direction === "encode" ? "enc." : "déc."})`;
      })
      .join(" → ");
  }
</script>

<svelte:window onpointerdown={onwindowpointerdown} onkeydown={onwindowkeydown} />

<div class="chain-toolbar glass-card">
  <div class="chain-toolbar-row">
    <div class="recipes-anchor">
      <button
        bind:this={menuButton}
        class="btn-ghost"
        aria-expanded={menuOpen}
        aria-controls="recipes-menu"
        onclick={toggleMenu}
      >
        <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"></path>
          <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"></path>
        </svg>
        <span>Recettes</span>
        <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <polyline points="6 9 12 15 18 9"></polyline>
        </svg>
      </button>

      {#if menuOpen}
        <div
          bind:this={menu}
          class="recipes-menu glass-card"
          id="recipes-menu"
          role="dialog"
          aria-label="Recettes"
        >
          <p class="menu-section-title">Prêtes à l'emploi</p>
          {#each presets as preset (preset.id)}
            <button class="menu-item" onclick={() => loadPreset(preset)}>
              <span class="menu-item-name">{preset.name}</span>
              <span class="menu-item-detail">{preset.description}</span>
            </button>
          {/each}

          <p class="menu-section-title">Mes recettes</p>
          {#each saved as recipe (recipe.name)}
            <div class="menu-row">
              <button class="menu-item" onclick={() => loadSaved(recipe)}>
                <span class="menu-item-name">{recipe.name}</span>
                <span class="menu-item-detail">{summary(recipe)}</span>
              </button>
              {#if confirming === recipe.name}
                <button class="btn-micro btn-danger-hover" onclick={() => remove(recipe.name)}>
                  Supprimer
                </button>
                <button class="btn-micro" onclick={() => (confirming = null)}>Garder</button>
              {:else}
                <button
                  class="menu-delete btn-danger-hover"
                  title="Supprimer cette recette"
                  aria-label="Supprimer la recette {recipe.name}"
                  onclick={() => (confirming = recipe.name)}
                >
                  <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.2" aria-hidden="true">
                    <line x1="18" y1="6" x2="6" y2="18"></line>
                    <line x1="6" y1="6" x2="18" y2="18"></line>
                  </svg>
                </button>
              {/if}
            </div>
          {:else}
            <p class="menu-empty">Aucune recette enregistrée : « Enregistrer » garde la chaîne en cours.</p>
          {/each}
          <p class="menu-note">
            Mes recettes restent sur cet ordinateur et ne contiennent jamais le texte saisi.
          </p>
        </div>
      {/if}
    </div>

    <label class="recipe-field">
      <span class="recipe-label">Recette</span>
      <input
        bind:this={field}
        class="recipe-input"
        type="text"
        value={text}
        placeholder="base64:dec|hex:dec"
        spellcheck="false"
        autocomplete="off"
        aria-invalid={error !== null}
        aria-describedby={error ? "recipe-error" : undefined}
        oninput={(event) => {
          text = event.currentTarget.value;
          edited = true;
          error = null;
        }}
        onkeydown={(event) => {
          if (event.key === "Enter") importText();
          if (event.key === "Escape" && edited) revert();
        }}
      />
    </label>
    <button class="btn-panel-action" title="Copier la forme courte de la chaîne" disabled={!text} onclick={copyText}>
      Copier
    </button>
    <button
      class="btn-panel-action"
      title="Remplacer la chaîne par la recette saisie (Entrée)"
      disabled={!edited}
      onclick={importText}
    >
      Importer
    </button>

    {#if naming}
      <form
        class="save-form"
        onsubmit={(event) => {
          event.preventDefault();
          save();
        }}
      >
        <input
          bind:this={nameField}
          bind:value={name}
          class="recipe-name"
          type="text"
          maxlength="60"
          placeholder="Nom de la recette"
          aria-label="Nom de la recette"
          onkeydown={(event) => event.key === "Escape" && (naming = false)}
        />
        <button class="btn-panel-action btn-highlight" type="submit" disabled={!name.trim()}>
          OK
        </button>
        <button class="btn-panel-action" type="button" onclick={() => (naming = false)}>
          Annuler
        </button>
      </form>
    {:else}
      <button
        class="btn-ghost"
        title="Ajouter la chaîne à Mes recettes (sans le texte saisi)"
        disabled={steps.length === 0}
        onclick={startNaming}
      >
        <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"></path>
          <polyline points="17 21 17 13 7 13 7 21"></polyline>
          <polyline points="7 3 7 8 15 8"></polyline>
        </svg>
        <span>Enregistrer</span>
      </button>
    {/if}
  </div>

  {#if error}
    <p class="recipe-error" id="recipe-error" role="alert">{error.message}</p>
  {/if}
</div>
