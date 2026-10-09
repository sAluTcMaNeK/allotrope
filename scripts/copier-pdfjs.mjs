// Copie les ressources de pdf.js (polices standard, tables de caractères, décodeurs) dans public/pdfjs/,
// pour que les miniatures des PDF s'affichent correctement sans aucun accès réseau.
// Lancé automatiquement avant `npm run dev` et `npm run build` (scripts « predev » / « prebuild »).
import { cpSync, existsSync } from "node:fs";

const source = "node_modules/pdfjs-dist";
const destination = "public/pdfjs";

for (const dossier of ["standard_fonts", "cmaps", "wasm", "iccs"]) {
  if (existsSync(`${source}/${dossier}`)) {
    cpSync(`${source}/${dossier}`, `${destination}/${dossier}`, { recursive: true });
  }
}
