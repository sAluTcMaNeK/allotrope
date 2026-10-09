// Dossier où enregistrer les fichiers produits, selon les Paramètres (partagé par tous les outils).
import { dirname, documentDir, join } from "@tauri-apps/api/path";
import { open } from "@tauri-apps/plugin-dialog";
import { reglages } from "./reglages.svelte";

export const destination = $state({
  /** Chemin complet de Documents\Allotrope, connu après `initialiserDestination()`. */
  documents: "",
});

/** À appeler une fois au démarrage. */
export async function initialiserDestination() {
  destination.documents = await join(await documentDir(), "Allotrope");
}

/** Destination lisible : « Documents › Allotrope », « À côté des originaux », « Images › Export ». */
export function libelleDestination(): string {
  if (reglages.destination === "documents") return "Documents › Allotrope";
  if (reglages.destination === "origine") return "À côté des originaux";
  return reglages.dossierPersonnalise.split(/[\\/]/).filter(Boolean).slice(-2).join(" › ");
}

/** Dossier où enregistrer le fichier produit à partir de `source`. */
export async function dossierPour(source: string): Promise<string> {
  if (reglages.destination === "origine") return dirname(source);
  if (reglages.destination === "personnalise" && reglages.dossierPersonnalise) {
    return reglages.dossierPersonnalise;
  }
  return destination.documents;
}

/** Fait choisir un autre dossier (lien « Modifier » des pieds de page). */
export async function choisirDestination() {
  const choix = await open({
    directory: true,
    defaultPath:
      reglages.destination === "personnalise" ? reglages.dossierPersonnalise : destination.documents,
    title: "Choisir le dossier de destination",
  });
  if (!choix) return;
  reglages.dossierPersonnalise = choix;
  reglages.destination = "personnalise";
}
