import { describe, expect, it } from "vitest";
import type { CodecMeta } from "../bindings/CodecMeta";
import { formatDuration, formatSize, fromSteps, moveLayer, newLayer, toSteps } from "./chain";

const base64: CodecMeta = {
  id: "base64",
  label: "Base64",
  icon: "B64",
  category: "bytes",
  aliases: [],
  reversible: true,
  encodes_text: false,
  options: [{ id: "padding", label: "Padding", kind: { type: "bool", default: true } }],
};

describe("couches", () => {
  it("une nouvelle couche prend les options par défaut du format", () => {
    const layer = newLayer(base64, "decode");
    expect(layer.step).toEqual({
      codec: "base64",
      direction: "decode",
      options: { padding: true },
      enabled: true,
    });
  });

  it("chaque couche a sa propre clé, et les étapes ressortent telles quelles", () => {
    const steps = toSteps([newLayer(base64, "encode"), newLayer(base64, "decode")]);
    const layers = fromSteps(steps);
    expect(new Set(layers.map((l) => l.key)).size).toBe(2);
    expect(toSteps(layers)).toEqual(steps);
  });

  it("déplace une couche sans toucher aux autres", () => {
    const layers = fromSteps(["a", "b", "c"].map((codec) => ({ ...newLayer(base64, "encode").step, codec })));
    const codecs = (moved: typeof layers) => moved.map((l) => l.step.codec);
    expect(codecs(moveLayer(layers, 0, 2))).toEqual(["b", "c", "a"]);
    expect(codecs(moveLayer(layers, 2, 1))).toEqual(["a", "c", "b"]);
    expect(codecs(moveLayer(layers, 0, -1))).toEqual(["a", "b", "c"]);
    expect(moveLayer(layers, 1, 1)).toBe(layers);
  });
});

describe("tailles et durées", () => {
  it("affiche des octets décimaux à la française", () => {
    expect(formatSize(12)).toBe("12 o");
    expect(formatSize(1_530)).toBe("1,5 Ko");
    expect(formatSize(3_200_000)).toBe("3,2 Mo");
  });

  it("affiche des durées lisibles", () => {
    expect(formatDuration(40)).toBe("< 0,1 ms");
    expect(formatDuration(2_400)).toBe("2,4 ms");
    expect(formatDuration(45_600)).toBe("46 ms");
    expect(formatDuration(1_250_000)).toBe("1,3 s");
  });
});
