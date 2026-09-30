const encoder = new TextEncoder();

/** Taille du texte en octets UTF-8, comme côté Rust. */
export function utf8ByteLength(text: string): number {
  return encoder.encode(text).length;
}

/** Nombre de caractères Unicode (points de code), comme `chars().count()` côté Rust. */
export function charCount(text: string): number {
  let count = 0;
  for (let i = 0; i < text.length; i++) {
    const unit = text.charCodeAt(i);
    // La seconde moitié d'une paire de substitution ne compte pas comme un caractère.
    if (unit < 0xdc00 || unit > 0xdfff) count++;
  }
  return count;
}

/** Taille lisible en mégaoctets, à une décimale près. */
export function formatMegabytes(bytes: number): string {
  const megabytes = (bytes / 1_000_000).toLocaleString("fr-FR", {
    maximumFractionDigits: 1,
  });
  return `${megabytes} Mo`;
}
