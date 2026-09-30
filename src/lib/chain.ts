import type { CodecMeta } from "../bindings/CodecMeta";
import type { Direction } from "../bindings/Direction";
import type { Step } from "../bindings/Step";
import { defaultOptions } from "./options";

/** Une couche de l'éditeur : une étape du moteur et une clé stable pour l'affichage. */
export interface Layer {
  key: number;
  step: Step;
}

let lastKey = 0;

export function newLayer(meta: CodecMeta, direction: Direction): Layer {
  return {
    key: ++lastKey,
    step: { codec: meta.id, direction, options: defaultOptions(meta), enabled: true },
  };
}

export function fromSteps(steps: Step[]): Layer[] {
  return steps.map((step) => ({ key: ++lastKey, step }));
}

export function toSteps(layers: Layer[]): Step[] {
  return layers.map((layer) => layer.step);
}

/** Copie de `layers` où la couche d'index `from` passe à l'index `to`. */
export function moveLayer(layers: Layer[], from: number, to: number): Layer[] {
  const target = Math.max(0, Math.min(layers.length - 1, to));
  if (from === target || from < 0 || from >= layers.length) return layers;
  const moved = [...layers];
  const [layer] = moved.splice(from, 1);
  moved.splice(target, 0, layer);
  return moved;
}

/** Taille lisible en octets décimaux : « 12 o », « 1,5 Ko », « 3,2 Mo ». */
export function formatSize(bytes: number): string {
  if (bytes < 1000) return `${bytes} o`;
  const [value, unit] = bytes < 1_000_000 ? [bytes / 1000, "Ko"] : [bytes / 1_000_000, "Mo"];
  return `${value.toLocaleString("fr-FR", { maximumFractionDigits: 1 })} ${unit}`;
}

/** Durée lisible depuis des microsecondes : « < 0,1 ms », « 2,4 ms », « 1,2 s ». */
export function formatDuration(micros: number): string {
  if (micros < 100) return "< 0,1 ms";
  if (micros < 1_000_000) {
    return `${(micros / 1000).toLocaleString("fr-FR", { maximumFractionDigits: micros < 10_000 ? 1 : 0 })} ms`;
  }
  return `${(micros / 1_000_000).toLocaleString("fr-FR", { maximumFractionDigits: 1 })} s`;
}
