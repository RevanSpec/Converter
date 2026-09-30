import { describe, expect, it } from "vitest";
import { charCount, formatMegabytes, utf8ByteLength } from "./text";

describe("utf8ByteLength", () => {
  it("compte les octets UTF-8", () => {
    expect(utf8ByteLength("a")).toBe(1);
    expect(utf8ByteLength("é")).toBe(2);
    expect(utf8ByteLength("€")).toBe(3);
    expect(utf8ByteLength("🦀")).toBe(4);
  });
});

describe("charCount", () => {
  it("compte un emoji hors BMP comme un seul caractère", () => {
    expect(charCount("🦀a")).toBe(2);
    expect(charCount("été")).toBe(3);
    expect(charCount("")).toBe(0);
  });
});

describe("formatMegabytes", () => {
  it("affiche des mégaoctets à la française", () => {
    expect(formatMegabytes(12_345_678)).toBe("12,3 Mo");
    expect(formatMegabytes(50_000_000)).toBe("50 Mo");
  });
});
