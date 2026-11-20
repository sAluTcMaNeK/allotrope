<!--
  Outil « Compresser un PDF » : allège un ou plusieurs PDF en réencodant leurs images.
  Mêmes réglages que pour les autres fichiers : taux (curseur) ou taille cible, dimensions maximales.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { Check, Folder, LoaderCircle, X } from "@lucide/svelte";
  import ToolLayout from "./ToolLayout.svelte";
  import DepotPdf from "./DepotPdf.svelte";
  import QueueFooter from "../QueueFooter.svelte";
  import CompressionFields from "../CompressionFields.svelte";
  import { analyserFichiers, formaterTaille, type OptionsCompression } from "../fichiers";
  import { reglages } from "../reglages.svelte";
  import { recevoirDepots } from "../depot.svelte";
  import { dossierPour } from "../destination.svelte";
  import { compresserPdf, type ResultatCompressionPdf } from "./pdf";

  interface Props {
    onretour: () => void;
    /** Réglages imposés par un préréglage ; sinon, compression par défaut des Paramètres. */
    reglagesInitiaux?: OptionsCompression | null;
  }
  let { onretour, reglagesInitiaux = null }: Props = $props();

  interface Fichier {
    chemin: string;
    nom: string;
    taille: number;
    etat: "attente" | "encours" | "termine" | "erreur";
    resultat?: ResultatCompressionPdf;
    erreur?: string;
  }

  let fichiers = $state<Fichier[]>([]);
  // Réglages de la session, initialisés avec la compression par défaut des Paramètres
  // svelte-ignore state_referenced_locally (seule la valeur initiale compte)
  let options = $state<OptionsCompression>({ ...(reglagesInitiaux ?? reglages.compression) });
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

  function changerOptions(modification: Partial<OptionsCompression>) {
    Object.assign(options, modification);
    // De nouveaux réglages permettent de recompresser les fichiers déjà traités
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
    const reglagesFiges = { ...options };
    for (const fichier of [...aTraiter]) {
      if (!fichiers.includes(fichier)) continue;
      fichier.etat = "encours";
      fichier.erreur = undefined;
      try {
        fichier.resultat = await compresserPdf(fichier.chemin, await dossierPour(fichier.chemin), reglagesFiges);
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
    if (!f.resultat.chemin) {
      return options.type === "taille" ? `${taille} · déjà sous la cible` : `${taille} · aucun gain possible`;
    }
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

  <section class="panneau" aria-label="Réglages de compression">
    <CompressionFields id="pdf" {options} onchange={changerOptions} desactive={enCours} famille="image" />
    <p class="note">
      {options.type === "taille"
        ? "La taille cible vise le poids du PDF entier : Allotrope baisse la qualité des images, puis leurs dimensions si besoin."
        : "Le taux règle la qualité des images (JPEG). Les dimensions maximales s'appliquent aux images du PDF."}
      Le texte, les polices et les dessins restent intacts.
    </p>
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
            <div class="pastille neutre">{options.type === "taille" ? "Déjà sous la cible" : "Déjà optimisé"}</div>
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
    align-items: stretch;
    gap: 14px;
    padding: 18px 20px;
    border-radius: 22px;
    border: 2px solid var(--ligne);
    background: var(--surface);
    /* Outil de compression : segment actif aux couleurs de la compression */
    --segment-actif: var(--mode-compresser);
    --segment-actif-texte: var(--mode-compresser-texte);
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
