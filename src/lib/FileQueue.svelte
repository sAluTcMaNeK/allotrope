<!-- Écran « File d'attente » : liste des fichiers reçus, destination et lancement de la conversion. -->
<script lang="ts">
  import { ArrowRight, Folder, Plus } from "@lucide/svelte";
  import FileCard from "./FileCard.svelte";
  import type { ElementFile, FormatImage } from "./fichiers";

  interface Props {
    elements: ElementFile[];
    survol: boolean;
    destination: string;
    enCours: boolean;
    onajouter: () => void;
    onretirer: (chemin: string) => void;
    onformat: (chemin: string, format: FormatImage) => void;
    onouvrir: (chemin: string) => void;
    onmodifierdestination: () => void;
    onconvertir: () => void;
  }
  let {
    elements,
    survol,
    destination,
    enCours,
    onajouter,
    onretirer,
    onformat,
    onouvrir,
    onmodifierdestination,
    onconvertir,
  }: Props = $props();

  const aConvertir = $derived(
    elements.filter((e) => e.format && (e.etat === "attente" || e.etat === "erreur")).length,
  );

  // « C:\Users\…\Documents\Allotrope » s'affiche « Documents › Allotrope »
  const destinationCourte = $derived(destination.split(/[\\/]/).filter(Boolean).slice(-2).join(" › "));

  const libelleBouton = $derived(
    enCours
      ? "Conversion en cours…"
      : aConvertir > 0
        ? `Convertir ${aConvertir} fichier${aConvertir > 1 ? "s" : ""}`
        : "Rien à convertir",
  );
</script>

<div class="file">
  <div class="defilement">
    <div class="entete">
      <h1>
        File d'attente
        <span class="compte">· {elements.length} fichier{elements.length > 1 ? "s" : ""}</span>
      </h1>
      <button class="ajouter" onclick={onajouter}>
        <Plus size={14} strokeWidth={3} />
        Ajouter des fichiers
      </button>
    </div>

    <div class="liste" class:survol>
      {#each elements as element (element.info.chemin)}
        <FileCard
          {element}
          onformat={(format) => onformat(element.info.chemin, format)}
          onretirer={() => onretirer(element.info.chemin)}
          onouvrir={() => element.resultat && onouvrir(element.resultat.chemin)}
        />
      {/each}
    </div>
  </div>

  <footer class="pied">
    <div class="destination">
      <span class="icone"><Folder size={20} strokeWidth={2.4} /></span>
      <span class="libelle">Enregistrer dans</span>
      <span class="chemin" title={destination}>{destinationCourte}</span>
      <button class="lien" onclick={onmodifierdestination}>Modifier</button>
    </div>
    <button class="principal" disabled={enCours || aConvertir === 0} onclick={onconvertir}>
      {libelleBouton}
      <ArrowRight size={18} strokeWidth={3} />
    </button>
  </footer>
</div>

<style>
  .file {
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .defilement {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 32px 40px 24px;
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

  .pied {
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
    padding: 16px 40px;
    border-top: 2px solid var(--ligne);
    background: var(--surface);
  }

  .destination {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
    font-size: 14px;
  }

  .icone {
    display: flex;
    color: var(--texte-secondaire);
  }

  .libelle {
    font-weight: 600;
    color: var(--texte-secondaire);
    white-space: nowrap;
  }

  .chemin {
    font-weight: 800;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .lien {
    border: none;
    background: none;
    padding: 0;
    min-height: 44px;
    color: var(--lien);
    font-size: 14px;
    font-weight: 800;
    text-decoration: underline;
    cursor: pointer;
  }

  .lien:hover {
    color: var(--givre);
  }

  /* Bouton « Principal » de la charte : le seul appel à l'action Braise de l'écran */
  .principal {
    height: 54px;
    padding: 0 30px;
    border-radius: 999px;
    border: 3px solid var(--givre);
    background: var(--braise);
    color: var(--abysse);
    font-size: 17px;
    font-weight: 900;
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
    transition: opacity var(--transition);
  }

  .principal:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>
