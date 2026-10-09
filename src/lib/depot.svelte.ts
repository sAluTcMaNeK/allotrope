// Aiguillage du glisser-déposer : la fenêtre reçoit les fichiers (App.svelte),
// et l'outil affiché (un outil PDF par exemple) peut s'inscrire pour les recevoir.

export const depot = $state<{
  /** Vrai pendant qu'un glisser-déposer survole la fenêtre. */
  survol: boolean;
  /** Fonction de l'outil affiché qui reçoit les fichiers déposés, s'il y en a une. */
  cible: ((chemins: string[]) => void) | null;
}>({ survol: false, cible: null });

/**
 * Inscrit `recevoir` comme destinataire des fichiers déposés tant que le composant est affiché.
 * À appeler dans `onMount` : renvoie la fonction de désinscription.
 */
export function recevoirDepots(recevoir: (chemins: string[]) => void): () => void {
  depot.cible = recevoir;
  return () => {
    if (depot.cible === recevoir) depot.cible = null;
  };
}
