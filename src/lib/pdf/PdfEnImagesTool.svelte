<!--
  Outil « PDF en images » : chaque page choisie devient une image (PNG, JPG ou WEBP),
  à la résolution voulue, enregistrée dans un dossier ou dans une archive ZIP.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import { open } from "@tauri-apps/plugin-dialog";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import type { PDFDocumentProxy } from "pdfjs-dist";
  import { Check, CheckCheck, Folder, X } from "@lucide/svelte";
  import ToolLayout from "./ToolLayout.svelte";
  import DepotPdf from "./DepotPdf.svelte";
  import Miniature from "./Miniature.svelte";
  import QueueFooter from "../QueueFooter.svelte";
  import Segmented from "../Segmented.svelte";
  import { recevoirDepots } from "../depot.svelte";
  import { dossierPour } from "../destination.svelte";
  import { reglages } from "../reglages.svelte";
  import { exporterPages, ouvrirPdf, type FormatPageImage, type ResultatExport } from "./pdf";

  interface Props {
    onretour: () => void;
  }
  let { onretour }: Props = $props();

  const LARGEUR_MINIATURE = 140;
  const FORMATS = [
    { id: "png" as const, libelle: "PNG" },
    { id: "jpeg" as const, libelle: "JPG" },
    { id: "webp" as const, libelle: "WEBP" },
  ];
  const RESOLUTIONS = [
    { id: "72", libelle: "Écran · 72 ppp" },
    { id: "150", libelle: "Standard · 150 ppp" },
    { id: "300", libelle: "Impression · 300 ppp" },
  ];
  const ENREGISTREMENTS = [
    { id: "dossier" as const, libelle: "Dossier" },
    { id: "zip" as const, libelle: "Archive ZIP" },
  ];

  let chemin = $state<string | null>(null);
  let pdf = $state.raw<PDFDocumentProxy | null>(null);
  let nombrePages = $state(0);
  /** Numéros des pages choisies. */
  const selection = new SvelteSet<number>();
  let format = $state<FormatPageImage>("png");
  let resolution = $state("150");
  let enregistrement = $state<"dossier" | "zip">("dossier");
  let chargement = $state(false);
  let enCours = $state(false);
  let faites = $state(0);
  let aExporter = $state(0);
  let annulation = false;
  let resultat = $state<ResultatExport | null>(null);
  let erreur = $state<string | null>(null);

  const nom = $derived(chemin ? (chemin.split(/[\\/]/).pop() ?? "") : "");
  const toutChoisi = $derived(nombrePages > 0 && selection.size === nombrePages);

  async function charger(nouveau: string) {
    erreur = null;
    resultat = null;
    chargement = true;
    try {
      const document = await ouvrirPdf(nouveau);
      await pdf?.loadingTask.destroy();
      pdf = document;
      chemin = nouveau;
      nombrePages = document.numPages;
      // Toutes les pages sont choisies au départ
      selection.clear();
      for (let i = 1; i <= nombrePages; i++) selection.add(i);
    } catch (e) {
      erreur = `Impossible d'ouvrir ce PDF : ${e instanceof Error ? e.message : e}`;
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

  function basculer(numero: number) {
    if (selection.has(numero)) selection.delete(numero);
    else selection.add(numero);
  }

  function toutBasculer() {
    if (toutChoisi) selection.clear();
    else for (let i = 1; i <= nombrePages; i++) selection.add(i);
  }

  async function exporter() {
    if (!pdf || !chemin) return;
    const pages = [...selection].sort((a, b) => a - b);
    enCours = true;
    annulation = false;
    erreur = null;
    resultat = null;
    faites = 0;
    aExporter = pages.length;
    try {
      resultat = await exporterPages(
        pdf,
        chemin,
        pages,
        {
          dossier: await dossierPour(chemin),
          zip: enregistrement === "zip",
          format,
          ppp: Number(resolution),
          qualite: reglages.qualite,
        },
        (n) => (faites = n),
        () => annulation,
      );
    } catch (e) {
      erreur = e === "Annulé" ? "Export annulé : aucun fichier n'a été gardé." : String(e);
    } finally {
      enCours = false;
    }
  }

  onMount(() => {
    const arret = recevoirDepots(recevoir);
    return () => {
      arret();
      pdf?.loadingTask.destroy();
    };
  });
</script>

<ToolLayout
  titre="PDF en images"
  description="Transformez chaque page d'un PDF en image, pour l'afficher, la partager ou l'imprimer."
  {onretour}
>
  {#if resultat}
    <div class="resultat" role="status">
      <span class="coche"><Check size={22} strokeWidth={3} /></span>
      <div class="infos">
        <div class="titre">
          {resultat.nombre} page{resultat.nombre > 1 ? "s" : ""} exportée{resultat.nombre > 1 ? "s" : ""}
        </div>
        <div class="meta">{resultat.chemin.split(/[\\/]/).pop()}</div>
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

  <section class="panneau" aria-label="Réglages de l'export">
    <div class="champ">
      <div class="etiquette">Format</div>
      <Segmented options={FORMATS} valeur={format} onchange={(f) => (format = f)} etiquette="Format des images" desactive={enCours} />
    </div>
    <div class="champ">
      <div class="etiquette">Résolution</div>
      <Segmented options={RESOLUTIONS} valeur={resolution} onchange={(r) => (resolution = r)} etiquette="Résolution" desactive={enCours} />
    </div>
    <div class="champ">
      <div class="etiquette">Enregistrer dans</div>
      <Segmented
        options={ENREGISTREMENTS}
        valeur={enregistrement}
        onchange={(e) => (enregistrement = e)}
        etiquette="Enregistrement"
        desactive={enCours}
      />
    </div>
    <p class="note">
      {format === "png"
        ? "PNG : sans perte, idéal pour le texte et les schémas."
        : `${format === "jpeg" ? "JPG" : "WEBP"} : plus léger, idéal pour les photos (qualité ${reglages.qualite} %, réglable dans les Paramètres).`}
      Une page A4 à 300 ppp mesure 2480 × 3508 pixels.
    </p>
  </section>

  {#if !pdf}
    <DepotPdf
      titre={chargement ? "Ouverture du PDF…" : "Glissez un PDF ici"}
      aide="Vous choisirez ensuite les pages à transformer en images."
      formats="PDF"
      onparcourir={parcourir}
    />
  {:else}
    <div class="barre">
      <div class="fichier">
        <div class="nom" title={chemin}>{nom}</div>
        <div class="aide">
          {selection.size} page{selection.size > 1 ? "s" : ""} sur {nombrePages} choisie{selection.size > 1 ? "s" : ""}.
          Cliquez sur une page pour l'ajouter ou la retirer.
        </div>
      </div>
      <button class="secondaire" disabled={enCours} onclick={toutBasculer}>
        <CheckCheck size={16} strokeWidth={2.6} />
        {toutChoisi ? "Tout désélectionner" : "Tout sélectionner"}
      </button>
      <button class="secondaire" disabled={enCours} onclick={parcourir}>Changer de PDF</button>
    </div>

    {#if enCours}
      <div class="avancement" role="status">
        <div class="ligne-avancement">
          <span>Page {Math.min(faites + 1, aExporter)} sur {aExporter}…</span>
          <button class="secondaire" onclick={() => (annulation = true)}>
            <X size={16} strokeWidth={2.6} />
            Annuler
          </button>
        </div>
        <div class="progression">
          <div class="remplissage" style:width="{aExporter ? (faites / aExporter) * 100 : 0}%"></div>
        </div>
      </div>
    {/if}

    <div class="grille">
      {#each { length: nombrePages } as _, i (i)}
        {@const numero = i + 1}
        <button
          class="page"
          class:choisie={selection.has(numero)}
          aria-pressed={selection.has(numero)}
          aria-label={`Page ${numero}`}
          disabled={enCours}
          onclick={() => basculer(numero)}
        >
          <span class="case" aria-hidden="true">
            {#if selection.has(numero)}<Check size={16} strokeWidth={3} />{/if}
          </span>
          <Miniature document={pdf} {numero} rotation={0} largeur={LARGEUR_MINIATURE} />
          <span class="legende">Page {numero}</span>
        </button>
      {/each}
    </div>
  {/if}

  {#snippet pied()}
    {#if pdf}
      <QueueFooter
        libelle={enCours
          ? "Export en cours…"
          : selection.size === 0
            ? "Choisissez des pages"
            : `Exporter ${selection.size} page${selection.size > 1 ? "s" : ""}`}
        desactive={enCours || selection.size === 0}
        onlancer={exporter}
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

  .panneau {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    gap: 16px 28px;
    padding: 18px 20px;
    border-radius: 22px;
    border: 2px solid var(--ligne);
    background: var(--surface);
  }

  .champ {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .etiquette {
    font-size: 14px;
    font-weight: 800;
  }

  .note {
    flex-basis: 100%;
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--texte-secondaire);
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

  .avancement {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 16px;
    border-radius: 22px;
    border: 2px solid var(--ligne);
    background: var(--surface);
  }

  .ligne-avancement {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    font-size: 14px;
    font-weight: 800;
  }

  /* Barre de progression de la charte (couleur de la famille Documents / PDF) */
  .progression {
    height: 12px;
    border-radius: 999px;
    background: var(--ligne);
    overflow: hidden;
  }

  .remplissage {
    height: 100%;
    border-radius: 999px;
    background: var(--famille-document);
    transition: width 300ms linear;
  }

  .grille {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(172px, 1fr));
    gap: 16px;
  }

  .page {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 14px 14px 12px;
    border-radius: 22px;
    border: 2px solid var(--ligne);
    background: var(--surface);
    color: var(--givre);
    font-family: inherit;
    cursor: pointer;
    transition:
      border-color var(--transition),
      background-color var(--transition);
  }

  .page:hover:not(:disabled) {
    border-color: var(--ligne-forte);
  }

  .page:disabled {
    cursor: default;
  }

  /* Page choisie : « carte sélectionnée » de la charte */
  .page.choisie {
    border: 3px solid var(--givre);
    padding: 13px 13px 11px;
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

  .legende {
    font-size: 13px;
    font-weight: 800;
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
