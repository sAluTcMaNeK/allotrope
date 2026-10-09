<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { open } from "@tauri-apps/plugin-dialog";
  import TitleBar from "./lib/TitleBar.svelte";
  import Sidebar from "./lib/Sidebar.svelte";
  import DropZone from "./lib/DropZone.svelte";
  import FileQueue from "./lib/FileQueue.svelte";
  import { analyserFichiers, type InfoFichier } from "./lib/fichiers";

  let fichiers = $state<InfoFichier[]>([]);
  // Vrai pendant qu'un glisser-déposer survole la fenêtre
  let survol = $state(false);

  async function ajouter(chemins: string[]) {
    if (chemins.length === 0) return;
    const infos = await analyserFichiers(chemins);
    for (const info of infos) {
      // Un même fichier n'apparaît qu'une fois dans la file
      if (!fichiers.some((f) => f.chemin === info.chemin)) {
        fichiers.push(info);
      }
    }
  }

  function retirer(chemin: string) {
    fichiers = fichiers.filter((f) => f.chemin !== chemin);
  }

  async function parcourir() {
    const choix = await open({ multiple: true, title: "Choisir les fichiers à convertir" });
    if (choix) await ajouter(choix);
  }

  onMount(() => {
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
      {#if fichiers.length === 0}
        <h1>Que voulez-vous convertir ?</h1>
        <DropZone {survol} onparcourir={parcourir} />
      {:else}
        <FileQueue {fichiers} {survol} onajouter={parcourir} onretirer={retirer} />
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
