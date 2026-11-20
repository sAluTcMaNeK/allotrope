// Outils PDF : appels au module Rust `pdf` (src-tauri/src/pdf.rs) et miniatures des pages avec pdf.js.
import { invoke } from "@tauri-apps/api/core";
import { GlobalWorkerOptions, getDocument, type PDFDocumentProxy } from "pdfjs-dist";
import urlTravailleur from "pdfjs-dist/build/pdf.worker.min.mjs?url";
import { octetsCible, type OptionsCompression } from "../fichiers";

// pdf.js décode les PDF dans un « worker » (un fil séparé) pour ne pas figer l'interface
GlobalWorkerOptions.workerSrc = urlTravailleur;

/** Miroir de la structure Rust `PageChoisie`. */
export interface PageChoisie {
  /** Indice du document dans la liste des sources. */
  source: number;
  /** Numéro de page, à partir de 1. */
  page: number;
  /** Rotation à ajouter, en degrés. */
  rotation: number;
}

/** Miroir de la structure Rust `ResultatPdf`. */
export interface ResultatPdf {
  chemin: string;
  taille: number;
  pages: number;
}

export type FormatPage = "image" | "a4";

export function infosPdf(chemin: string): Promise<{ pages: number }> {
  return invoke("infos_pdf", { chemin });
}

export function assemblerPdf(
  sources: string[],
  pages: PageChoisie[],
  dossier: string,
  nom: string,
): Promise<ResultatPdf> {
  return invoke("assembler_pdf", { sources, pages, dossier, nom });
}

export function imagesEnPdf(
  images: string[],
  dossier: string,
  nom: string,
  formatPage: FormatPage,
  marge: number,
): Promise<ResultatPdf> {
  return invoke("images_en_pdf", { images, dossier, nom, formatPage, marge });
}

/** Miroir de la structure Rust `ResultatCompressionPdf`. `chemin` est `null` si aucun gain n'était possible. */
export interface ResultatCompressionPdf {
  chemin: string | null;
  taille: number;
  images: number;
}

/** Compresse un PDF avec les mêmes réglages que les autres fichiers (taux ou taille cible, dimensions max.). */
export function compresserPdf(
  source: string,
  dossier: string,
  options: OptionsCompression,
): Promise<ResultatCompressionPdf> {
  return invoke("compresser_pdf", {
    source,
    dossier,
    taux: options.taux,
    tailleCible: options.type === "taille" ? octetsCible(options) : null,
    tailleMax: options.tailleMax,
  });
}

/** Miroir de la structure Rust `ImageTrouvee`. */
export interface ImageTrouvee {
  /** Identifiant de l'image dans le PDF, à renvoyer pour l'extraction. */
  id: string;
  page: number | null;
  largeur: number;
  hauteur: number;
  /** Format du fichier extrait : « jpg », « jp2 » ou « png ». */
  format: string;
  /** Miniature (data URL), ou `null` si l'image n'a pas pu être décodée. */
  apercu: string | null;
}

/** Miroir de la structure Rust `ResultatExtraction`. */
export interface ResultatExtraction {
  /** Dossier créé, ou archive ZIP. */
  chemin: string;
  nombre: number;
  echecs: number;
}

export function analyserImagesPdf(source: string): Promise<ImageTrouvee[]> {
  return invoke("analyser_images_pdf", { source });
}

export function extraireImagesPdf(
  source: string,
  ids: string[],
  dossier: string,
  zip: boolean,
): Promise<ResultatExtraction> {
  return invoke("extraire_images_pdf", { source, ids, dossier, zip });
}

/** Ouvre un PDF avec pdf.js pour en afficher les pages (les octets sont lus par le cœur Rust). */
export async function ouvrirPdf(chemin: string): Promise<PDFDocumentProxy> {
  const octets = await invoke<ArrayBuffer>("lire_pdf", { chemin });
  return getDocument({
    data: new Uint8Array(octets),
    // Ressources copiées localement par scripts/copier-pdfjs.mjs : aucun accès réseau
    standardFontDataUrl: "/pdfjs/standard_fonts/",
    cMapUrl: "/pdfjs/cmaps/",
    cMapPacked: true,
    wasmUrl: "/pdfjs/wasm/",
    iccUrl: "/pdfjs/iccs/",
  }).promise;
}

/** Dessine la page `numero` dans `canvas`, à la largeur d'affichage `largeur` (en pixels CSS). */
export async function dessinerPage(
  document: PDFDocumentProxy,
  numero: number,
  canvas: HTMLCanvasElement,
  largeur: number,
): Promise<void> {
  const page = await document.getPage(numero);
  const naturelle = page.getViewport({ scale: 1 });
  // Rendu net sur les écrans haute densité
  const echelle = (largeur * window.devicePixelRatio) / naturelle.width;
  const vue = page.getViewport({ scale: echelle });
  canvas.width = Math.round(vue.width);
  canvas.height = Math.round(vue.height);
  await page.render({ canvas, viewport: vue }).promise;
}

