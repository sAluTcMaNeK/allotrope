// Mises à jour de l'application (plugin officiel « updater » de Tauri).
// Seule la recherche de mise à jour utilise internet : un petit fichier de version est lu sur GitHub.
// Aucun fichier de l'utilisateur n'est envoyé. Chaque mise à jour est signée : une version qui
// n'aurait pas été signée avec la clé d'Allotrope est refusée.
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export type EtatMiseAJour =
  | { etape: "inactif" }
  | { etape: "recherche" }
  | { etape: "a-jour" }
  | { etape: "disponible"; version: string; notes: string }
  | { etape: "telechargement"; version: string; pourcentage: number | null }
  | { etape: "erreur"; message: string };

export const miseAJour = $state<{ etat: EtatMiseAJour }>({ etat: { etape: "inactif" } });

/** Mise à jour trouvée par la dernière recherche. */
let trouvee: Update | null = null;

/** Recherche une nouvelle version. `discrete` : pas de message d'erreur (recherche au démarrage). */
export async function rechercherMiseAJour(discrete = false): Promise<void> {
  if (miseAJour.etat.etape === "recherche" || miseAJour.etat.etape === "telechargement") return;
  miseAJour.etat = { etape: "recherche" };
  try {
    trouvee = await check();
    miseAJour.etat = trouvee
      ? { etape: "disponible", version: trouvee.version, notes: trouvee.body ?? "" }
      : { etape: "a-jour" };
  } catch (erreur) {
    miseAJour.etat = discrete
      ? { etape: "inactif" }
      : { etape: "erreur", message: `Impossible de vérifier les mises à jour (${String(erreur)})` };
  }
}

/** Télécharge et installe la mise à jour trouvée, puis relance Allotrope. */
export async function installerMiseAJour(): Promise<void> {
  if (!trouvee) return;
  const version = trouvee.version;
  let total = 0;
  let recu = 0;
  miseAJour.etat = { etape: "telechargement", version, pourcentage: null };
  try {
    await trouvee.downloadAndInstall((evenement) => {
      if (evenement.event === "Started") total = evenement.data.contentLength ?? 0;
      if (evenement.event === "Progress") {
        recu += evenement.data.chunkLength;
        miseAJour.etat = {
          etape: "telechargement",
          version,
          pourcentage: total > 0 ? Math.min(100, Math.round((recu / total) * 100)) : null,
        };
      }
    });
    await relaunch();
  } catch (erreur) {
    miseAJour.etat = { etape: "erreur", message: `L'installation a échoué (${String(erreur)})` };
  }
}
