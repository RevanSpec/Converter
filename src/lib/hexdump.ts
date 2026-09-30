/** Octets affichés au plus dans l'hexdump : au-delà, la copie reste complète. */
export const HEXDUMP_LIMIT = 16_384;

/** Octets en hexadécimal tels que le moteur les rend : « ff d8 ff e0 ». */
export function hexToBytes(hex: string): Uint8Array {
  const digits = hex.replace(/\s+/g, "");
  const bytes = new Uint8Array(digits.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(digits.slice(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}

/** Base64 standard, avec padding. */
export function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  // Par tranches, pour ne pas dépasser le nombre d'arguments d'une fonction.
  for (let i = 0; i < bytes.length; i += 0x8000) {
    binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  }
  return btoa(binary);
}

/** Longueur d'une ligne d'hexdump, en caractères, pour `perLine` octets par ligne. */
export function lineLength(perLine: number): number {
  return 14 + 4 * perLine;
}

/** Octets par ligne (16, 8 ou 4) qui tiennent dans `columns` caractères. */
export function bytesPerLine(columns: number): number {
  return [16, 8].find((perLine) => lineLength(perLine) <= columns) ?? 4;
}

function hexByte(byte: number): string {
  return byte.toString(16).padStart(2, "0");
}

/**
 * Hexdump façon `hexdump -C` : décalage, octets en hexadécimal (en deux groupes), puis
 * leurs caractères ASCII imprimables. S'arrête après `limit` octets.
 */
export function hexdump(bytes: Uint8Array, perLine = 16, limit = HEXDUMP_LIMIT): string {
  const shown = Math.min(bytes.length, limit);
  const half = perLine / 2;
  const lines: string[] = [];
  for (let offset = 0; offset < shown; offset += perLine) {
    const line = bytes.subarray(offset, Math.min(offset + perLine, shown));
    const hex = Array.from({ length: perLine }, (_, i) =>
      i < line.length ? hexByte(line[i]) : "  "
    );
    const ascii = Array.from(line, (byte) =>
      byte >= 0x20 && byte < 0x7f ? String.fromCharCode(byte) : "."
    ).join("");
    lines.push(
      `${offset.toString(16).padStart(8, "0")}  ${hex.slice(0, half).join(" ")}  ${hex.slice(half).join(" ")}  |${ascii}|`
    );
  }
  return lines.join("\n");
}
