import { describe, expect, it } from "vitest";
import type { CodecMeta } from "../bindings/CodecMeta";
import { defaultOptions } from "./options";

const hex: CodecMeta = {
  id: "hex",
  label: "Hexadécimal",
  icon: "⬡",
  category: "bytes",
  aliases: [],
  reversible: true,
  encodes_text: false,
  options: [
    {
      id: "separator",
      label: "Séparateur",
      kind: { type: "choice", default: " ", choices: [{ value: " ", label: "Espace" }] },
    },
    { id: "uppercase", label: "Majuscules", kind: { type: "bool", default: false } },
    { id: "shift", label: "Décalage", kind: { type: "int", default: 13, min: 1, max: 25 } },
  ],
};

describe("defaultOptions", () => {
  it("reprend la valeur par défaut de chaque option", () => {
    expect(defaultOptions(hex)).toEqual({ separator: " ", uppercase: false, shift: 13 });
  });

  it("renvoie un objet vide pour un format sans option", () => {
    expect(defaultOptions({ ...hex, options: [] })).toEqual({});
  });
});
