<!--
  Outil « Extraire les images » : liste les images d'un PDF avec leurs miniatures ;
  on choisit celles à enregistrer (dans un dossier), ou on télécharge tout dans une archive ZIP.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import { open } from "@tauri-apps/plugin-dialog";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { Check, CheckCheck, FileArchive, Folder, ImageOff, LoaderCircle } from "@lucide/svelte";
  import ToolLayout from "./ToolLayout.svelte";
  import DepotPdf from "./DepotPdf.svelte";
  import QueueFooter from "../QueueFooter.svelte";
  import { recevoirDepots } from "../depot.svelte";
  import { dossierPour } from "../destination.svelte";
  import { analyserImagesPdf, extraireImagesPdf, type ImageTrouvee, type ResultatExtraction } from "./pdf";

  interface Props {
    onretour: () => void;
  }
  let { onretour }: Props = $props();

  let chemin = $state<string | null>(null);
  let images = $state<ImageTrouvee[]>([]);
  /** Identifiants des images choisies. */
  const selection = new SvelteSet<string>();
  let chargement = $state(false);
  let enCours = $state(false);
  let resultat = $state<(ResultatExtraction & { zip: boolean }) | null>(null);
  let erreur = $state<string | null>(null);

  const nom = $derived(chemin ? (chemin.split(/[\\/]/).pop() ?? "") : "");
  const toutChoisi = $derived(images.length > 0 && selection.size === images.length);

  async function charger(nouveau: string) {
    erreur = null;
    resultat = null;
    chargement = true;
    chemin = nouveau;
    images = [];
    selection.clear();
    try {
      images = await analyserImagesPdf(nouveau);
    } catch (e) {
      erreur = String(e);
      chemin = null;
    } finally {
      chargement = false;
    }
  }

  function recevoir(chemins: string[]) {
    const premier = chemins.find((c) => c.toLowerCase().endsWith(".pdf"));
    if (premier) charger(premier);
    else erreur = "Déposez un fichier PDF.";
  }

  async function parcourir() {
    const choix = await open({ title: "Choisir un PDF", filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (choix) await charger(choix);
  }

  function basculer(id: string) {
    if (selection.has(id)) selection.delete(id);
    else selection.add(id);
  }

  function toutBasculer() {
    if (toutChoisi) selection.clear();
    else images.forEach((i) => selection.add(i.id));
  }

  /** Enregistre les images choisies (dossier), ou toutes les images (archive ZIP). */
  async function extraire(zip: boolean) {
    if (!chemin) return;
    enCours = true;
    erreur = null;
    try {
      const ids = zip ? images.map((i) => i.id) : images.filter((i) => selection.has(i.id)).map((i) => i.id);
      resultat = { ...(await extraireImagesPdf(chemin, ids, await dossierPour(chemin), zip)), zip };
    } catch (e) {
      erreur = String(e);
    } finally {
      enCours = false;
    }
  }

  onMount(() => recevoirDepots(recevoir));
</script>

<ToolLayout
  titre="Extraire les images"
  description="Récupérez les photos et illustrations d'un PDF, dans leur qualité d'origine."
  {onretour}
>
  {#if resultat}
    <div class="resultat" role="status">
      <span class="coche"><Check size={22} strokeWidth={3} /></span>
      <div class="infos">
        <div class="titre">
          {resultat.nombre} image{resultat.nombre > 1 ? "s" : ""} enregistrée{resultat.nombre > 1 ? "s" : ""}
          {resultat.zip ? "dans l'archive" : "dans le dossier"}
        </div>
        <div class="meta">
          {resultat.chemin.split(/[\\/]/).pop()}
          {#if resultat.echecs > 0}· {resultat.echecs} image{resultat.echecs > 1 ? "s" : ""} illisible{resultat.echecs > 1 ? "s" : ""}{/if}
        </div>
      </div>
      <button class="secondaire" onclick={() => resultat && revealItemInDir(resultat.chemin)}>
        <Folder size={16} strokeWidth={2.6} />
        Afficher
      </button>
    </div>
  {/if}
  {#if erreur}
    <p class="erreur" role="alert">{erreur}</p>
  {/if}

  {#if !chemin}
    <DepotPdf
      titre="Glissez un PDF ici"
      aide="Allotrope affichera toutes ses images : choisissez celles à récupérer."
      formats="PDF"
      onparcourir={parcourir}
    />
  {:else if chargement}
    <div class="attente" role="status">
      <span class="rotation"><LoaderCircle size={22} strokeWidth={2.6} /></span>
      Recherche des images dans « {nom} »…
    </div>
  {:else}
    <div class="barre">
      <div class="fichier">
        <div class="nom" title={chemin}>{nom}</div>
        <div class="aide">
          {#if images.length === 0}
            Aucune image trouvée dans ce PDF (le texte et les dessins vectoriels ne sont pas des images).
          {:else}
            {images.length} image{images.length > 1 ? "s" : ""} trouvée{images.length > 1 ? "s" : ""} ·
            {selection.size} sélectionnée{selection.size > 1 ? "s" : ""}. Cliquez sur une image pour la choisir.
          {/if}
        </div>
      </div>
      {#if images.length > 0}
        <button class="secondaire" disabled={enCours} onclick={toutBasculer}>
          <CheckCheck size={16} strokeWidth={2.6} />
          {toutChoisi ? "Tout désélectionner" : "Tout sélectionner"}
        </button>
        <button class="secondaire" disabled={enCours} onclick={() => extraire(true)}>
          <FileArchive size={16} strokeWidth={2.6} />
          Tout télécharger (ZIP)
        </button>
      {/if}
      <button class="secondaire" disabled={enCours} onclick={parcourir}>Changer de PDF</button>
    </div>

    <div class="grille">
      {#each images as image (image.id)}
        <button
          class="image"
          class:choisie={selection.has(image.id)}
          aria-pressed={selection.has(image.id)}
          aria-label={`Image ${image.largeur} × ${image.hauteur}${image.page ? `, page ${image.page}` : ""}`}
          onclick={() => basculer(image.id)}
        >
          <span class="case" aria-hidden="true">
            {#if selection.has(image.id)}<Check size={16} strokeWidth={3} />{/if}
          </span>
          <span class="apercu">
            {#if image.apercu}
              <img src={image.apercu} alt="" draggable="false" />
            {:else}
              <span class="sans-apercu"><ImageOff size={28} strokeWidth={2.2} />Aperçu indisponible</span>
            {/if}
          </span>
          <span class="legende">
            {image.page ? `Page ${image.page}` : "Hors page"} · {image.largeur} × {image.hauteur} ·
            {image.format.toUpperCase()}
          </span>
        </button>
      {/each}
    </div>
  {/if}

  {#snippet pied()}
    {#if images.length > 0}
      <QueueFooter
        libelle={enCours
          ? "Enregistrement…"
          : selection.size === 0
            ? "Choisissez des images"
            : `Enregistrer ${selection.size} image${selection.size > 1 ? "s" : ""}`}
        desactive={enCours || selection.size === 0}
        onlancer={() => extraire(false)}
      />
    {/if}
  {/snippet}
</ToolLayout>

<style>
  .erreur {
    margin: 0;
    font-size: 14px;
    font-weight: 700;
    color: var(--braise);
  }

  .attente {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 24px;
    border-radius: 22px;
    border: 2px solid var(--ligne);
    background: var(--surface);
    font-size: 15px;
    font-weight: 700;
  }

  .rotation {
    display: flex;
    animation: tourner 1s linear infinite;
  }

  @keyframes tourner {
    to {
      transform: rotate(360deg);
    }
  }

  .barre {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 12px;
  }

  .fichier {
    flex: 1;
    min-width: 240px;
  }

  .nom {
    font-size: 18px;
    font-weight: 800;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .aide {
    font-size: 13px;
    font-weight: 600;
    color: var(--texte-secondaire);
  }

  /* Bouton « Secondaire » de la charte */
  .secondaire {
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

  .secondaire:hover:not(:disabled) {
    border-color: var(--givre);
  }

  .secondaire:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .grille {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 16px;
  }

  .image {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px;
    border-radius: 22px;
    border: 2px solid var(--ligne);
    background: var(--surface);
    color: var(--givre);
    font-family: inherit;
    text-align: left;
    cursor: pointer;
    transition:
      border-color var(--transition),
      background-color var(--transition);
  }

  .image:hover {
    border-color: var(--ligne-forte);
  }

  /* Image choisie : « carte sélectionnée » de la charte */
  .image.choisie {
    border: 3px solid var(--givre);
    padding: 11px;
    background: var(--surface-active);
  }

  .case {
    position: absolute;
    top: 18px;
    left: 18px;
    z-index: 1;
    width: 26px;
    height: 26px;
    border-radius: 8px;
    border: 2px solid var(--givre);
    background: var(--abysse);
    color: var(--blanc);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .choisie .case {
    background: var(--cobalt);
  }

  .apercu {
    height: 150px;
    border-radius: 14px;
    background: var(--abysse);
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }

  .apercu img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    pointer-events: none;
  }

  .sans-apercu {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    font-weight: 700;
    color: var(--texte-secondaire);
  }

  .legende {
    font-size: 12px;
    font-weight: 700;
    color: var(--texte-secondaire);
  }

  .choisie .legende {
    color: var(--givre);
  }

  .resultat {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 14px;
    padding: 16px 18px;
    border-radius: 22px;
    border: 2px solid var(--menthe);
    background: var(--menthe-fond);
  }

  .coche {
    display: flex;
    color: var(--menthe);
  }

  .infos {
    flex: 1;
    min-width: 200px;
  }

  .titre {
    font-size: 15px;
    font-weight: 900;
    color: var(--menthe);
  }

  .meta {
    font-size: 13px;
    font-weight: 600;
    overflow-wrap: anywhere;
  }

  .resultat .secondaire {
    background: var(--abysse);
  }
</style>
