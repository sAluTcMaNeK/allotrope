<!-- Écran « File d'attente » : liste des fichiers reçus. -->
<script lang="ts">
  import { Plus } from "@lucide/svelte";
  import FileCard from "./FileCard.svelte";
  import type { InfoFichier } from "./fichiers";

  interface Props {
    fichiers: InfoFichier[];
    survol: boolean;
    onajouter: () => void;
    onretirer: (chemin: string) => void;
  }
  let { fichiers, survol, onajouter, onretirer }: Props = $props();
</script>

<div class="file">
  <div class="entete">
    <h1>
      File d'attente
      <span class="compte">· {fichiers.length} fichier{fichiers.length > 1 ? "s" : ""}</span>
    </h1>
    <button class="ajouter" onclick={onajouter}>
      <Plus size={14} strokeWidth={3} />
      Ajouter des fichiers
    </button>
  </div>

  <div class="liste" class:survol>
    {#each fichiers as fichier (fichier.chemin)}
      <FileCard {fichier} onretirer={() => onretirer(fichier.chemin)} />
    {/each}
  </div>
</div>

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
    margin-bottom: 6px;
  }

  h1 {
    margin: 0;
    font-size: 28px;
    font-weight: 900;
    letter-spacing: -0.3px;
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
