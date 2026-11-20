<!-- Page « Outils PDF » : grille de cartouches carrées ; un clic ouvre l'outil correspondant. -->
<script lang="ts">
  import type { Component } from "svelte";
  import { ArrowDownUp, FileText, FileType, ImageDown, ImagePlus, Images, Layers, Shrink, Trash } from "@lucide/svelte";
  import FusionTool from "./FusionTool.svelte";
  import PagesTool from "./PagesTool.svelte";
  import ImagesTool from "./ImagesTool.svelte";
  import CompressionPdfTool from "./CompressionPdfTool.svelte";
  import ExtractionImagesTool from "./ExtractionImagesTool.svelte";
  import PdfEnImagesTool from "./PdfEnImagesTool.svelte";
  import PdfVersTexteTool from "./PdfVersTexteTool.svelte";
  import DocumentVersPdfTool from "./DocumentVersPdfTool.svelte";

  import type { OptionsCompression } from "../fichiers";

  interface Props {
    /** Demande d'ouverture d'un outil (préréglage) ; `jeton` change à chaque nouvelle demande. */
    demande?: { outil: "compresser"; compression: OptionsCompression; jeton: number } | null;
  }
  let { demande = null }: Props = $props();

  type IdOutil =
    | "fusionner"
    | "reorganiser"
    | "supprimer"
    | "images"
    | "compresser"
    | "extraire-images"
    | "en-images"
    | "pdf-texte"
    | "word";

  interface Outil {
    id: IdOutil;
    titre: string;
    description: string;
    icone: Component<{ size?: number; strokeWidth?: number }>;
    /** Outil pas encore disponible : la cartouche est affichée mais inactive. */
    bientot?: string;
  }

  const OUTILS: Outil[] = [
    {
      id: "fusionner",
      titre: "Fusionner des PDF",
      description: "Assemblez plusieurs PDF en un seul document.",
      icone: Layers,
    },
    {
      id: "reorganiser",
      titre: "Réorganiser les pages",
      description: "Changez l'ordre des pages et faites-les pivoter.",
      icone: ArrowDownUp,
    },
    {
      id: "supprimer",
      titre: "Supprimer des pages",
      description: "Retirez les pages inutiles d'un PDF.",
      icone: Trash,
    },
    {
      id: "images",
      titre: "Images en PDF",
      description: "Une photo ou un scan par page, dans l'ordre choisi.",
      icone: ImagePlus,
    },
    {
      id: "compresser",
      titre: "Compresser un PDF",
      description: "Allégez un PDF pour l'envoyer par e-mail.",
      icone: Shrink,
    },
    {
      id: "extraire-images",
      titre: "Extraire les images",
      description: "Récupérez les photos d'un PDF, une à une ou toutes en ZIP.",
      icone: ImageDown,
    },
    {
      id: "en-images",
      titre: "PDF en images",
      description: "Exportez chaque page en PNG, JPG ou WEBP.",
      icone: Images,
    },
    {
      id: "pdf-texte",
      titre: "PDF en Word / ODT",
      description: "Rendez un PDF modifiable : DOCX ou ODT.",
      icone: FileType,
    },
    {
      id: "word",
      titre: "Word en PDF",
      description: "Transformez un document Word ou ODT en PDF.",
      icone: FileText,
    },
  ];

  let ouvert = $state<IdOutil | null>(null);
  /** Réglages du préréglage, valables tant que l'outil qu'il a ouvert reste affiché. */
  let compressionDemandee = $state<OptionsCompression | null>(null);
  const retour = () => {
    ouvert = null;
    compressionDemandee = null;
  };

  // Un préréglage ouvre directement l'outil demandé
  $effect(() => {
    if (!demande) return;
    ouvert = demande.outil;
    compressionDemandee = demande.compression;
  });
</script>

{#if ouvert === "fusionner"}
  <FusionTool onretour={retour} />
{:else if ouvert === "reorganiser" || ouvert === "supprimer"}
  {#key ouvert}
    <PagesTool mode={ouvert} onretour={retour} />
  {/key}
{:else if ouvert === "images"}
  <ImagesTool onretour={retour} />
{:else if ouvert === "compresser"}
  {#key demande?.jeton}
    <CompressionPdfTool onretour={retour} reglagesInitiaux={compressionDemandee} />
  {/key}
{:else if ouvert === "extraire-images"}
  <ExtractionImagesTool onretour={retour} />
{:else if ouvert === "en-images"}
  <PdfEnImagesTool onretour={retour} />
{:else if ouvert === "pdf-texte"}
  <PdfVersTexteTool onretour={retour} />
{:else if ouvert === "word"}
  <DocumentVersPdfTool onretour={retour} />
{:else}
  <div class="accueil">
    <header>
      <h1>Outils PDF</h1>
      <p class="description">
        Choisissez un outil. Vos documents restent sur votre ordinateur, et les originaux ne sont
        jamais modifiés.
      </p>
    </header>

    <div class="grille">
      {#each OUTILS as outil (outil.id)}
        <button
          class="cartouche"
          class:bientot={outil.bientot}
          disabled={!!outil.bientot}
          onclick={() => (ouvert = outil.id)}
        >
          {#if outil.bientot}
            <span class="etiquette">{outil.bientot}</span>
          {/if}
          <span class="icone"><outil.icone size={28} strokeWidth={2.4} /></span>
          <span class="titre">{outil.titre}</span>
          <span class="texte">{outil.description}</span>
        </button>
      {/each}
    </div>
  </div>
{/if}

<style>
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

  .description {
    margin: 6px 0 0;
    font-size: 15px;
    font-weight: 600;
    color: var(--texte-secondaire);
  }

  .grille {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(196px, 1fr));
    gap: 16px;
  }

  /* Cartouche carrée : rayon « carte » de la charte, contour Givre au survol */
  .cartouche {
    position: relative;
    aspect-ratio: 1;
    padding: 22px;
    border-radius: 26px;
    border: 2px solid var(--ligne);
    background: var(--surface);
    color: var(--givre);
    font-family: inherit;
    text-align: left;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    gap: 8px;
    cursor: pointer;
    transition:
      border-color var(--transition),
      background-color var(--transition);
  }

  .cartouche:hover:not(:disabled) {
    border-color: var(--givre);
    background: var(--surface-active);
  }

  .cartouche.bientot {
    cursor: default;
  }

  .cartouche.bientot .icone,
  .cartouche.bientot .titre,
  .cartouche.bientot .texte {
    opacity: 0.5;
  }

  /* Pastille de l'icône, aux couleurs de la famille Documents */
  .icone {
    width: 56px;
    height: 56px;
    margin-bottom: auto;
    border-radius: 16px;
    border: 2px solid var(--givre);
    background: var(--famille-document);
    color: var(--abysse);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .titre {
    font-size: 17px;
    font-weight: 900;
    line-height: 1.2;
  }

  .texte {
    font-size: 13px;
    font-weight: 600;
    color: var(--texte-secondaire);
    line-height: 1.35;
  }

  .etiquette {
    position: absolute;
    top: 18px;
    right: 18px;
    padding: 4px 10px;
    border-radius: 999px;
    background: var(--abysse);
    color: var(--texte-secondaire);
    font-size: 12px;
    font-weight: 800;
  }
</style>
