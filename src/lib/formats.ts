import type { Category } from "../bindings/Category";
import type { CodecMeta } from "../bindings/CodecMeta";

export const CATEGORY_LABELS: Record<Category, string> = {
  bytes: "Octets",
  text: "Texte",
  web: "Web",
  cipher: "Chiffrement",
  compression: "Compression",
};

export interface CodecGroup {
  category: Category;
  codecs: CodecMeta[];
}

/** Formats regroupés par catégorie, les catégories dans l'ordre de leur premier format. */
export function groupByCategory(codecs: CodecMeta[]): CodecGroup[] {
  const groups: CodecGroup[] = [];
  for (const codec of codecs) {
    const group = groups.find((g) => g.category === codec.category);
    if (group) group.codecs.push(codec);
    else groups.push({ category: codec.category, codecs: [codec] });
  }
  return groups;
}

/** Texte sans accents ni majuscules, pour une recherche tolérante : « Décimal » ≈ « decimal ». */
function fold(text: string): string {
  return text.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLowerCase();
}

/**
 * Formats dont le libellé, l'identifiant, un alias ou la catégorie contient chacun des
 * mots de la recherche.
 */
export function searchCodecs(codecs: CodecMeta[], query: string): CodecMeta[] {
  const words = fold(query).split(/\s+/).filter(Boolean);
  if (words.length === 0) return codecs;
  return codecs.filter((codec) => {
    const haystack = fold(
      [codec.label, codec.id, ...codec.aliases, CATEGORY_LABELS[codec.category]].join(" ")
    );
    return words.every((word) => haystack.includes(word));
  });
}
