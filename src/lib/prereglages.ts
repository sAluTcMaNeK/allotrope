// Préréglages : un clic dans la barre latérale ouvre la bonne page avec l'outil et les réglages adaptés.
// Les préréglages intégrés sont définis ici ; ceux de l'utilisateur sont conservés dans les réglages.
import {
  OPTIONS_COMPRESSION_DEFAUT,
  type Famille,
  type FormatSortie,
  type Mode,
  type OptionsCompression,
} from "./fichiers";

export interface Prereglage {
  id: string;
  nom: string;
  /** Page ouverte : une famille de fichiers, ou les Outils PDF (compression). */
  cible: Famille | "pdf";
  mode: Mode;
  /** Format de sortie (conversion). */
  format?: FormatSortie;
  /** Réglages complets (compression). */
  compression?: OptionsCompression;
  /** Vrai pour les préréglages fournis avec l'application (non supprimables). */
  integre?: boolean;
}

/** Réglages de compression complets à partir des valeurs qui diffèrent des valeurs par défaut. */
function compression(modification: Partial<OptionsCompression>): OptionsCompression {
  return { ...OPTIONS_COMPRESSION_DEFAUT, ...modification };
}

export const PREREGLAGES_INTEGRES: Prereglage[] = [
  {
    id: "image-web",
    nom: "Image pour le web",
    cible: "image",
    mode: "compresser",
    compression: compression({ type: "taux", taux: 50, tailleMax: 1920 }),
    integre: true,
  },
  {
    id: "video-email",
    nom: "Vidéo légère pour e-mail",
    cible: "video",
    mode: "compresser",
    // Les messageries limitent les pièces jointes à 20-25 Mo
    compression: compression({ type: "taille", tailleCible: 20, unite: "Mo", hauteurMax: 720 }),
    integre: true,
  },
  {
    id: "pdf-compresse",
    nom: "PDF compressé",
    cible: "pdf",
    mode: "compresser",
    compression: compression({ type: "taux", taux: 50, tailleMax: 1920 }),
    integre: true,
  },
  {
    id: "logo-svg",
    nom: "Logo en SVG",
    cible: "image",
    mode: "convertir",
    format: "svg",
    integre: true,
  },
  {
    id: "son-mp3",
    nom: "Extraire le son (MP3)",
    cible: "video",
    mode: "convertir",
    format: "mp3",
    integre: true,
  },
];

/** Identifiant unique pour un nouveau préréglage. */
export function nouvelIdentifiant(): string {
  return `perso-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`;
}
