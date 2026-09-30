import { describe, expect, it } from "vitest";
import type { CodecMeta } from "../bindings/CodecMeta";
import { groupByCategory, searchCodecs } from "./formats";

function codec(id: string, label: string, category: CodecMeta["category"], aliases: string[] = []): CodecMeta {
  return { id, label, icon: id, category, aliases, reversible: true, encodes_text: false, options: [] };
}

const codecs = [
  codec("hex", "Hexadécimal", "bytes", ["base16"]),
  codec("morse", "Code Morse", "text"),
  codec("base64", "Base64", "bytes", ["b64"]),
  codec("gzip", "gzip", "compression", ["gz"]),
];

describe("groupByCategory", () => {
  it("garde l'ordre du premier format de chaque catégorie", () => {
    expect(groupByCategory(codecs).map((g) => [g.category, g.codecs.map((c) => c.id)])).toEqual([
      ["bytes", ["hex", "base64"]],
      ["text", ["morse"]],
      ["compression", ["gzip"]],
    ]);
  });
});

describe("searchCodecs", () => {
  const ids = (query: string) => searchCodecs(codecs, query).map((c) => c.id);

  it("ignore les accents et la casse", () => {
    expect(ids("HEXADECIMAL")).toEqual(["hex"]);
    expect(ids("décimal")).toEqual(["hex"]);
  });

  it("cherche aussi dans les alias et la catégorie", () => {
    expect(ids("b64")).toEqual(["base64"]);
    expect(ids("compression")).toEqual(["gzip"]);
  });

  it("exige chacun des mots", () => {
    expect(ids("code morse")).toEqual(["morse"]);
    expect(ids("code base")).toEqual([]);
    expect(ids("   ")).toHaveLength(4);
  });
});
