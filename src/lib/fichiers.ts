// Lien avec les modules Rust `fichiers` et `conversion` (src-tauri/src/) et outils d'affichage.
import { invoke } from "@tauri-apps/api/core";

export type Famille = "image" | "video" | "audio" | "document";
export type FormatImage = "jpeg" | "png" | "webp";

/** Miroir de la structure Rust `InfoFichier`. */
export interface InfoFichier {
  chemin: string;
  nom: string;
  extension: string;
  taille: number;
  famille: Famille | null;
  /** [largeur, hauteur] en pixels, pour les images lisibles. */
  dimensions: [number, number] | null;
  /** Formats de conversion proposés, le format par défaut en premier. */
  formatsSortie: FormatImage[];
}

/** Miroir de la structure Rust `ResultatConversion`. */
export interface ResultatConversion {
  chemin: string;
  taille: number;
}

export type Etat = "attente" | "encours" | "termine" | "erreur";

/** Un fichier de la file d'attente et l'avancement de sa conversion. */
export interface ElementFile {
  info: InfoFichier;
  /** Format choisi ; `null` si le fichier ne peut pas être converti. */
  format: FormatImage | null;
  etat: Etat;
  resultat?: ResultatConversion;
  erreur?: string;
}

/** Demande au cœur Rust d'analyser des chemins (les dossiers sont ignorés). */
export function analyserFichiers(chemins: string[]): Promise<InfoFichier[]> {
  return invoke<InfoFichier[]>("analyser_fichiers", { chemins });
}

/** Convertit une image ; rejette la promesse avec un message en français en cas d'échec. */
export function convertirImage(
  source: string,
  dossier: string,
  format: FormatImage,
  qualite: number,
): Promise<ResultatConversion> {
  return invoke<ResultatConversion>("convertir_image", { source, dossier, format, qualite });
}

export const LIBELLES_FORMAT: Record<FormatImage, string> = {
  jpeg: "JPG",
  png: "PNG",
  webp: "WEBP",
};

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
