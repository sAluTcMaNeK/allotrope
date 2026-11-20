// Réglages de l'application, conservés d'une session à l'autre.
// Le suffixe `.svelte.ts` permet d'utiliser les runes ($state, $effect) en dehors d'un composant.
import { OPTIONS_COMPRESSION_DEFAUT, type OptionsCompression } from "./fichiers";
import type { Prereglage } from "./prereglages";

export type TypeDestination = "documents" | "origine" | "personnalise";

export interface Reglages {
  /** Où enregistrer les fichiers produits. */
  destination: TypeDestination;
  /** Dossier choisi quand `destination` vaut « personnalise ». */
  dossierPersonnalise: string;
  /** Qualité JPEG / WebP des conversions (50 à 100). */
  qualite: number;
  /** Réglages de compression proposés par défaut. */
  compression: OptionsCompression;
  /** Préréglages créés par l'utilisateur. */
  prereglages: Prereglage[];
  /** Rechercher une nouvelle version au démarrage. */
  miseAJourAuto: boolean;
}

const CLE = "allotrope.reglages";

const DEFAUTS: Reglages = {
  destination: "documents",
  dossierPersonnalise: "",
  qualite: 85,
  compression: { ...OPTIONS_COMPRESSION_DEFAUT },
  prereglages: [],
  miseAJourAuto: true,
};

function charger(): Reglages {
  try {
    const texte = localStorage.getItem(CLE);
    if (!texte) return structuredClone(DEFAUTS);
    // Les réglages enregistrés complètent les valeurs par défaut (utile si de nouveaux réglages apparaissent)
    const lus = JSON.parse(texte);
    return {
      destination: lus.destination ?? DEFAUTS.destination,
      dossierPersonnalise: lus.dossierPersonnalise ?? DEFAUTS.dossierPersonnalise,
      qualite: lus.qualite ?? DEFAUTS.qualite,
      compression: { ...DEFAUTS.compression, ...lus.compression },
      prereglages: Array.isArray(lus.prereglages) ? lus.prereglages : [],
      miseAJourAuto: lus.miseAJourAuto ?? DEFAUTS.miseAJourAuto,
    };
  } catch {
    return structuredClone(DEFAUTS);
  }
}

export const reglages = $state<Reglages>(charger());

// Enregistre automatiquement à chaque modification
$effect.root(() => {
  $effect(() => {
    try {
      localStorage.setItem(CLE, JSON.stringify(reglages));
    } catch {
      // Stockage indisponible : les réglages restent valables pour la session en cours
    }
  });
});
