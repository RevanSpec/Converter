// Vérifie que le build (dist/) ne référence aucune ressource externe :
// l'application doit fonctionner hors ligne et ne rien envoyer sur le réseau.
import { readdirSync, readFileSync } from "node:fs";
import { extname, join } from "node:path";

const DIST = "dist";
const TEXT_FILES = new Set([".html", ".js", ".mjs", ".css", ".json", ".svg", ".webmanifest"]);
const URL_PATTERN = /https?:\/\/[^\s"'`()<>\\]+/g;
const ALLOWED = [
  /^http:\/\/ipc\.localhost/, // IPC de Tauri, local à l'application
  /^http:\/\/www\.w3\.org\//, // espaces de noms XML (SVG), jamais téléchargés
  /^https?:\/\/example\.(com|org|net)(\/|$)/, // domaines réservés aux exemples (RFC 2606)
  /^https:\/\/svelte\.dev\/e\//, // texte des erreurs du runtime Svelte (lien de documentation), jamais téléchargé
];

function* walk(dir) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) yield* walk(path);
    else yield path;
  }
}

const offenders = [];
for (const file of walk(DIST)) {
  if (!TEXT_FILES.has(extname(file))) continue;
  for (const [url] of readFileSync(file, "utf8").matchAll(URL_PATTERN)) {
    if (!ALLOWED.some((allowed) => allowed.test(url))) offenders.push(`${file} : ${url}`);
  }
}

if (offenders.length > 0) {
  console.error(`URL externes trouvées dans ${DIST}/ :\n${offenders.join("\n")}`);
  process.exit(1);
}
console.log(`${DIST}/ : aucune URL externe.`);
