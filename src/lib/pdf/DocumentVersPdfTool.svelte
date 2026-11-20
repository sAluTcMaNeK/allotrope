<!--
  Outil « Word en PDF » : transforme des documents (DOCX, DOC, ODT, RTF, TXT) en PDF,
  grâce à Microsoft Word ou LibreOffice installés sur l'ordinateur (conversion locale).
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { Check, Folder, Info, LoaderCircle, TriangleAlert, X } from "@lucide/svelte";
  import ToolLayout from "./ToolLayout.svelte";
  import DepotPdf from "./DepotPdf.svelte";
  import QueueFooter from "../QueueFooter.svelte";
  import Segmented from "../Segmented.svelte";
  import { analyserFichiers, formaterTaille } from "../fichiers";
  import { recevoirDepots } from "../depot.svelte";
  import { dossierPour } from "../destination.svelte";
  import { EXTENSIONS_DOCUMENT, documentVersPdf, moteursBureautique, type MoteurBureautique } from "./pdf";

  interface Props {
    onretour: () => void;
  }
  let { onretour }: Props = $props();

  interface Fichier {
    chemin: string;
    nom: string;
    extension: string;
    taille: number;
    etat: "attente" | "encours" | "termine" | "erreur";
    resultat?: { chemin: string; taille: number };
    erreur?: string;
  }

  let moteurs = $state<{ word: boolean; libreoffice: boolean } | null>(null);
  let moteur = $state<MoteurBureautique>("word");
  let fichiers = $state<Fichier[]>([]);
  let enCours = $state(false);
  let message = $state<string | null>(null);

  const aucunMoteur = $derived(moteurs !== null && !moteurs.word && !moteurs.libreoffice);
  const aTraiter = $derived(fichiers.filter((f) => f.etat === "attente" || f.etat === "erreur"));
  const nomMoteur = $derived(moteur === "word" ? "Word" : "LibreOffice");

  async function ajouter(chemins: string[]) {
    const infos = await analyserFichiers(chemins);
    const documents = infos.filter((i) => EXTENSIONS_DOCUMENT.includes(i.extension));
    const ignores = infos.length - documents.length;
    message =
      ignores === 0
        ? null
        : ignores === 1
          ? "1 fichier ignoré : ce n'est pas un document texte (DOCX, DOC, ODT, RTF, TXT)."
          : `${ignores} fichiers ignorés : ce ne sont pas des documents texte (DOCX, DOC, ODT, RTF, TXT).`;
    for (const info of documents) {
      if (!fichiers.some((f) => f.chemin === info.chemin)) {
        fichiers.push({
          chemin: info.chemin,
          nom: info.nom,
          extension: info.extension,
          taille: info.taille,
          etat: "attente",
        });
      }
    }
  }

  async function parcourir() {
    const choix = await open({
      multiple: true,
      title: "Choisir les documents à convertir",
      filters: [{ name: "Documents", extensions: [...EXTENSIONS_DOCUMENT] }],
    });
    if (choix) await ajouter(choix);
  }

  async function lancer() {
    enCours = true;
    // Un document à la fois : le traitement de texte ne sait en convertir qu'un à la fois
    for (const fichier of [...aTraiter]) {
      if (!fichiers.includes(fichier)) continue;
      fichier.etat = "encours";
      fichier.erreur = undefined;
      try {
        fichier.resultat = await documentVersPdf(fichier.chemin, await dossierPour(fichier.chemin), moteur);
        fichier.etat = "termine";
      } catch (e) {
        fichier.erreur = String(e);
        fichier.etat = "erreur";
      }
    }
    enCours = false;
  }

  onMount(() => {
    moteursBureautique().then((m) => {
      moteurs = m;
      // Word d'abord (mise en page la plus fidèle pour un document Word), sinon LibreOffice
      moteur = m.word ? "word" : "libreoffice";
    });
    return recevoirDepots(ajouter);
  });
</script>

<ToolLayout
  titre="Word en PDF"
  description="Transformez vos documents Word, ODT ou RTF en PDF, prêts à partager ou à imprimer."
  {onretour}
