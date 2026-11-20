<!-- File d'attente : liste des fichiers reçus. -->
<script lang="ts">
  import { Plus } from "@lucide/svelte";
  import FileCard from "./FileCard.svelte";
  import Apercu from "./Apercu.svelte";
  import type { ElementFile, FormatSortie, Mode, TypeCompression } from "./fichiers";

  interface Props {
    elements: ElementFile[];
    mode: Mode;
    typeCompression: TypeCompression;
    survol: boolean;
    onajouter: () => void;
    onretirer: (chemin: string) => void;
    onformat: (chemin: string, format: FormatSortie) => void;
    onannuler: (chemin: string) => void;
    onouvrir: (chemin: string) => void;
  }
  let { elements, mode, typeCompression, survol, onajouter, onretirer, onformat, onouvrir, onannuler }: Props =
    $props();

  /** Chemin du fichier dont l'aperçu avant / après est ouvert. */
  let enApercu = $state<string | null>(null);
  const apercu = $derived(elements.find((e) => e.info.chemin === enApercu && e.etat === "termine" && e.resultat?.chemin));
</script>

<section class="file" aria-label="File d'attente">
  <div class="entete">
    <h2>
      File d'attente
      <span class="compte">· {elements.length} fichier{elements.length > 1 ? "s" : ""}</span>
    </h2>
    <button class="ajouter" onclick={onajouter}>
      <Plus size={14} strokeWidth={3} />
      Ajouter des fichiers
    </button>
  </div>

  <div class="liste" class:survol>
    {#each elements as element (element.info.chemin)}
      <FileCard
        {element}
        {mode}
        {typeCompression}
        onformat={(format) => onformat(element.info.chemin, format)}
        onretirer={() => onretirer(element.info.chemin)}
        onannuler={() => onannuler(element.info.chemin)}
        onouvrir={() => element.resultat?.chemin && onouvrir(element.resultat.chemin)}
        onapercu={() => (enApercu = element.info.chemin)}
      />
    {/each}
  </div>

  {#if apercu}
    <Apercu element={apercu} {mode} {onouvrir} onfermer={() => (enApercu = null)} />
  {/if}
</section>

<style>
  .file {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .entete {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
  }

  h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 800;
  }

  .compte {
    color: var(--texte-secondaire);
    font-weight: 700;
  }

  /* Bouton « Secondaire » de la charte */
  .ajouter {
    height: 44px;
    padding: 0 18px;
    border-radius: 999px;
    border: 2px solid var(--ligne-forte);
    background: transparent;
    color: var(--givre);
    font-size: 14px;
    font-weight: 800;
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    transition: border-color var(--transition);
  }

  .ajouter:hover {
    border-color: var(--givre);
  }

  /* La liste reprend l'état « survol » de la zone de dépôt pendant un glisser-déposer */
  .liste {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 8px;
    margin: -8px;
    border-radius: 30px;
    border: 3px solid transparent;
    transition:
      background-color var(--transition),
      border-color var(--transition);
  }

  .liste.survol {
    border-color: var(--givre);
    background: var(--surface-active);
  }
</style>
