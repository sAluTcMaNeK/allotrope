<!-- Outil « Images en PDF » : une image par page, dans l'ordre choisi. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import ToolLayout from "./ToolLayout.svelte";
  import DepotPdf from "./DepotPdf.svelte";
  import LigneFichier from "./LigneFichier.svelte";
  import ResultatPdf from "./ResultatPdf.svelte";
  import QueueFooter from "../QueueFooter.svelte";
  import Segmented from "../Segmented.svelte";
  import { FAMILLES, analyserFichiers, formaterTaille, type InfoFichier } from "../fichiers";
  import { recevoirDepots } from "../depot.svelte";
  import { dossierPour } from "../destination.svelte";
  import { imagesEnPdf, nomSansExtension, type FormatPage, type ResultatPdf as Resultat } from "./pdf";

  interface Props {
    onretour: () => void;
  }
  let { onretour }: Props = $props();

  type Marge = "aucune" | "petite" | "normale";
  /** Marges en points (1 pt = 1/72 de pouce, environ 0,35 mm). */
  const MARGES: Record<Marge, number> = { aucune: 0, petite: 18, normale: 36 };

  let images = $state<InfoFichier[]>([]);
  let formatPage = $state<FormatPage>("a4");
  let marge = $state<Marge>("petite");
  let enCours = $state(false);
  let resultat = $state<Resultat | null>(null);
  let erreur = $state<string | null>(null);
  let message = $state<string | null>(null);

  async function ajouter(chemins: string[]) {
    resultat = null;
    erreur = null;
    const infos = await analyserFichiers(chemins);
    // Toute image lisible par Allotrope (formats de sortie non vides)
    const lisibles = infos.filter((i) => i.formatsSortie.length > 0);
    const ignores = infos.length - lisibles.length;
    message =
      ignores === 0
        ? null
        : `${ignores} fichier${ignores > 1 ? "s ignorés" : " ignoré"} : format d'image non pris en charge.`;
    for (const info of lisibles) {
      if (!images.some((i) => i.chemin === info.chemin)) images.push(info);
    }
  }

  async function parcourir() {
    const choix = await open({
      multiple: true,
      title: "Choisir les images",
      filters: [{ name: "Images", extensions: FAMILLES.image.extensions.concat("gif") }],
    });
    if (choix) await ajouter(choix);
  }

  function deplacer(index: number, decalage: number) {
    const [image] = images.splice(index, 1);
    images.splice(index + decalage, 0, image);
    resultat = null;
  }

  async function creer() {
    enCours = true;
    erreur = null;
    try {
      resultat = await imagesEnPdf(
        images.map((i) => i.chemin),
        await dossierPour(images[0].chemin),
        nomSansExtension(images[0].chemin),
        formatPage,
        MARGES[marge],
      );
    } catch (e) {
      erreur = String(e);
    } finally {
      enCours = false;
    }
  }

  onMount(() => recevoirDepots(ajouter));
</script>

<ToolLayout
  titre="Images en PDF"
  description="Rassemblez des photos ou des scans dans un PDF, une image par page."
  {onretour}
>
  {#if resultat}
    <ResultatPdf {resultat} onrecommencer={() => ((images = []), (resultat = null))} />
  {/if}
  {#if erreur}
    <p class="erreur" role="alert">{erreur}</p>
  {/if}
  {#if message}
    <p class="message">{message}</p>
  {/if}

  <section class="panneau" aria-label="Mise en page">
    <div class="champ">
      <div class="etiquette">Format des pages</div>
      <Segmented
        options={[
          { id: "a4", libelle: "A4" },
          { id: "image", libelle: "Taille de l'image" },
        ]}
        valeur={formatPage}
        onchange={(f) => (formatPage = f)}
        etiquette="Format des pages"
        desactive={enCours}
      />
    </div>
    {#if formatPage === "a4"}
      <div class="champ">
        <div class="etiquette">Marges</div>
        <Segmented
          options={[
            { id: "aucune", libelle: "Aucune" },
            { id: "petite", libelle: "Petites" },
            { id: "normale", libelle: "Normales" },
          ]}
          valeur={marge}
          onchange={(m) => (marge = m)}
          etiquette="Marges"
          desactive={enCours}
        />
      </div>
    {/if}
    <p class="note">
      {formatPage === "a4"
        ? "Chaque image est centrée sur une page A4, en portrait ou en paysage selon sa forme."
        : "Chaque page prend exactement les dimensions de son image."}
      Les JPEG sont intégrés sans perte de qualité.
    </p>
  </section>

  {#if images.length === 0}
    <DepotPdf
      titre="Glissez vos images ici"
      aide="Photos, scans, captures d'écran : rangez-les ensuite dans l'ordre des pages."
      formats="JPG · PNG · WEBP · TIFF · SVG…"
      couleur="var(--famille-image)"
      onparcourir={parcourir}
    />
  {:else}
    <section class="liste" aria-label="Images à assembler">
      <h2>Pages <span class="compte">· {images.length} image{images.length > 1 ? "s" : ""}</span></h2>
      {#each images as image, index (image.chemin)}
        <LigneFichier
          position={index + 1}
          badge={image.extension.slice(0, 4).toUpperCase()}
          couleur="var(--famille-image)"
          texte="var(--abysse)"
          nom={image.nom}
          meta={image.dimensions
            ? `${formaterTaille(image.taille)} · ${image.dimensions[0]} × ${image.dimensions[1]}`
            : formaterTaille(image.taille)}
          premier={index === 0}
          dernier={index === images.length - 1}
          desactive={enCours}
          onmonter={() => deplacer(index, -1)}
          ondescendre={() => deplacer(index, 1)}
          onretirer={() => images.splice(index, 1)}
        />
      {/each}
    </section>
    <DepotPdf
      compacte
      titre="Ajouter d'autres images"
      aide="Glissez-les ici ou parcourez vos dossiers."
      formats=""
      onparcourir={parcourir}
    />
  {/if}

  {#snippet pied()}
    {#if images.length > 0}
      <QueueFooter
        libelle={enCours ? "Création du PDF…" : `Créer un PDF de ${images.length} page${images.length > 1 ? "s" : ""}`}
        desactive={enCours}
        onlancer={creer}
      />
    {/if}
  {/snippet}
</ToolLayout>

<style>
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
