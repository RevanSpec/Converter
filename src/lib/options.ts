import type { CodecMeta } from "../bindings/CodecMeta";
import type { Options } from "../bindings/Options";

/** Valeurs par défaut de toutes les options d'un format, telles que décrites par le moteur. */
export function defaultOptions(meta: CodecMeta): Options {
  return Object.fromEntries(meta.options.map((spec) => [spec.id, spec.kind.default]));
}
