import { describe, expect, it } from "vitest";
import { spanToUtf16 } from "./positions";

describe("spanToUtf16", () => {
  it("ne change rien pour un texte sans emoji", () => {
    expect(spanToUtf16("48 65 zz", { start: 6, end: 7 })).toEqual({ start: 6, end: 7 });
  });

  it("compte deux unités UTF-16 par caractère hors BMP", () => {
    // « 🦀 » est 1 caractère côté Rust, 2 unités côté JavaScript.
    expect(spanToUtf16("🦀ab", { start: 1, end: 2 })).toEqual({ start: 2, end: 3 });
    expect(spanToUtf16("a🦀b", { start: 1, end: 2 })).toEqual({ start: 1, end: 3 });
  });

  it("ramène une plage trop longue à la fin du texte", () => {
    expect(spanToUtf16("abc", { start: 2, end: 9 })).toEqual({ start: 2, end: 3 });
    expect(spanToUtf16("abc", { start: 7, end: 9 })).toEqual({ start: 3, end: 3 });
  });
});