// --- PDF en images ---

export type FormatPageImage = "png" | "jpeg" | "webp";

/** Miroir de la structure Rust `ResultatExport`. */
export interface ResultatExport {
  /** Dossier créé, ou archive ZIP. */
  chemin: string;
  nombre: number;
}

/** Côté maximal d'une page rendue, en pixels (limite de taille des canevas du navigateur). */
const COTE_MAX_RENDU = 10_000;

/** Dessine une page à `ppp` points par pouce, sur fond blanc, et renvoie l'image PNG (sans perte). */
async function rendrePagePng(pdf: PDFDocumentProxy, numero: number, ppp: number): Promise<Uint8Array> {
  const page = await pdf.getPage(numero);
  const naturelle = page.getViewport({ scale: 1 });
  // 1 point PDF = 1/72 de pouce
  const echelle = Math.min(ppp / 72, COTE_MAX_RENDU / Math.max(naturelle.width, naturelle.height));
  const vue = page.getViewport({ scale: echelle });
  const canvas = window.document.createElement("canvas");
  canvas.width = Math.round(vue.width);
  canvas.height = Math.round(vue.height);
  try {
    await page.render({ canvas, viewport: vue, background: "white" }).promise;
    const image = await new Promise<Blob>((ok, echec) =>
      canvas.toBlob((b) => (b ? ok(b) : echec(new Error("Rendu de la page impossible"))), "image/png"),
    );
    return new Uint8Array(await image.arrayBuffer());
  } finally {
    // Libère tout de suite la mémoire du canevas et de la page
    canvas.width = 0;
    canvas.height = 0;
    page.cleanup();
  }
}

/**
 * Exporte des pages en images : chacune est dessinée par pdf.js puis envoyée au cœur Rust,
 * qui l'encode (PNG, JPG, WEBP) et l'enregistre dans un dossier ou une archive ZIP.
 * `onprogression` reçoit le nombre de pages faites ; `annule()` permet d'arrêter en cours de route.
 */
export async function exporterPages(
  pdf: PDFDocumentProxy,
  source: string,
  pages: number[],
  options: { dossier: string; zip: boolean; format: FormatPageImage; ppp: number; qualite: number },
  onprogression: (faites: number) => void,
  annule: () => boolean,
): Promise<ResultatExport> {
  const id = await invoke<number>("export_pages_debut", { source, dossier: options.dossier, zip: options.zip });
  try {
    for (const [index, numero] of pages.entries()) {
      if (annule()) throw "Annulé";
      const png = await rendrePagePng(pdf, numero, options.ppp);
      // Image envoyée telle quelle (binaire brut) ; les réglages passent par les en-têtes
      await invoke("export_pages_ajouter", png, {
        headers: {
          export: String(id),
          page: String(numero),
          format: options.format,
          qualite: String(options.qualite),
        },
      });
      onprogression(index + 1);
    }
    return await invoke<ResultatExport>("export_pages_fin", { export: id });
  } catch (e) {
    await invoke("export_pages_annuler", { export: id });
    throw e;
  }
}

// --- PDF en document Word / OpenDocument ---

export type FormatTexte = "docx" | "odt";
export type MoteurBureautique = "word" | "libreoffice";

export function moteursBureautique(): Promise<{ word: boolean; libreoffice: boolean }> {
  return invoke("moteurs_bureautique");
}

export function pdfVersTexte(
  source: string,
  dossier: string,
  format: FormatTexte,
  moteur: MoteurBureautique,
): Promise<{ chemin: string; taille: number }> {
  return invoke("pdf_vers_texte", { source, dossier, format, moteur });
}

/** Nom du fichier sans dossier ni extension (« rapport » pour « C:\…\rapport.pdf »). */
export function nomSansExtension(chemin: string): string {
  const nom = chemin.split(/[\\/]/).pop() ?? chemin;
  return nom.replace(/\.[^.]+$/, "");
}

/** Documents que l'on sait transformer en PDF (miroir de EXTENSIONS_DOCUMENT, bureautique.rs). */
export const EXTENSIONS_DOCUMENT: readonly string[] = ["doc", "docx", "docm", "dot", "dotx", "odt", "ott", "rtf", "txt"];

/** Transforme un document (Word, ODT, RTF, texte) en PDF avec Word ou LibreOffice. */
export function documentVersPdf(
  source: string,
  dossier: string,
  moteur: MoteurBureautique,
): Promise<{ chemin: string; taille: number }> {
  return invoke("document_vers_pdf", { source, dossier, moteur });
}
