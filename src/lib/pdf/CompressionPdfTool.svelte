<!-- Outil « Compresser un PDF » : allège un ou plusieurs PDF en réencodant leurs images. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { Check, Folder, LoaderCircle, X } from "@lucide/svelte";
  import ToolLayout from "./ToolLayout.svelte";
  import DepotPdf from "./DepotPdf.svelte";
  import QueueFooter from "../QueueFooter.svelte";
  import Segmented from "../Segmented.svelte";
  import { analyserFichiers, formaterTaille } from "../fichiers";
  import { recevoirDepots } from "../depot.svelte";
  import { dossierPour } from "../destination.svelte";
  import { compresserPdf, type NiveauPdf, type ResultatCompressionPdf } from "./pdf";

  interface Props {
    onretour: () => void;
  }
  let { onretour }: Props = $props();

  interface Fichier {
    chemin: string;
    nom: string;
    taille: number;
    etat: "attente" | "encours" | "termine" | "erreur";
    resultat?: ResultatCompressionPdf;
    erreur?: string;
  }

  const NIVEAUX: { id: NiveauPdf; libelle: string; detail: string }[] = [
    { id: "legere", libelle: "Légère", detail: "Qualité préservée : images en qualité 80 %, jusqu'à 3000 px." },
    { id: "recommandee", libelle: "Recommandée", detail: "Bon compromis pour l'écran et l'e-mail : qualité 65 %, jusqu'à 2000 px." },
    { id: "forte", libelle: "Forte", detail: "Fichier le plus léger : qualité 45 %, jusqu'à 1400 px. Idéal pour un envoi rapide." },
  ];

  let fichiers = $state<Fichier[]>([]);
  let niveau = $state<NiveauPdf>("recommandee");
  let enCours = $state(false);
  let message = $state<string | null>(null);

  const aTraiter = $derived(fichiers.filter((f) => f.etat === "attente" || f.etat === "erreur"));

  async function ajouter(chemins: string[]) {
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
      if (!fichiers.some((f) => f.chemin === info.chemin)) {
        fichiers.push({ chemin: info.chemin, nom: info.nom, taille: info.taille, etat: "attente" });
      }
    }
  }

  async function parcourir() {
    const choix = await open({
      multiple: true,
      title: "Choisir les PDF à compresser",
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (choix) await ajouter(choix);
  }

  function changerNiveau(nouveau: NiveauPdf) {
    niveau = nouveau;
    // Un nouveau niveau permet de recompresser les fichiers déjà traités
    for (const f of fichiers) {
      if (f.etat !== "encours") {
        f.etat = "attente";
        f.resultat = undefined;
        f.erreur = undefined;
      }
    }
  }

  async function lancer() {
    enCours = true;
    // Un PDF à la fois : chaque compression utilise déjà plusieurs cœurs pour les images
    for (const fichier of [...aTraiter]) {
      if (!fichiers.includes(fichier)) continue;
      fichier.etat = "encours";
      fichier.erreur = undefined;
      try {
        fichier.resultat = await compresserPdf(fichier.chemin, await dossierPour(fichier.chemin), niveau);
        fichier.etat = "termine";
      } catch (e) {
        fichier.erreur = String(e);
        fichier.etat = "erreur";
      }
    }
    enCours = false;
  }

  function meta(f: Fichier): string {
    const taille = formaterTaille(f.taille);
    if (f.etat !== "termine" || !f.resultat) return taille;
    if (!f.resultat.chemin) return `${taille} · aucun gain possible`;
    const gain = Math.round((1 - f.resultat.taille / f.taille) * 100);
    return `${taille} → ${formaterTaille(f.resultat.taille)} · −${gain} %`;
  }

  onMount(() => recevoirDepots(ajouter));
</script>

<ToolLayout
  titre="Compresser un PDF"
  description="Allégez vos PDF pour les envoyer par e-mail. Le texte reste net : seules les images sont réduites."
  {onretour}
>
  {#if message}
    <p class="message">{message}</p>
  {/if}

  <section class="panneau" aria-label="Niveau de compression">
    <div class="etiquette">Niveau de compression</div>
    <Segmented
      options={NIVEAUX.map(({ id, libelle }) => ({ id, libelle }))}
      valeur={niveau}
      onchange={changerNiveau}
      etiquette="Niveau de compression"
      desactive={enCours}
    />
    <p class="note">{NIVEAUX.find((n) => n.id === niveau)?.detail}</p>
  </section>

  {#if fichiers.length === 0}
    <DepotPdf
      titre="Glissez vos PDF ici"
      aide="Vous pouvez en compresser plusieurs à la fois."
      formats="PDF"
      onparcourir={parcourir}
    />
  {:else}
    <section class="liste" aria-label="PDF à compresser">
      <h2>PDF <span class="compte">· {fichiers.length}</span></h2>
      {#each fichiers as fichier, index (fichier.chemin)}
        <div class="ligne" class:en-erreur={fichier.etat === "erreur"}>
          <div class="badge">PDF</div>
          <div class="infos">
            <div class="nom" title={fichier.chemin}>{fichier.nom}</div>
            {#if fichier.etat === "erreur"}
              <div class="erreur">{fichier.erreur}</div>
            {:else}
              <div class="meta">{meta(fichier)}</div>
            {/if}
          </div>

          {#if fichier.etat === "encours"}
            <div class="etat"><span class="rotation"><LoaderCircle size={16} strokeWidth={2.6} /></span>Compression…</div>
          {:else if fichier.etat === "termine" && fichier.resultat?.chemin}
            <div class="pastille succes"><Check size={16} strokeWidth={3} />Terminé</div>
            <button
              class="carre"
              aria-label="Afficher dans le dossier"
              onclick={() => fichier.resultat?.chemin && revealItemInDir(fichier.resultat.chemin)}
            >
              <Folder size={18} strokeWidth={2.4} />
            </button>
          {:else if fichier.etat === "termine"}
            <div class="pastille neutre">Déjà optimisé</div>
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
    <DepotPdf compacte titre="Ajouter d'autres PDF" aide="Glissez-les ici ou parcourez vos dossiers." formats="" onparcourir={parcourir} />
  {/if}

  {#snippet pied()}
    {#if fichiers.length > 0}
      <QueueFooter
        libelle={enCours
          ? "Compression en cours…"
          : aTraiter.length === 0
            ? "Rien à compresser"
            : `Compresser ${aTraiter.length} PDF`}
        desactive={enCours || aTraiter.length === 0}
        onlancer={lancer}
      />
    {/if}
  {/snippet}
</ToolLayout>

<style>
  .panneau {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
    padding: 18px 20px;
    border-radius: 22px;
    border: 2px solid var(--ligne);
    background: var(--surface);
    /* Outil de compression : segment actif aux couleurs de la compression */
    --segment-actif: var(--mode-compresser);
    --segment-actif-texte: var(--mode-compresser-texte);
  }

  .etiquette {
    font-size: 14px;
    font-weight: 800;
  }

  .note,
  .message {
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
    font-size: 13px;
    font-weight: 800;
    white-space: nowrap;
  }

  .pastille.succes {
    background: var(--menthe-fond);
    color: var(--menthe);
  }

  .pastille.neutre {
    background: var(--abysse);
    color: var(--texte-secondaire);
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
