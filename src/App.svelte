<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { documentDir, join } from "@tauri-apps/api/path";
  import { open } from "@tauri-apps/plugin-dialog";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import TitleBar from "./lib/TitleBar.svelte";
  import Sidebar from "./lib/Sidebar.svelte";
  import DropZone from "./lib/DropZone.svelte";
  import FileQueue from "./lib/FileQueue.svelte";
  import {
    analyserFichiers,
    convertirImage,
    type ElementFile,
    type FormatImage,
  } from "./lib/fichiers";

  /** Qualité JPEG par défaut (le panneau d'options la rendra réglable). */
  const QUALITE_JPEG = 85;
  /** Nombre maximal de conversions simultanées. */
  const CONVERSIONS_PARALLELES = Math.max(1, Math.min(4, navigator.hardwareConcurrency || 2));

  let elements = $state<ElementFile[]>([]);
  // Vrai pendant qu'un glisser-déposer survole la fenêtre
  let survol = $state(false);
  let destination = $state("");
  let enCours = $state(false);

  async function ajouter(chemins: string[]) {
    if (chemins.length === 0) return;
    const infos = await analyserFichiers(chemins);
    for (const info of infos) {
      // Un même fichier n'apparaît qu'une fois dans la file
      if (!elements.some((e) => e.info.chemin === info.chemin)) {
        elements.push({ info, format: info.formatsSortie[0] ?? null, etat: "attente" });
      }
    }
  }

  function retirer(chemin: string) {
    elements = elements.filter((e) => e.info.chemin !== chemin);
  }

  function changerFormat(chemin: string, format: FormatImage) {
    const element = elements.find((e) => e.info.chemin === chemin);
    if (!element) return;
    element.format = format;
    // Changer le format d'un fichier déjà converti permet de le reconvertir
    element.etat = "attente";
    element.resultat = undefined;
    element.erreur = undefined;
  }

  async function parcourir() {
    const choix = await open({ multiple: true, title: "Choisir les fichiers à convertir" });
    if (choix) await ajouter(choix);
  }

  async function modifierDestination() {
    const choix = await open({
      directory: true,
      defaultPath: destination,
      title: "Choisir le dossier de destination",
    });
    if (choix) destination = choix;
  }

  async function convertir() {
    const aFaire = elements.filter(
      (e) => e.format && (e.etat === "attente" || e.etat === "erreur"),
    );
    enCours = true;

    // Chaque « travailleur » prend le fichier suivant tant qu'il en reste
    let suivant = 0;
    const travailleur = async () => {
      while (suivant < aFaire.length) {
        const element = aFaire[suivant++];
        // Le fichier a pu être retiré de la file entre-temps
        if (!elements.some((e) => e.info.chemin === element.info.chemin)) continue;
        element.etat = "encours";
        element.erreur = undefined;
        try {
          element.resultat = await convertirImage(
            element.info.chemin,
            destination,
            element.format!,
            QUALITE_JPEG,
          );
          element.etat = "termine";
        } catch (erreur) {
          element.erreur = String(erreur);
          element.etat = "erreur";
        }
      }
    };
    await Promise.all(
      Array.from({ length: Math.min(CONVERSIONS_PARALLELES, aFaire.length) }, travailleur),
    );
    enCours = false;
  }

  onMount(() => {
    // Destination par défaut : Documents\Allotrope (créé à la première conversion)
    documentDir()
      .then((documents) => join(documents, "Allotrope"))
      .then((chemin) => (destination = chemin));

    // Tauri intercepte le glisser-déposer au niveau de la fenêtre et fournit les chemins complets
    const arret = getCurrentWebview().onDragDropEvent(({ payload }) => {
      if (payload.type === "enter" || payload.type === "over") {
        survol = true;
      } else if (payload.type === "leave") {
        survol = false;
      } else if (payload.type === "drop") {
        survol = false;
        ajouter(payload.paths);
      }
    });
    return () => {
      arret.then((desabonner) => desabonner());
    };
  });
</script>

<div class="application">
  <TitleBar />

  <div class="corps">
    <Sidebar />

    <main class="contenu">
      {#if elements.length === 0}
        <div class="accueil">
          <h1>Que voulez-vous convertir ?</h1>
          <DropZone {survol} onparcourir={parcourir} />
        </div>
      {:else}
        <FileQueue
          {elements}
          {survol}
          {destination}
          {enCours}
          onajouter={parcourir}
          onretirer={retirer}
          onformat={changerFormat}
          onouvrir={(chemin) => revealItemInDir(chemin)}
          onmodifierdestination={modifierDestination}
          onconvertir={convertir}
        />
      {/if}
    </main>
  </div>
</div>

<style>
  .application {
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .corps {
    flex: 1;
    min-height: 0;
    display: flex;
  }

  .contenu {
    flex: 1;
    min-width: 0;
    min-height: 0;
  }

  .accueil {
    height: 100%;
    overflow-y: auto;
    padding: 32px 40px 40px;
    display: flex;
    flex-direction: column;
    gap: 28px;
  }

  h1 {
    margin: 0;
    font-size: 30px;
    font-weight: 900;
    letter-spacing: -0.3px;
  }
</style>
