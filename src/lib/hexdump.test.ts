import { describe, expect, it } from "vitest";
import { bytesPerLine, bytesToBase64, hexdump, hexToBytes, lineLength } from "./hexdump";

describe("hexToBytes et bytesToBase64", () => {
  it("relit l'hexadécimal du moteur et le donne en Base64", () => {
    const bytes = hexToBytes("ff d8 ff e0");
    expect(Array.from(bytes)).toEqual([0xff, 0xd8, 0xff, 0xe0]);
    expect(bytesToBase64(bytes)).toBe("/9j/4A==");
    expect(bytesToBase64(new Uint8Array())).toBe("");
  });

  it("encode aussi de gros volumes", () => {
    const bytes = new Uint8Array(100_000).fill(0x41);
    expect(bytesToBase64(bytes)).toBe("QUFB".repeat(33_333) + "QQ==");
  });
});

describe("hexdump", () => {
  it("aligne décalage, hexadécimal et ASCII comme hexdump -C", () => {
    const bytes = new TextEncoder().encode("Hello, Glass Converter!\n");
    // Une ligne incomplète garde la colonne ASCII à sa place (colonne 60).
    expect(hexdump(bytes)).toBe(
      [
        "00000000  48 65 6c 6c 6f 2c 20 47  6c 61 73 73 20 43 6f 6e  |Hello, Glass Con|",
        "00000010  76 65 72 74 65 72 21 0a" + " ".repeat(27) + "|verter!.|",
      ].join("\n")
    );
  });

  it("s'arrête à la limite", () => {
    const lines = hexdump(new Uint8Array(100), 16, 32).split("\n");
    expect(lines).toHaveLength(2);
    expect(lines[1].startsWith("00000010")).toBe(true);
  });

  it("s'adapte à la largeur disponible", () => {
    expect(bytesPerLine(80)).toBe(16);
    expect(bytesPerLine(60)).toBe(8);
    expect(bytesPerLine(20)).toBe(4);
    const line = hexdump(new TextEncoder().encode("Hello, Glass"), 8).split("\n")[0];
    expect(line).toBe("00000000  48 65 6c 6c  6f 2c 20 47  |Hello, G|");
    expect(line).toHaveLength(lineLength(8));
  });
});
