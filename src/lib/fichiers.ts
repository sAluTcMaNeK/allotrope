// Lien avec les modules Rust `fichiers`, `conversion` et `compression` (src-tauri/src/) et outils d'affichage.
import { invoke } from "@tauri-apps/api/core";

export type Famille = "image" | "video" | "audio" | "document";
export type FormatImage = "jpeg" | "png" | "webp" | "bmp" | "tiff" | "gif" | "ico" | "tga" | "qoi";
export type Mode = "convertir" | "compresser";

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
  compressible: boolean;
}

/** Résultat d'une conversion ou d'une compression. `chemin` est `null` si aucun fichier n'a été créé. */
export interface Resultat {
  chemin: string | null;
  taille: number;
}

export type Etat = "attente" | "encours" | "termine" | "erreur";

/** Un fichier de la file d'attente et l'avancement de son traitement. */
export interface ElementFile {
  info: InfoFichier;
  /** Format de conversion choisi ; `null` si aucun format n'est (encore) choisi. */
  format: FormatImage | null;
  etat: Etat;
  resultat?: Resultat;
  erreur?: string;
}

/** Vrai si l'élément peut être traité dans le mode donné. */
export function traitable(element: ElementFile, mode: Mode): boolean {
  return mode === "convertir" ? element.format !== null : element.info.compressible;
}

// --- Compression ---

/** Réglage de la compression : par taux, ou par taille de fichier cible. */
export type TypeCompression = "taux" | "taille";
export type Unite = "Ko" | "Mo";

export interface OptionsCompression {
  type: TypeCompression;
  /** Taux de compression, de 5 (qualité maximale) à 95 (fichier le plus léger). */
  taux: number;
  /** Taille cible par fichier, exprimée dans `unite`. */
  tailleCible: number;
  unite: Unite;
  /** Plus grand côté de l'image, en pixels ; `null` = taille d'origine. */
  tailleMax: number | null;
}

export const OPTIONS_COMPRESSION_DEFAUT: OptionsCompression = {
  type: "taux",
  taux: 40,
  tailleCible: 500,
  unite: "Ko",
  tailleMax: null,
};

/** Taille cible en octets. */
export function octetsCible(options: OptionsCompression): number {
  return Math.round(options.tailleCible * (options.unite === "Mo" ? 1024 * 1024 : 1024));
}

/** Choix proposés pour la taille maximale (plus grand côté, en pixels) ; `null` = taille d'origine. */
export const TAILLES_MAX: { valeur: number | null; libelle: string }[] = [
  { valeur: null, libelle: "Taille d'origine" },
  { valeur: 3840, libelle: "3840 px (4K)" },
  { valeur: 2560, libelle: "2560 px" },
  { valeur: 1920, libelle: "1920 px (Full HD)" },
  { valeur: 1280, libelle: "1280 px" },
  { valeur: 800, libelle: "800 px" },
];

// --- Appels au cœur Rust ---

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
): Promise<Resultat> {
  return invoke<Resultat>("convertir_image", { source, dossier, format, qualite });
}

/** Compresse une image en gardant son format. */
export function compresserImage(
  source: string,
  dossier: string,
  options: OptionsCompression,
): Promise<Resultat> {
  return invoke<Resultat>("compresser_image", {
    source,
    dossier,
    taux: options.taux,
    tailleCible: options.type === "taille" ? octetsCible(options) : null,
    tailleMax: options.tailleMax,
  });
}

// --- Présentation ---

export const LIBELLES_FORMAT: Record<FormatImage, string> = {
  jpeg: "JPG",
  png: "PNG",
  webp: "WEBP",
  tiff: "TIFF",
  bmp: "BMP",
  gif: "GIF",
  ico: "ICO",
  tga: "TGA",
  qoi: "QOI",
};

/**
 * Présentation de chaque famille (voir DESIGN.md, « Couleur par famille de fichiers »).
 * - `extensions` sert uniquement au filtre du sélecteur de fichiers ; la détection fait foi côté Rust ;
 * - `cibles` : formats proposés dans l'onglet Conversion de la page de la famille ;
 * - `conversion` / `compression` : message si l'outil n'est pas encore disponible pour cette famille.
 */
export const FAMILLES: Record<
  Famille,
  {
    libelle: string;
    nom: string;
    couleur: string;
    texte: string;
    formats: string;
    extensions: string[];
    cibles: { id: string; libelle: string }[];
    conversion: string | null;
    compression: string | null;
  }
> = {
  image: {
    libelle: "Images",
    nom: "images",
    couleur: "var(--famille-image)",
    texte: "var(--abysse)",
    formats: "JPG · PNG · WEBP · SVG · TIFF · JXL…",
    // Miroir de EXTENSIONS_LISIBLES (conversion.rs), plus HEIC / AVIF reconnus mais pas encore lisibles
    extensions: [
      "jpg", "jpeg", "jfif", "jpe", "png", "apng", "webp", "bmp", "dib", "tif", "tiff", "ico", "tga",
      "qoi", "pbm", "pgm", "ppm", "pnm", "pam", "hdr", "exr", "dds", "ff", "svg", "svgz", "jxl",
      "heic", "heif", "avif",
    ],
    cibles: (Object.entries(LIBELLES_FORMAT) as [FormatImage, string][]).map(([id, libelle]) => ({
      id,
      libelle,
    })),
    conversion: null,
    compression: null,
  },
  video: {
    libelle: "Vidéo",
    nom: "vidéos",
    couleur: "var(--famille-video)",
    texte: "var(--blanc)",
    formats: "MP4 · MOV · MKV · GIF",
    extensions: ["mp4", "mov", "mkv", "avi", "webm", "m4v", "gif"],
    cibles: [
      { id: "mp4", libelle: "MP4" },
      { id: "webm", libelle: "WEBM" },
      { id: "gif", libelle: "GIF" },
      { id: "mp3", libelle: "MP3 (son)" },
    ],
    conversion: "La conversion des vidéos arrive avec le module vidéo (FFmpeg).",
    compression: "La compression des vidéos arrive avec le module vidéo (FFmpeg).",
  },
  audio: {
    libelle: "Audio",
    nom: "fichiers audio",
    couleur: "var(--famille-audio)",
    texte: "var(--abysse)",
    formats: "MP3 · WAV · FLAC",
    extensions: ["mp3", "wav", "flac", "ogg", "m4a", "aac", "opus"],
    cibles: [
      { id: "mp3", libelle: "MP3" },
      { id: "wav", libelle: "WAV" },
      { id: "flac", libelle: "FLAC" },
      { id: "ogg", libelle: "OGG" },
    ],
    conversion: "La conversion audio arrive avec le module vidéo et audio (FFmpeg).",
    compression: "La compression audio arrive avec le module vidéo et audio (FFmpeg).",
  },
  document: {
    libelle: "Documents",
    nom: "documents",
    couleur: "var(--famille-document)",
    texte: "var(--abysse)",
    formats: "DOCX · PDF · ODT",
    extensions: ["pdf", "docx", "doc", "odt", "rtf"],
    cibles: [{ id: "pdf", libelle: "PDF" }],
    conversion: "La conversion des documents arrive avec le module documents.",
    compression: "La compression des PDF arrive avec le module documents.",
  },
};

export const LISTE_FAMILLES = Object.entries(FAMILLES) as [Famille, (typeof FAMILLES)[Famille]][];

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
