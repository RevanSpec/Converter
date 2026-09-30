import type { Span } from "../bindings/Span";

/**
 * Convertit une plage en caractères Unicode (celle du moteur Rust) en indices UTF-16,
 * ceux de JavaScript et de `setSelectionRange` : un emoji compte pour 1 côté Rust et
 * pour 2 côté JavaScript. Une plage qui dépasse le texte est ramenée à sa fin.
 */
export function spanToUtf16(text: string, span: Span): { start: number; end: number } {
  let start = text.length;
  let end = text.length;
  let codePoint = 0;
  let index = 0;
  while (index <= text.length) {
    if (codePoint === span.start) start = index;
    if (codePoint === span.end) {
      end = index;
      break;
    }
    if (index === text.length) break;
    index += (text.codePointAt(index) ?? 0) > 0xffff ? 2 : 1;
    codePoint++;
  }
  return { start: Math.min(start, end), end };
}
