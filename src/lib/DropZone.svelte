<!--
  Zone de dépôt de l'écran d'accueil.
  Le glisser-déposer lui-même est écouté au niveau de la fenêtre (App.svelte) :
  ce composant ne fait qu'afficher l'état `survol`.
-->
<script lang="ts">
  import { Folder } from "@lucide/svelte";
  import formes from "../../assets/allotrope-formes.svg";
  import { FAMILLES } from "./fichiers";

  interface Props {
    survol: boolean;
    onparcourir: () => void;
  }
  let { survol, onparcourir }: Props = $props();
</script>

<div class="zone" class:survol>
  <img src={formes} alt="" width="132" height="66" draggable="false" />
  <div class="textes">
    <div class="titre">{survol ? "Déposez pour ajouter" : "Glissez vos fichiers ici"}</div>
    <div class="aide">Allotrope détecte le format et vous propose les conversions possibles.</div>
  </div>
  <button class="parcourir" onclick={onparcourir}>
    <Folder size={18} strokeWidth={2.6} />
    Parcourir les fichiers
  </button>
  <div class="formats">
    {#each Object.values(FAMILLES) as famille (famille.libelle)}
      <div class="format">
        <span class="carre" style:background={famille.couleur}></span>
        {famille.formats}
      </div>
    {/each}
  </div>
</div>

<style>
  .zone {
    border: 3px dashed var(--azur);
    border-radius: 32px;
    background: var(--surface);
    padding: 48px 32px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 20px;
    text-align: center;
    transition:
      background-color var(--transition),
      border-color var(--transition);
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
    font-size: 26px;
    font-weight: 900;
  }

  .aide {
    font-size: 15px;
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

  .formats {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 10px;
    margin-top: 8px;
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
