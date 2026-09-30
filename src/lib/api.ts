import { invoke } from "@tauri-apps/api/core";
import type { CodecError } from "../bindings/CodecError";
import type { CodecMeta } from "../bindings/CodecMeta";
import type { ConvertRequest } from "../bindings/ConvertRequest";
import type { ConvertResponse } from "../bindings/ConvertResponse";

/** Formats disponibles et leurs options, décrits par le moteur Rust. */
export function listCodecs(): Promise<CodecMeta[]> {
  return invoke<CodecMeta[]>("list_codecs");
}

export function convert(request: ConvertRequest): Promise<ConvertResponse> {
  return invoke<ConvertResponse>("convert", { request });
}

/** Ramène toute erreur (du moteur ou d'ailleurs) à la forme d'une `CodecError`. */
export function asCodecError(error: unknown): CodecError {
  if (typeof error === "object" && error !== null && "code" in error && "message" in error) {
    return error as CodecError;
  }
  return { code: "internal", message: String(error), span: null };
}
