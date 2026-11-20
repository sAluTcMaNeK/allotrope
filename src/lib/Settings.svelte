<!-- Écran Paramètres : destination, qualité par défaut et À propos. Les changements sont enregistrés aussitôt. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { open } from "@tauri-apps/plugin-dialog";
  import { Download, FolderOpen, LoaderCircle, RefreshCw, ShieldCheck, X } from "@lucide/svelte";
  import { installerMiseAJour, miseAJour, rechercherMiseAJour } from "./mises-a-jour.svelte";
  import logo from "../../assets/allotrope-logo-complet.svg";
  import CompressionFields from "./CompressionFields.svelte";
  import Segmented from "./Segmented.svelte";
  import { reglages, type TypeDestination } from "./reglages.svelte";
  import { destination } from "./destination.svelte";

  interface Props {
    onfermer: () => void;
  }
  let { onfermer }: Props = $props();

  let version = $state("");
  onMount(() => {
    getVersion().then((v) => (version = v));
  });

  const destinations: { id: TypeDestination; libelle: string }[] = [
    { id: "documents", libelle: "Documents › Allotrope" },
    { id: "origine", libelle: "À côté de l'original" },
    { id: "personnalise", libelle: "Autre dossier" },
  ];

  async function choisirDossier(): Promise<boolean> {
    const choix = await open({
      directory: true,
      defaultPath: reglages.dossierPersonnalise || destination.documents,
      title: "Choisir le dossier de destination",
    });
    if (!choix) return false;
    reglages.dossierPersonnalise = choix;
    return true;
  }

  async function changerDestination(type: TypeDestination) {
    // « Autre dossier » sans dossier choisi : on le demande tout de suite
    if (type === "personnalise" && !reglages.dossierPersonnalise && !(await choisirDossier())) {
      return;
    }
    reglages.destination = type;
  }

  const licences = [
    ["Tauri", "MIT / Apache 2.0"],
    ["Svelte", "MIT"],
    ["image (Rust)", "MIT / Apache 2.0"],
    ["libwebp (Google)", "BSD"],
    ["oxipng", "MIT"],
    ["resvg (SVG)", "MIT / Apache 2.0"],
    ["vtracer (vectorisation)", "MIT / Apache 2.0"],
    ["jxl-oxide (JPEG XL)", "MIT / Apache 2.0"],
    ["lopdf (PDF)", "MIT"],
    ["pdf.js (Mozilla)", "Apache 2.0"],
    ["FFmpeg (build LGPL, programme séparé)", "LGPL 2.1+"],
    ["OpenH264 (Cisco)", "BSD"],
    ["Police Nunito", "SIL Open Font License"],
    ["Icônes Lucide", "ISC"],
  ];
</script>

