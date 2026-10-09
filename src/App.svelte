<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { open } from "@tauri-apps/plugin-dialog";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import TitleBar from "./lib/TitleBar.svelte";
  import Sidebar from "./lib/Sidebar.svelte";
  import DropZone from "./lib/DropZone.svelte";
  import FileQueue from "./lib/FileQueue.svelte";
  import ModeSwitch from "./lib/ModeSwitch.svelte";
  import OptionsPanel from "./lib/OptionsPanel.svelte";
  import QueueFooter from "./lib/QueueFooter.svelte";
  import Settings from "./lib/Settings.svelte";
  import Splash from "./lib/Splash.svelte";
  import PdfTools from "./lib/pdf/PdfTools.svelte";
  import { depot } from "./lib/depot.svelte";
  import { dossierPour, initialiserDestination } from "./lib/destination.svelte";
  import {
    FAMILLES,
    LISTE_FAMILLES,
    analyserFichiers,
    compresserImage,
    convertirImage,
    estMedia,
    operationMedia,
    traiterMedia,
    annulerMedia,
    traitable,
    type ElementFile,
    type Famille,
    type FormatImage,
    type FormatSortie,
    type InfoFichier,
    type Mode,
    type OptionsCompression,
  } from "./lib/fichiers";
  import type { Page } from "./lib/navigation";
  import { reglages } from "./lib/reglages.svelte";

  /** Nombre maximal de traitements simultanés. */
  const TRAITEMENTS_PARALLELES = Math.max(1, Math.min(4, navigator.hardwareConcurrency || 2));

  let demarrage = $state(true);
  let page = $state<Page>("accueil");
  /** Page à retrouver en fermant les paramètres. */
  let pagePrecedente = $state<Page>("accueil");
  let mode = $state<Mode>("convertir");
  let elements = $state<ElementFile[]>([]);
  let enCours = $state(false);
  /** Message temporaire (fichiers ignorés…). */
  let message = $state<string | null>(null);

  /** Format de sortie choisi sur la page de chaque famille (onglet Conversion). */
  let cibles = $state<Record<Famille, string>>({
    image: "jpeg",
    video: "mp4",
    audio: "mp3",
  });
  // Réglages de compression de la session, initialisés avec les valeurs par défaut des paramètres
  let compression = $state<OptionsCompression>({ ...reglages.compression });

  /** Famille de la page affichée, ou `null` sur l'accueil. */
  const famille = $derived<Famille | null>(
    page !== "accueil" && page !== "parametres" && page !== "pdf" ? page : null,
  );
  /** Fichiers affichés : tous sur l'accueil, ceux de la famille sur une page dédiée. */
  const visibles = $derived(famille ? elements.filter((e) => e.info.famille === famille) : elements);
  const aTraiter = $derived(
    visibles.filter((e) => traitable(e, mode) && (e.etat === "attente" || e.etat === "erreur")),
  );

  const compteurs = $derived.by(() => {
    const c: Record<string, number> = { accueil: elements.length };
    for (const [id] of LISTE_FAMILLES) c[id] = elements.filter((e) => e.info.famille === id).length;
    return c;
  });

  const libelleLancement = $derived.by(() => {
    if (enCours) return mode === "convertir" ? "Conversion en cours…" : "Compression en cours…";
    const n = aTraiter.length;
    if (n === 0) return "Rien à traiter";
    return `${mode === "convertir" ? "Convertir" : "Compresser"} ${n} fichier${n > 1 ? "s" : ""}`;
  });

  function naviguer(cible: Page) {
    if (cible === "parametres" && page !== "parametres") pagePrecedente = page;
    page = cible;
  }

  function basculerParametres() {
    naviguer(page === "parametres" ? pagePrecedente : "parametres");
  }

  function afficherMessage(texte: string) {
    message = texte;
    setTimeout(() => {
      if (message === texte) message = null;
    }, 5000);
  }

  /** Format de conversion d'un nouveau fichier : celui choisi pour le lot, sinon le premier proposé. */
  function formatInitial(info: InfoFichier): FormatSortie | null {
    if (famille) {
      const cible = cibles[famille] as FormatSortie;
      return info.formatsSortie.includes(cible) ? cible : null;
    }
    return info.formatsSortie[0] ?? null;
  }

  async function ajouter(chemins: string[]) {
    if (chemins.length === 0) return;
    let infos = await analyserFichiers(chemins);

    // Sur une page dédiée, seuls les fichiers de la famille sont acceptés
    if (famille) {
      const ignores = infos.filter((i) => i.famille !== famille).length;
      infos = infos.filter((i) => i.famille === famille);
      if (ignores > 0) {
        afficherMessage(
          `${ignores} fichier${ignores > 1 ? "s" : ""} ignoré${ignores > 1 ? "s" : ""} : ce ne sont pas des ${FAMILLES[famille].nom}.`,
        );
      }
    }

    for (const info of infos) {
      // Un même fichier n'apparaît qu'une fois dans la file
      if (!elements.some((e) => e.info.chemin === info.chemin)) {
        elements.push({ info, format: formatInitial(info), etat: "attente" });
      }
    }
  }

  function retirer(chemin: string) {
    elements = elements.filter((e) => e.info.chemin !== chemin);
  }

  /** Remet en attente un fichier déjà traité (nouveau réglage, changement d'outil…). */
  function reinitialiser(element: ElementFile) {
    if (element.etat === "encours") return;
    element.etat = "attente";
    element.resultat = undefined;
    element.erreur = undefined;
  }

  function changerMode(nouveau: Mode) {
    if (enCours || nouveau === mode) return;
    mode = nouveau;
    elements.forEach(reinitialiser);
  }

  /** Nouveau format pour tout le lot de la page (onglet Conversion d'une famille). */
  function changerCible(cible: string) {
    if (!famille) return;
    cibles[famille] = cible;
    for (const element of visibles) {
      if (element.etat === "encours") continue;
      const format = cible as FormatSortie;
      element.format = element.info.formatsSortie.includes(format) ? format : null;
      reinitialiser(element);
    }
  }

  function changerFormat(chemin: string, format: FormatSortie) {
    const element = elements.find((e) => e.info.chemin === chemin);
    if (!element) return;
    element.format = format;
    reinitialiser(element);
  }

  function changerCompression(modification: Partial<OptionsCompression>) {
    Object.assign(compression, modification);
    elements.forEach(reinitialiser);
  }

  async function parcourir() {
    const choix = await open({
      multiple: true,
      title: "Choisir les fichiers",
      filters: famille
        ? [{ name: FAMILLES[famille].libelle, extensions: FAMILLES[famille].extensions }]
        : undefined,
    });
    if (choix) await ajouter(choix);
  }

  async function lancer() {
    const outil = mode;
    const aFaire = [...aTraiter];
    // Les réglages sont figés au lancement, même si on les modifie pendant le traitement
    const options = { ...compression };
    const qualite = reglages.qualite;
    enCours = true;

    /** Traite un fichier : image (cœur Rust) ou audio / vidéo (FFmpeg, avec progression). */
    async function traiter(element: ElementFile) {
      const dossier = await dossierPour(element.info.chemin);
      if (estMedia(element.info)) {
        element.progression = 0;
        return traiterMedia(element.info.chemin, dossier, operationMedia(element, outil, options), (p) => {
          element.progression = p;
        });
      }
      return outil === "convertir"
        ? convertirImage(element.info.chemin, dossier, element.format as FormatImage, qualite)
        : compresserImage(element.info.chemin, dossier, options);
    }

    /** Lance `paralleles` « travailleurs » qui prennent chacun le fichier suivant de `liste`. */
    async function traiterListe(liste: ElementFile[], paralleles: number) {
      let suivant = 0;
      const travailleur = async () => {
        while (suivant < liste.length) {
          const element = liste[suivant++];
          // Le fichier a pu être retiré de la file entre-temps
          if (!elements.some((e) => e.info.chemin === element.info.chemin)) continue;
          element.etat = "encours";
          element.erreur = undefined;
          try {
            element.resultat = await traiter(element);
            element.etat = "termine";
          } catch (erreur) {
            element.erreur = String(erreur);
            element.etat = "erreur";
          }
        }
      };
      await Promise.all(Array.from({ length: Math.min(paralleles, liste.length) }, travailleur));
    }

    // Images : plusieurs à la fois. Audio / vidéo : une à la fois (FFmpeg utilise déjà tous les cœurs).
    await Promise.all([
      traiterListe(aFaire.filter((e) => !estMedia(e.info)), TRAITEMENTS_PARALLELES),
      traiterListe(aFaire.filter((e) => estMedia(e.info)), 1),
    ]);
    enCours = false;
  }

  onMount(() => {
    // Destination par défaut : Documents\Allotrope (créé au premier fichier produit)
    initialiserDestination();

    // Tauri intercepte le glisser-déposer au niveau de la fenêtre et fournit les chemins complets.
    // Sur la page Outils PDF, les fichiers vont à l'outil ouvert (s'il en attend).
    const arret = getCurrentWebview().onDragDropEvent(({ payload }) => {
      if (page === "parametres" || (page === "pdf" && !depot.cible)) return;
      if (payload.type === "enter" || payload.type === "over") {
        depot.survol = true;
      } else if (payload.type === "leave") {
        depot.survol = false;
      } else if (payload.type === "drop") {
        depot.survol = false;
        if (page === "pdf") depot.cible?.(payload.paths);
        else ajouter(payload.paths);
      }
    });
    return () => {
      arret.then((desabonner) => desabonner());
    };
  });
