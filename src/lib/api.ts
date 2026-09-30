import { invoke } from "@tauri-apps/api/core";
import type { CodecError } from "../bindings/CodecError";
import type { CodecMeta } from "../bindings/CodecMeta";
import type { ConvertRequest } from "../bindings/ConvertRequest";
import type { ConvertResponse } from "../bindings/ConvertResponse";
import type { PipelineRequest } from "../bindings/PipelineRequest";
import type { PipelineResponse } from "../bindings/PipelineResponse";
import type { Preset } from "../bindings/Preset";
import type { SavedRecipe } from "../bindings/SavedRecipe";
import type { Step } from "../bindings/Step";

/** Formats disponibles et leurs options, décrits par le moteur Rust. */
export function listCodecs(): Promise<CodecMeta[]> {
  return invoke<CodecMeta[]>("list_codecs");
}

export function convert(request: ConvertRequest): Promise<ConvertResponse> {
  return invoke<ConvertResponse>("convert", { request });
}

/** Toute une chaîne de couches, avec un compte rendu par couche. */
export function runPipeline(request: PipelineRequest): Promise<PipelineResponse> {
  return invoke<PipelineResponse>("run_pipeline", { request });
}

/** Chaîne inverse : ordre et sens inversés. Rejetée si une couche est irréversible. */
export function invertChain(steps: Step[]): Promise<Step[]> {
  return invoke<Step[]>("invert_chain", { steps });
}

/** Forme courte d'une chaîne, par exemple `base64:dec|hex:dec`. */
export function recipeToShort(steps: Step[]): Promise<string> {
  return invoke<string>("recipe_to_short", { steps });
}

export function recipeFromShort(text: string): Promise<Step[]> {
  return invoke<Step[]>("recipe_from_short", { text });
}

export function listPresets(): Promise<Preset[]> {
  return invoke<Preset[]>("list_presets");
}

/** « Mes recettes » : lues au démarrage, écrites seulement sur action de l'utilisateur. */
export function listSavedRecipes(): Promise<SavedRecipe[]> {
  return invoke<SavedRecipe[]>("list_saved_recipes");
}

export function saveRecipe(name: string, steps: Step[]): Promise<SavedRecipe[]> {
  return invoke<SavedRecipe[]>("save_recipe", { name, steps });
}

export function deleteSavedRecipe(name: string): Promise<SavedRecipe[]> {
  return invoke<SavedRecipe[]>("delete_saved_recipe", { name });
}

/**
 * Ramène toute erreur (du moteur ou d'ailleurs) à la forme d'une `CodecError`. Les
 * erreurs de chaîne (`PipelineError`) ont aussi un code, un message et une plage.
 */
export function asCodecError(error: unknown): CodecError {
  if (typeof error === "object" && error !== null && "code" in error && "message" in error) {
    const { code, message, span } = error as CodecError;
    return { code, message, span: span ?? null };
  }
  return { code: "internal", message: String(error), span: null };
}