>
  {#if message}
    <p class="message">{message}</p>
  {/if}

  {#if aucunMoteur}
    <section class="alerte" role="alert">
      <span class="icone-alerte"><TriangleAlert size={22} strokeWidth={2.4} /></span>
      <div>
        <div class="titre-alerte">Un traitement de texte est nécessaire</div>
        <p>
          Pour mettre en page vos documents fidèlement, Allotrope s'appuie sur <strong>Microsoft Word</strong> ou
          sur <strong>LibreOffice</strong> (gratuit, libreoffice.org), installés sur votre ordinateur. La conversion
          reste entièrement locale. Installez l'un des deux, puis rouvrez cet outil.
        </p>
      </div>
    </section>
  {:else}
    <section class="panneau" aria-label="Réglages de la conversion">
      {#if moteurs?.word && moteurs?.libreoffice}
        <div class="champ">
          <div class="etiquette">Moteur</div>
          <Segmented
            options={[
              { id: "word" as const, libelle: "Microsoft Word" },
              { id: "libreoffice" as const, libelle: "LibreOffice" },
            ]}
            valeur={moteur}
            onchange={(m) => (moteur = m)}
            etiquette="Moteur de conversion"
            desactive={enCours}
          />
        </div>
      {/if}
      <p class="note">
        <Info size={16} strokeWidth={2.6} />
        {#if moteurs}
          Conversion réalisée par {moteur === "word" ? "Microsoft Word" : "LibreOffice"}, sur cet ordinateur.
        {/if}
        Avec LibreOffice, la mise en page d'un document Word peut différer légèrement (polices, sauts de page).
      </p>
    </section>

    {#if fichiers.length === 0}
      <DepotPdf
        titre="Glissez vos documents ici"
        aide="Chaque document devient un PDF, avec sa mise en page."
        formats="DOCX · DOC · ODT · RTF · TXT"
        onparcourir={parcourir}
      />
    {:else}
      <section class="liste" aria-label="Documents à convertir">
        <h2>Documents <span class="compte">· {fichiers.length}</span></h2>
        {#each fichiers as fichier, index (fichier.chemin)}
          <div class="ligne" class:en-erreur={fichier.etat === "erreur"}>
            <div class="badge">{fichier.extension.slice(0, 4).toUpperCase()}</div>
            <div class="infos">
              <div class="nom" title={fichier.chemin}>{fichier.nom}</div>
              {#if fichier.etat === "erreur"}
                <div class="erreur" title={fichier.erreur}>{fichier.erreur}</div>
              {:else if fichier.etat === "termine" && fichier.resultat}
                <div class="meta">
                  {formaterTaille(fichier.taille)} → PDF · {formaterTaille(fichier.resultat.taille)}
                </div>
              {:else}
                <div class="meta">{formaterTaille(fichier.taille)}</div>
              {/if}
            </div>

            {#if fichier.etat === "encours"}
              <div class="etat"><span class="rotation"><LoaderCircle size={16} strokeWidth={2.6} /></span>Conversion par {nomMoteur}…</div>
            {:else if fichier.etat === "termine" && fichier.resultat}
              <div class="pastille"><Check size={16} strokeWidth={3} />Terminé</div>
              <button
                class="carre"
                aria-label="Afficher dans le dossier"
                onclick={() => fichier.resultat && revealItemInDir(fichier.resultat.chemin)}
              >
                <Folder size={18} strokeWidth={2.4} />
              </button>
            {:else if fichier.etat === "erreur"}
              <div class="etat echec">Échec</div>
            {:else}
              <div class="etat">En attente</div>
            {/if}

            {#if fichier.etat !== "encours"}
              <button class="carre" aria-label={`Retirer ${fichier.nom}`} onclick={() => fichiers.splice(index, 1)}>
                <X size={18} strokeWidth={2.4} />
              </button>
            {/if}
          </div>
        {/each}
      </section>
      <DepotPdf compacte titre="Ajouter d'autres documents" aide="Glissez-les ici ou parcourez vos dossiers." formats="" onparcourir={parcourir} />
    {/if}
  {/if}

  {#snippet pied()}
    {#if fichiers.length > 0 && !aucunMoteur}
      <QueueFooter
        libelle={enCours
          ? "Conversion en cours…"
          : aTraiter.length === 0
            ? "Rien à convertir"
            : `Convertir ${aTraiter.length} document${aTraiter.length > 1 ? "s" : ""} en PDF`}
        desactive={enCours || aTraiter.length === 0 || moteurs === null}
        onlancer={lancer}
      />
    {/if}
  {/snippet}
</ToolLayout>

<style>
  .message {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--texte-secondaire);
  }

  .alerte {
    display: flex;
    gap: 14px;
    padding: 18px 20px;
    border-radius: 22px;
    border: 2px solid var(--ambre);
    background: var(--surface);
  }

  .icone-alerte {
    display: flex;
    color: var(--ambre);
  }

  .titre-alerte {
    font-size: 15px;
    font-weight: 900;
  }

  .alerte p {
    margin: 4px 0 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--texte-secondaire);
    line-height: 1.45;
  }

  .alerte strong {
    color: var(--givre);
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
    display: flex;
    align-items: flex-start;
    gap: 10px;
    font-size: 13px;
    font-weight: 600;
    color: var(--texte-secondaire);
    line-height: 1.4;
  }

  .note :global(svg) {
    flex: none;
    margin-top: 1px;
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

  .ligne {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 14px 16px;
    border-radius: 22px;
    border: 2px solid var(--ligne);
    background: var(--surface);
  }

  .ligne.en-erreur {
    border-color: var(--braise);
  }

  .badge {
    width: 48px;
    height: 48px;
    flex: none;
    border-radius: 14px;
    border: 2px solid var(--givre);
    background: var(--famille-document);
    color: var(--abysse);
    font-size: 12px;
    font-weight: 900;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .infos {
    flex: 1;
    min-width: 0;
  }

  .nom,
  .erreur {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .nom {
    font-size: 15px;
    font-weight: 800;
  }

  .meta {
    font-size: 13px;
    font-weight: 600;
    color: var(--texte-secondaire);
  }

  .erreur {
    font-size: 13px;
    font-weight: 700;
    color: var(--braise);
  }

  .etat {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    font-weight: 700;
    color: var(--texte-secondaire);
    white-space: nowrap;
  }

  .etat.echec {
    color: var(--braise);
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

  .pastille {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 36px;
    padding: 0 14px;
    border-radius: 999px;
    background: var(--menthe-fond);
    color: var(--menthe);
    font-size: 13px;
    font-weight: 800;
    white-space: nowrap;
  }

  /* Bouton « Icône seule » de la charte */
  .carre {
    width: 44px;
    height: 44px;
    flex: none;
    border-radius: 14px;
    border: 2px solid var(--ligne);
    background: transparent;
    color: var(--givre);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: border-color var(--transition);
  }

  .carre:hover {
    border-color: var(--ligne-forte);
  }
</style>
