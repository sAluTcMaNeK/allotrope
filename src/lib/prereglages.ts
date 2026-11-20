// Préréglages : un clic dans la barre latérale ouvre la bonne page avec l'outil et les réglages enregistrés.
// Aucun préréglage n'est fourni avec l'application : l'utilisateur crée les siens (conservés dans les réglages).
import type { Famille, FormatSortie, Mode, OptionsCompression } from "./fichiers";

export interface Prereglage {
  id: string;
  nom: string;
  /** Page ouverte : une famille de fichiers. */
  cible: Famille;
  mode: Mode;
  /** Format de sortie (conversion). */
  format?: FormatSortie;
  /** Réglages complets (compression). */
  compression?: OptionsCompression;
}

/** Identifiant unique pour un nouveau préréglage. */
export function nouvelIdentifiant(): string {
  return `perso-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`;
}
