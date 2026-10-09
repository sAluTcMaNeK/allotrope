// Outils PDF : appels au module Rust `pdf` (src-tauri/src/pdf.rs) et miniatures des pages avec pdf.js.
import { invoke } from "@tauri-apps/api/core";
import { GlobalWorkerOptions, getDocument, type PDFDocumentProxy } from "pdfjs-dist";
import urlTravailleur from "pdfjs-dist/build/pdf.worker.min.mjs?url";

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

/** Nom du fichier sans dossier ni extension (« rapport » pour « C:\…\rapport.pdf »). */
export function nomSansExtension(chemin: string): string {
  const nom = chemin.split(/[\\/]/).pop() ?? chemin;
  return nom.replace(/\.[^.]+$/, "");
}
