// Liste des outils PDF, partagée par la page « Outils PDF » et le menu « Outil PDF » des cartes
// de fichiers de l'accueil.
import type { Component } from "svelte";
import { ArrowDownUp, FileText, FileType, ImageDown, ImagePlus, Images, Layers, Shrink, Trash } from "@lucide/svelte";

export type IdOutil =
  | "fusionner"
  | "reorganiser"
  | "supprimer"
  | "images"
  | "compresser"
  | "extraire-images"
  | "en-images"
  | "pdf-texte"
  | "word";

export interface Outil {
  id: IdOutil;
  titre: string;
  description: string;
  icone: Component<{ size?: number; strokeWidth?: number }>;
  /** Fichiers que l'outil reçoit : des PDF, des images ou des documents (Word, ODT, RTF). */
  entree: "pdf" | "image" | "document";
  /** Outil pas encore disponible : la cartouche est affichée mais inactive. */
  bientot?: string;
}

export const OUTILS_PDF: Outil[] = [
  {
    id: "fusionner",
    titre: "Fusionner des PDF",
    description: "Assemblez plusieurs PDF en un seul document.",
    icone: Layers,
    entree: "pdf",
  },
  {
    id: "reorganiser",
    titre: "Réorganiser les pages",
    description: "Changez l'ordre des pages et faites-les pivoter.",
    icone: ArrowDownUp,
    entree: "pdf",
  },
  {
    id: "supprimer",
    titre: "Supprimer des pages",
    description: "Retirez les pages inutiles d'un PDF.",
    icone: Trash,
    entree: "pdf",
  },
  {
    id: "images",
    titre: "Images en PDF",
    description: "Une photo ou un scan par page, dans l'ordre choisi.",
    icone: ImagePlus,
    entree: "image",
  },
  {
    id: "compresser",
    titre: "Compresser un PDF",
    description: "Allégez un PDF pour l'envoyer par e-mail.",
    icone: Shrink,
    entree: "pdf",
  },
  {
    id: "extraire-images",
    titre: "Extraire les images",
    description: "Récupérez les photos d'un PDF, une à une ou toutes en ZIP.",
    icone: ImageDown,
    entree: "pdf",
  },
  {
    id: "en-images",
    titre: "PDF en images",
    description: "Exportez chaque page en PNG, JPG ou WEBP.",
    icone: Images,
    entree: "pdf",
  },
  {
    id: "pdf-texte",
    titre: "PDF en Word / ODT",
    description: "Rendez un PDF modifiable : DOCX ou ODT.",
    icone: FileType,
    entree: "pdf",
  },
  {
    id: "word",
    titre: "Word en PDF",
    description: "Transformez un document Word ou ODT en PDF.",
    icone: FileText,
    entree: "document",
  },
];

/** Demande d'ouverture d'un outil PDF avec des fichiers déjà chargés (depuis l'accueil). */
export interface DemandeOutilPdf {
  outil: IdOutil;
  fichiers: string[];
}
