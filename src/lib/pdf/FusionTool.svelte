<!-- Outil « Fusionner des PDF » : plusieurs PDF, dans l'ordre choisi, deviennent un seul document. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import ToolLayout from "./ToolLayout.svelte";
  import DepotPdf from "./DepotPdf.svelte";
  import LigneFichier from "./LigneFichier.svelte";
  import ResultatPdf from "./ResultatPdf.svelte";
  import QueueFooter from "../QueueFooter.svelte";
  import { analyserFichiers, formaterTaille } from "../fichiers";
  import { recevoirDepots } from "../depot.svelte";
  import { dossierPour } from "../destination.svelte";
  import { assemblerPdf, infosPdf, nomSansExtension, type PageChoisie, type ResultatPdf as Resultat } from "./pdf";

  interface Props {
    onretour: () => void;
  }
  let { onretour }: Props = $props();

  interface Document {
    chemin: string;
    nom: string;
    taille: number;
    /** Nombre de pages, connu après lecture (`null` en cours de lecture). */
    pages: number | null;
    erreur?: string;
  }

  let documents = $state<Document[]>([]);
  let enCours = $state(false);
  let resultat = $state<Resultat | null>(null);
  let erreur = $state<string | null>(null);
  let message = $state<string | null>(null);

  const valides = $derived(documents.filter((d) => d.pages !== null && !d.erreur));
  const totalPages = $derived(valides.reduce((total, d) => total + (d.pages ?? 0), 0));
  const pret = $derived(
    valides.length >= 2 && valides.length === documents.length && !enCours,
  );

  async function ajouter(chemins: string[]) {
    resultat = null;
    erreur = null;
    const infos = await analyserFichiers(chemins);
    const pdfs = infos.filter((i) => i.extension === "pdf");
    const ignores = infos.length - pdfs.length;
    message =
      ignores === 0
        ? null
        : ignores === 1
          ? "1 fichier ignoré : ce n'est pas un PDF."
          : `${ignores} fichiers ignorés : ce ne sont pas des PDF.`;

    for (const info of pdfs) {
      if (documents.some((d) => d.chemin === info.chemin)) continue;
      documents.push({ chemin: info.chemin, nom: info.nom, taille: info.taille, pages: null });
      const document = documents[documents.length - 1];
      infosPdf(info.chemin)
        .then((i) => (document.pages = i.pages))
        .catch((e) => (document.erreur = String(e)));
    }
  }

  async function parcourir() {
    const choix = await open({
      multiple: true,
      title: "Choisir les PDF à fusionner",
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (choix) await ajouter(choix);
  }

  function deplacer(index: number, decalage: number) {
    const [document] = documents.splice(index, 1);
    documents.splice(index + decalage, 0, document);
    resultat = null;
  }

  function retirer(index: number) {
    documents.splice(index, 1);
    resultat = null;
  }

  async function fusionner() {
    enCours = true;
    erreur = null;
    try {
      // Toutes les pages de chaque document, dans l'ordre de la liste
      const pages: PageChoisie[] = documents.flatMap((d, source) =>
        Array.from({ length: d.pages ?? 0 }, (_, i) => ({ source, page: i + 1, rotation: 0 })),
      );
      resultat = await assemblerPdf(
        documents.map((d) => d.chemin),
        pages,
        await dossierPour(documents[0].chemin),
        `${nomSansExtension(documents[0].chemin)} (fusionné)`,
      );
    } catch (e) {
      erreur = String(e);
    } finally {
      enCours = false;
    }
  }

  function recommencer() {
    documents = [];
    resultat = null;
  }

  onMount(() => recevoirDepots(ajouter));
</script>

<ToolLayout
  titre="Fusionner des PDF"
  description="Assemblez plusieurs PDF en un seul document, dans l'ordre de votre choix."
  {onretour}
>
  {#if resultat}
    <ResultatPdf {resultat} onrecommencer={recommencer} />
  {/if}
  {#if erreur}
    <p class="erreur" role="alert">{erreur}</p>
  {/if}
  {#if message}
    <p class="message">{message}</p>
  {/if}

  {#if documents.length === 0}
    <DepotPdf
      titre="Glissez vos PDF ici"
      aide="Ajoutez au moins deux documents, puis rangez-les dans l'ordre voulu."
      formats="PDF"
      onparcourir={parcourir}
    />
  {:else}
    <section class="liste" aria-label="Documents à fusionner">
      <h2>
        Documents
        <span class="compte">· {documents.length} PDF · {totalPages} pages</span>
      </h2>
      {#each documents as document, index (document.chemin)}
        <LigneFichier
          position={index + 1}
          badge="PDF"
          couleur="var(--famille-document)"
          texte="var(--abysse)"
          nom={document.nom}
          meta={`${document.pages === null ? "Lecture…" : `${document.pages} page${document.pages > 1 ? "s" : ""}`} · ${formaterTaille(document.taille)}`}
          erreur={document.erreur}
          premier={index === 0}
          dernier={index === documents.length - 1}
          desactive={enCours}
          onmonter={() => deplacer(index, -1)}
          ondescendre={() => deplacer(index, 1)}
          onretirer={() => retirer(index)}
        />
      {/each}
    </section>
    <DepotPdf
      compacte
      titre="Ajouter d'autres PDF"
      aide="Glissez-les ici ou parcourez vos dossiers."
      formats="PDF"
      onparcourir={parcourir}
    />
  {/if}

  {#snippet pied()}
    {#if documents.length > 0}
      <QueueFooter
        libelle={enCours ? "Fusion en cours…" : documents.length < 2 ? "Ajoutez un 2e PDF" : `Fusionner ${documents.length} PDF`}
        desactive={!pret}
        onlancer={fusionner}
      />
    {/if}
  {/snippet}
</ToolLayout>

<style>
  .liste {
    display: flex;
    flex-direction: column;
    gap: 12px;
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

  .erreur,
  .message {
    margin: 0;
    font-size: 14px;
    font-weight: 700;
  }

  .erreur {
    color: var(--braise);
  }

  .message {
    color: var(--texte-secondaire);
  }
</style>