<div class="parametres">
  <div class="entete">
    <h1>Paramètres</h1>
    <button class="carre" aria-label="Fermer les paramètres" onclick={onfermer}>
      <X size={18} strokeWidth={2.4} />
    </button>
  </div>

  <section class="carte">
    <h2>Enregistrement</h2>
    <div class="champ">
      <div class="etiquette">Dossier de destination</div>
      <Segmented
        options={destinations}
        valeur={reglages.destination}
        onchange={changerDestination}
        etiquette="Dossier de destination"
      />
      <div class="aide">
        {#if reglages.destination === "documents"}
          {destination.documents}
        {:else if reglages.destination === "origine"}
          Chaque fichier produit est enregistré dans le même dossier que son original.
        {:else}
          {reglages.dossierPersonnalise}
        {/if}
      </div>
      {#if reglages.destination === "personnalise"}
        <button class="secondaire" onclick={choisirDossier}>
          <FolderOpen size={16} strokeWidth={2.6} />
          Changer de dossier
        </button>
      {/if}
    </div>
    <p class="note">Les fichiers d'origine ne sont jamais modifiés ni écrasés.</p>
  </section>

  <section class="carte">
    <h2>Conversion</h2>
    <div class="champ">
      <div class="ligne">
        <label class="etiquette" for="qualite">Qualité des images JPG et WEBP</label>
        <span class="valeur">{reglages.qualite} %</span>
      </div>
      <input id="qualite" type="range" min="50" max="100" step="1" bind:value={reglages.qualite} />
      <div class="aide">Plus la qualité est élevée, plus le fichier est lourd. 85 % est un bon équilibre.</div>
    </div>
  </section>

  <section class="carte">
    <h2>Compression par défaut</h2>
    <CompressionFields
      id="parametres"
      options={reglages.compression}
      onchange={(modification) => Object.assign(reglages.compression, modification)}
    />
    <p class="note">
      Réglages proposés à chaque ouverture d'Allotrope, pour les images, l'audio, la vidéo et les PDF ;
      vous pouvez les ajuster au moment de compresser. Les PNG restent toujours sans perte.
    </p>
  </section>

  <section class="carte">
    <h2>Mises à jour</h2>
    <label class="case">
      <input type="checkbox" bind:checked={reglages.miseAJourAuto} />
      Rechercher les mises à jour au démarrage
    </label>

    <div class="mise-a-jour">
      {#if miseAJour.etat.etape === "disponible"}
        <button class="marque" onclick={installerMiseAJour}>
          <Download size={18} strokeWidth={2.6} />
          Installer la version {miseAJour.etat.version}
        </button>
        <span class="aide">Allotrope redémarrera une fois la mise à jour installée.</span>
      {:else if miseAJour.etat.etape === "telechargement"}
        <span class="etat">
          <span class="rotation"><LoaderCircle size={16} strokeWidth={2.6} /></span>
          Téléchargement de la version {miseAJour.etat.version}{miseAJour.etat.pourcentage !== null
            ? ` · ${miseAJour.etat.pourcentage} %`
            : "…"}
        </span>
      {:else}
        <button
          class="secondaire"
          disabled={miseAJour.etat.etape === "recherche"}
          onclick={() => rechercherMiseAJour()}
        >
          <RefreshCw size={16} strokeWidth={2.6} />
          {miseAJour.etat.etape === "recherche" ? "Recherche…" : "Rechercher une mise à jour"}
        </button>
        {#if miseAJour.etat.etape === "a-jour"}
          <span class="succes">Allotrope est à jour.</span>
        {:else if miseAJour.etat.etape === "erreur"}
          <span class="echec">{miseAJour.etat.message}</span>
        {/if}
      {/if}
    </div>
    <p class="note">
      Seule la recherche de mise à jour utilise internet (lecture du numéro de la dernière version).
      Aucun fichier n'est envoyé. Chaque mise à jour est signée et vérifiée avant installation.
    </p>
  </section>

  <section class="carte">
    <h2>À propos</h2>
    <img class="logo" src={logo} alt="Allotrope" draggable="false" />
    <div class="version">Version {version}</div>
    <div class="local">
      <span class="bouclier"><ShieldCheck size={22} strokeWidth={2.4} /></span>
      <div>
        <div class="local-titre">100 % local</div>
        <div class="aide">
          Toutes les conversions se font sur votre ordinateur. Aucun fichier n'est envoyé sur
          internet.
        </div>
      </div>
    </div>
    <div class="champ">
      <div class="etiquette">Composants open source</div>
      <ul class="licences">
        {#each licences as [nom, licence] (nom)}
          <li><span>{nom}</span><span class="licence">{licence}</span></li>
        {/each}
      </ul>
    </div>
  </section>
</div>

<style>
  .parametres {
    height: 100%;
    overflow-y: auto;
    padding: 32px 40px 40px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .entete {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  h1 {
    margin: 0;
    font-size: 30px;
    font-weight: 900;
    letter-spacing: -0.3px;
  }

  h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 800;
  }

  .carte {
    max-width: 760px;
    padding: 22px;
    border-radius: 26px;
    border: 2px solid var(--ligne);
    background: var(--surface);
    display: flex;
    flex-direction: column;
    gap: 22px;
  }

  .champ {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }

  .ligne {
    align-self: stretch;
    display: flex;
    justify-content: space-between;
    gap: 12px;
  }

  .etiquette {
    font-size: 14px;
    font-weight: 800;
  }

  .valeur {
    font-size: 14px;
    font-weight: 900;
    color: var(--lien);
  }

  .aide,
  .note {
    font-size: 13px;
    font-weight: 600;
    color: var(--texte-secondaire);
    line-height: 1.4;
    user-select: text;
  }

  .note {
    margin: 0;
  }

  input[type="range"] {
    align-self: stretch;
    accent-color: var(--cobalt);
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
    cursor: default;
    opacity: 0.7;
  }

  /* Bouton « Marque » de la charte */
  .marque {
    height: 52px;
    padding: 0 24px;
    border-radius: 999px;
    border: 3px solid var(--givre);
    background: var(--cobalt);
    color: var(--blanc);
    font-family: inherit;
    font-size: 15px;
    font-weight: 800;
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
  }

  .case {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 44px;
    font-size: 14px;
    font-weight: 700;
    cursor: pointer;
  }

  .case input {
    width: 20px;
    height: 20px;
    accent-color: var(--cobalt);
  }

  .mise-a-jour {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 12px 16px;
  }

  .etat,
  .succes,
  .echec {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 700;
  }

  .etat {
    color: var(--texte-secondaire);
  }

  .succes {
    color: var(--menthe);
  }

  .echec {
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

  .logo {
    width: 300px;
    max-width: 100%;
    border-radius: 22px;
    border: 2px solid var(--ligne);
    pointer-events: none;
  }

  .version {
    margin-top: -12px;
    font-size: 13px;
    font-weight: 700;
    color: var(--texte-secondaire);
  }

  .local {
    display: flex;
    gap: 12px;
    align-items: flex-start;
  }

  .bouclier {
    flex: none;
    display: flex;
    color: var(--menthe);
  }

  .local-titre {
    font-size: 14px;
    font-weight: 800;
  }

  .licences {
    align-self: stretch;
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
  }

  .licences li {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 0;
    border-bottom: 2px solid var(--ligne);
    font-size: 14px;
    font-weight: 700;
  }

  .licences li:last-child {
    border-bottom: none;
  }

  .licence {
    color: var(--texte-secondaire);
    font-weight: 600;
  }
</style>
