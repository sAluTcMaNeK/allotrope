// Lien avec le module Rust `fichiers` (src-tauri/src/fichiers.rs) et outils d'affichage.
import { invoke } from "@tauri-apps/api/core";

export type Famille = "image" | "video" | "audio" | "document";

/** Miroir de la structure Rust `InfoFichier`. */
export interface InfoFichier {
  chemin: string;
  nom: string;
  extension: string;
  taille: number;
  famille: Famille | null;
}

/** Demande au cœur Rust d'analyser des chemins (les dossiers sont ignorés). */
export function analyserFichiers(chemins: string[]): Promise<InfoFichier[]> {
  return invoke<InfoFichier[]>("analyser_fichiers", { chemins });
}

/** Présentation de chaque famille (voir DESIGN.md, « Couleur par famille de fichiers »). */
export const FAMILLES: Record<Famille, { libelle: string; couleur: string; texte: string; formats: string }> = {
  image: {
    libelle: "Images",
    couleur: "var(--famille-image)",
    texte: "var(--abysse)",
    formats: "JPG · PNG · WEBP · SVG · HEIC",
  },
  video: {
    libelle: "Vidéo",
    couleur: "var(--famille-video)",
    texte: "var(--blanc)",
    formats: "MP4 · MOV · MKV · GIF",
  },
  audio: {
    libelle: "Audio",
    couleur: "var(--famille-audio)",
    texte: "var(--abysse)",
    formats: "MP3 · WAV · FLAC",
  },
  document: {
    libelle: "Documents",
    couleur: "var(--famille-document)",
    texte: "var(--abysse)",
    formats: "DOCX · PDF · ODT",
  },
};

const nombre = new Intl.NumberFormat("fr-FR", { maximumFractionDigits: 1 });

/** Taille lisible à la française : « 640 Ko », « 2,4 Mo ». */
export function formaterTaille(octets: number): string {
  const unites = ["octets", "Ko", "Mo", "Go", "To"];
  let valeur = octets;
  let i = 0;
  while (valeur >= 1024 && i < unites.length - 1) {
    valeur /= 1024;
    i++;
  }
  return `${nombre.format(valeur)} ${unites[i]}`;
}
