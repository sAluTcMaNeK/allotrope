// Lien avec les modules Rust `fichiers`, `conversion`, `compression` et `media` (src-tauri/src/) et outils d'affichage.
import { Channel, invoke } from "@tauri-apps/api/core";

/** Familles de fichiers de la barre latérale (les PDF ont leur propre page : Outils PDF). */
export type Famille = "image" | "video" | "audio";
export type FormatImage = "jpeg" | "png" | "webp" | "bmp" | "tiff" | "gif" | "ico" | "tga" | "qoi";
export type FormatMedia = "mp4" | "webm" | "mkv" | "mov" | "gif" | "mp3" | "m4a" | "ogg" | "opus" | "flac" | "wav";
/** Format de sortie : image, ou audio / vidéo (selon la famille du fichier source). */
export type FormatSortie = FormatImage | FormatMedia;
export type Mode = "convertir" | "compresser";

/** Miroir de la structure Rust `InfoFichier`. */
export interface InfoFichier {
  chemin: string;
  nom: string;
  extension: string;
  taille: number;
  famille: Famille | null;
  /** [largeur, hauteur] en pixels (images lisibles et vidéos). */
  dimensions: [number, number] | null;
  /** Durée en secondes (audio et vidéo). */
  duree: number | null;
  /** Formats de conversion proposés, le format par défaut en premier. */
  formatsSortie: FormatSortie[];
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
  format: FormatSortie | null;
  etat: Etat;
  /** Avancement en pourcentage (audio et vidéo). */
  progression?: number;
  resultat?: Resultat;
  erreur?: string;
}

/** Vrai pour les fichiers traités par FFmpeg (audio et vidéo). */
export function estMedia(info: InfoFichier): boolean {
  return info.famille === "video" || info.famille === "audio";
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
  /** Hauteur maximale des vidéos (720 pour du 720p) ; `null` = définition d'origine. */
  hauteurMax: number | null;
}

export const OPTIONS_COMPRESSION_DEFAUT: OptionsCompression = {
  type: "taux",
  taux: 40,
  tailleCible: 500,
  unite: "Ko",
  tailleMax: null,
  hauteurMax: null,
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

/** Choix proposés pour la définition maximale des vidéos ; `null` = définition d'origine. */
export const HAUTEURS_MAX: { valeur: number | null; libelle: string }[] = [
  { valeur: null, libelle: "Définition d'origine" },
  { valeur: 2160, libelle: "2160p (4K)" },
  { valeur: 1440, libelle: "1440p" },
  { valeur: 1080, libelle: "1080p (Full HD)" },
  { valeur: 720, libelle: "720p (HD)" },
  { valeur: 480, libelle: "480p" },
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

/** Opération audio / vidéo (miroir de l'énumération Rust `OperationMedia`). */
export type OperationMedia =
  | { type: "convertir"; format: FormatMedia }
  | { type: "compresser"; taux: number; tailleCible: number | null; hauteurMax: number | null };

/**
 * Convertit ou compresse un fichier audio / vidéo avec FFmpeg.
 * `onprogression` reçoit l'avancement (0 à 100) en temps réel, par un canal Tauri.
 */
export function traiterMedia(
  source: string,
  dossier: string,
  operation: OperationMedia,
  onprogression: (pourcentage: number) => void,
): Promise<Resultat> {
  const progression = new Channel<{ pourcentage: number }>();
  progression.onmessage = (message) => onprogression(message.pourcentage);
  return invoke<Resultat>("traiter_media", { source, dossier, operation, progression });
}

/** Arrête le traitement audio / vidéo en cours de `source`. */
export function annulerMedia(source: string): Promise<void> {
  return invoke("annuler_media", { source });
}

/** Opération FFmpeg correspondant au mode et aux réglages choisis. */
export function operationMedia(element: ElementFile, mode: Mode, options: OptionsCompression): OperationMedia {
  if (mode === "convertir") return { type: "convertir", format: element.format as FormatMedia };
  return {
    type: "compresser",
    taux: options.taux,
    tailleCible: options.type === "taille" ? octetsCible(options) : null,
    hauteurMax: options.hauteurMax,
  };
}

// --- Présentation ---

export const LIBELLES_FORMAT: Record<FormatSortie, string> = {
  mp4: "MP4",
  webm: "WEBM",
  mkv: "MKV",
  mov: "MOV",
  mp3: "MP3",
  m4a: "M4A",
  ogg: "OGG",
  opus: "OPUS",
  flac: "FLAC",
  wav: "WAV",
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
    cibles: (["jpeg", "png", "webp", "tiff", "bmp", "gif", "ico", "tga", "qoi"] as FormatImage[]).map((id) => ({
      id,
      libelle: LIBELLES_FORMAT[id],
    })),
    conversion: null,
    compression: null,
  },
  video: {
    libelle: "Vidéo",
    nom: "vidéos",
    couleur: "var(--famille-video)",
    texte: "var(--blanc)",
    formats: "MP4 · MOV · MKV · AVI · GIF…",
    // Miroir de EXTENSIONS_VIDEO (media.rs)
    extensions: ["mp4", "m4v", "mov", "mkv", "webm", "avi", "wmv", "flv", "mpg", "mpeg", "ts", "mts", "m2ts", "3gp", "ogv", "gif"],
    cibles: [
      { id: "mp4", libelle: "MP4" },
      { id: "webm", libelle: "WEBM" },
      { id: "mkv", libelle: "MKV" },
      { id: "mov", libelle: "MOV" },
      { id: "gif", libelle: "GIF" },
      { id: "mp3", libelle: "MP3 (son)" },
      { id: "m4a", libelle: "M4A (son)" },
      { id: "wav", libelle: "WAV (son)" },
    ],
    conversion: null,
    compression: null,
  },
  audio: {
    libelle: "Audio",
    nom: "fichiers audio",
    couleur: "var(--famille-audio)",
    texte: "var(--abysse)",
    formats: "MP3 · WAV · FLAC · M4A · OGG…",
    // Miroir de EXTENSIONS_AUDIO (media.rs)
    extensions: ["mp3", "wav", "flac", "ogg", "oga", "m4a", "aac", "opus", "wma", "aiff", "aif", "ac3", "amr", "mka"],
    cibles: [
      { id: "mp3", libelle: "MP3" },
      { id: "m4a", libelle: "M4A" },
      { id: "ogg", libelle: "OGG" },
      { id: "opus", libelle: "OPUS" },
      { id: "flac", libelle: "FLAC" },
      { id: "wav", libelle: "WAV" },
    ],
    conversion: null,
    compression: null,
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
