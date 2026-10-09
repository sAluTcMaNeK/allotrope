<!-- Zone de dépôt des outils PDF (même style que la zone de dépôt principale, contour Cobalt). -->
<script lang="ts">
  import { Folder } from "@lucide/svelte";
  import formes from "../../../assets/allotrope-formes.svg";
  import { depot } from "../depot.svelte";

  interface Props {
    titre: string;
    aide: string;
    /** Formats acceptés, affichés en pastille (« PDF », « JPG · PNG · WEBP… »). */
    formats: string;
    onparcourir: () => void;
    /** Version réduite, sous une liste déjà remplie. */
    compacte?: boolean;
    /** Couleur de la pastille des formats (famille de fichiers acceptée). */
    couleur?: string;
  }
  let { titre, aide, formats, onparcourir, compacte = false, couleur = "var(--famille-document)" }: Props =
    $props();
</script>

<div class="zone" class:survol={depot.survol} class:compacte>
  {#if !compacte}
    <img src={formes} alt="" width="132" height="66" draggable="false" />
  {/if}
  <div class="textes">
    <div class="titre">{depot.survol ? "Déposez pour ajouter" : titre}</div>
    <div class="aide">{aide}</div>
  </div>
  <button class="parcourir" onclick={onparcourir}>
    <Folder size={18} strokeWidth={2.6} />
    Parcourir les fichiers
  </button>
  {#if !compacte}
    <div class="format"><span class="carre" style:background={couleur}></span>{formats}</div>
  {/if}
</div>

<style>
  .zone {
    border: 3px dashed var(--mode-convertir);
    border-radius: 32px;
    background: var(--surface);
    padding: 40px 32px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 18px;
    text-align: center;
    transition:
      background-color var(--transition),
      border-color var(--transition);
  }

  .zone.compacte {
    flex-direction: row;
    flex-wrap: wrap;
    justify-content: space-between;
    padding: 20px 24px;
    text-align: left;
  }

  /* Pendant un glisser-déposer : bordure pleine Givre et fond actif */
  .zone.survol {
    border-style: solid;
    border-color: var(--givre);
    background: var(--surface-active);
  }

  img {
    pointer-events: none;
  }

  .textes {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .titre {
    font-size: 24px;
    font-weight: 900;
  }

  .compacte .titre {
    font-size: 17px;
  }

  .aide {
    font-size: 14px;
    font-weight: 600;
    color: var(--texte-secondaire);
  }

  /* Bouton « Marque » de la charte */
  .parcourir {
    height: 52px;
    padding: 0 28px;
    border-radius: 999px;
    border: 3px solid var(--givre);
    background: var(--cobalt);
    color: var(--blanc);
    font-size: 16px;
    font-weight: 800;
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
    transition: background-color var(--transition);
  }

  .parcourir:hover {
    background: var(--azur);
    color: var(--abysse);
  }

  .format {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    border-radius: 999px;
    border: 2px solid var(--ligne);
    background: var(--abysse);
    font-size: 13px;
    font-weight: 700;
  }

  .carre {
    width: 10px;
    height: 10px;
    border-radius: 3px;
  }
</style>