</script>

{#if demarrage}
  <Splash onfin={() => (demarrage = false)} />
{/if}

<div class="application">
  <TitleBar parametresActif={page === "parametres"} onparametres={basculerParametres} />

  <div class="corps">
    <Sidebar {page} {compteurs} onnaviguer={naviguer} />

    <main class="contenu">
      {#if page === "parametres"}
        <Settings onfermer={basculerParametres} />
      {:else if page === "pdf"}
        <PdfTools />
      {:else}
        <div class="page">
          <div class="defilement">
            <header class="titre-page">
              <h1>
                {#if famille}
                  <span class="pastille" style:background={FAMILLES[famille].couleur}></span>
                  {FAMILLES[famille].libelle}
                {:else}
                  Que voulez-vous {mode === "convertir" ? "convertir" : "compresser"} ?
                {/if}
              </h1>
              <ModeSwitch {mode} onmode={changerMode} desactive={enCours} />
            </header>

            <OptionsPanel
              {mode}
              {famille}
              cible={famille ? cibles[famille] : ""}
              {compression}
              desactive={enCours}
              oncible={changerCible}
              oncompression={changerCompression}
            />

            {#if visibles.length === 0}
              <DropZone {mode} {famille} survol={depot.survol} onparcourir={parcourir} />
            {:else}
              <FileQueue
                elements={visibles}
                {mode}
                typeCompression={compression.type}
                survol={depot.survol}
                onajouter={parcourir}
                onretirer={retirer}
                onformat={changerFormat}
                onannuler={(chemin) => annulerMedia(chemin)}
                onouvrir={(chemin) => revealItemInDir(chemin)}
              />
            {/if}
          </div>

          {#if visibles.length > 0}
            <QueueFooter
              libelle={libelleLancement}
              desactive={enCours || aTraiter.length === 0}
              onlancer={lancer}
            />
          {/if}
        </div>
      {/if}

      {#if message}
        <div class="message" role="status">{message}</div>
      {/if}
    </main>
  </div>
</div>

<style>
  .application {
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .corps {
    flex: 1;
    min-height: 0;
    display: flex;
  }

  .contenu {
    position: relative;
    flex: 1;
    min-width: 0;
    min-height: 0;
  }

  .page {
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .defilement {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 32px 40px 32px;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .titre-page {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
  }

  h1 {
    margin: 0;
    font-size: 30px;
    font-weight: 900;
    letter-spacing: -0.3px;
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .pastille {
    width: 18px;
    height: 18px;
    border-radius: 6px;
    border: 2px solid var(--givre);
  }

  .message {
    position: absolute;
    left: 50%;
    bottom: 110px;
    transform: translateX(-50%);
    max-width: calc(100% - 80px);
    padding: 12px 20px;
    border-radius: 999px;
    border: 2px solid var(--givre);
    background: var(--surface-active);
    font-size: 14px;
    font-weight: 700;
  }
</style>
