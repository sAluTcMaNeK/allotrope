<!--
  Outils « Réorganiser les pages » et « Supprimer des pages » : les miniatures du PDF s'affichent,
  on les déplace / fait pivoter, ou on clique sur celles à retirer. Le PDF d'origine n'est jamais modifié.
-->
<script lang="ts">
  import { onMount, tick } from "svelte";
  import { flip } from "svelte/animate";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { PDFDocumentProxy } from "pdfjs-dist";
  import { GripVertical, RotateCw, Trash, Undo2 } from "@lucide/svelte";
  import ToolLayout from "./ToolLayout.svelte";
  import DepotPdf from "./DepotPdf.svelte";
  import Miniature from "./Miniature.svelte";
  import ResultatPdf from "./ResultatPdf.svelte";
  import QueueFooter from "../QueueFooter.svelte";
  import { recevoirDepots } from "../depot.svelte";
  import { dossierPour } from "../destination.svelte";
  import { assemblerPdf, nomSansExtension, ouvrirPdf, type ResultatPdf as Resultat } from "./pdf";

  interface Props {
    mode: "reorganiser" | "supprimer";
    onretour: () => void;
  }
  let { mode, onretour }: Props = $props();

  interface Page {
    /** Numéro dans le PDF d'origine (sert aussi d'identifiant). */
    numero: number;
    rotation: number;
    supprimee: boolean;
  }

  const LARGEUR_MINIATURE = 140;

  let chemin = $state<string | null>(null);
  let pdf = $state.raw<PDFDocumentProxy | null>(null);
  let pages = $state<Page[]>([]);
  let chargement = $state(false);
  let enCours = $state(false);
  let resultat = $state<Resultat | null>(null);
  let erreur = $state<string | null>(null);

  const nom = $derived(chemin ? (chemin.split(/[\\/]/).pop() ?? "") : "");
  const aSupprimer = $derived(pages.filter((p) => p.supprimee).length);
  const modifie = $derived(
    pages.some((p, i) => p.numero !== i + 1 || p.rotation % 360 !== 0 || p.supprimee),
  );

  const libelleBouton = $derived.by(() => {
    if (enCours) return "Création du PDF…";
    if (mode === "reorganiser") return "Enregistrer le PDF";
    if (aSupprimer === 0) return "Sélectionnez des pages";
    if (aSupprimer === pages.length) return "Gardez au moins une page";
    return `Supprimer ${aSupprimer} page${aSupprimer > 1 ? "s" : ""}`;
  });
  const pret = $derived(
    !enCours && modifie && pages.length > 0 && (mode === "reorganiser" || aSupprimer < pages.length),
  );

  async function charger(nouveau: string) {
    erreur = null;
    resultat = null;
    chargement = true;
    try {
      const document = await ouvrirPdf(nouveau);
      await pdf?.loadingTask.destroy();
      pdf = document;
      chemin = nouveau;
      pages = Array.from({ length: document.numPages }, (_, i) => ({
        numero: i + 1,
        rotation: 0,
        supprimee: false,
      }));
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

  function retablir() {
    pages.sort((a, b) => a.numero - b.numero);
    pages.forEach((p) => {
      p.rotation = 0;
      p.supprimee = false;
    });
  }

  function deplacer(numero: number, vers: number) {
    const de = pages.findIndex((p) => p.numero === numero);
    if (vers < 0 || vers >= pages.length || de === vers) return;
    const [page] = pages.splice(de, 1);
    pages.splice(vers, 0, page);
  }

  // --- Glisser une page pour la déplacer (événements du pointeur) ---
  // Tauri désactive le glisser-déposer HTML dans la fenêtre (il intercepte les fichiers déposés) :
  // une copie de la page (le « fantôme ») suit donc le curseur, et un emplacement en pointillés
  // montre où elle sera déposée.

  /** Distance (px) à parcourir avant que le déplacement commence : un simple clic ne déplace rien. */
  const SEUIL = 6;
  /** Hauteur (px) des bandes en haut et en bas de la zone qui déclenchent le défilement automatique. */
  const BANDE_DEFILEMENT = 80;

  let grille = $state<HTMLElement>();
  let canvasFantome = $state<HTMLCanvasElement>();
  /** Page en cours de déplacement (son numéro). */
  let glissee = $state<number | null>(null);
  /** Position du fantôme : coin haut-gauche suivant le curseur, et taille de la page d'origine. */
  let fantome = $state<{ x: number; y: number; largeur: number } | null>(null);
  const pageGlissee = $derived(pages.find((p) => p.numero === glissee));

  /** Appui en cours, pas encore assez déplacé pour commencer le glissement. */
  let appui: { numero: number; x: number; y: number; element: HTMLElement } | null = null;
  /** Décalage entre le curseur et le coin de la page saisie (la page ne « saute » pas sous le curseur). */
  let decalage = { x: 0, y: 0 };
  let ordreInitial: Page[] = [];
  let pointeur = { x: 0, y: 0 };
  let pause = false;
  let defilement: HTMLElement | null = null;
  let boucle = 0;

  function debutGlisser(e: PointerEvent, numero: number) {
    if (mode !== "reorganiser" || e.button !== 0) return;
    if ((e.target as Element).closest("button")) return;
    const element = e.currentTarget as HTMLElement;
    appui = { numero, x: e.clientX, y: e.clientY, element };
    element.setPointerCapture(e.pointerId);
  }

  /** Premier conteneur défilant au-dessus de la grille (la zone de contenu de l'outil). */
  function conteneurDefilant(element: HTMLElement | undefined): HTMLElement | null {
    let parent = element?.parentElement ?? null;
    while (parent) {
      const style = getComputedStyle(parent).overflowY;
      if ((style === "auto" || style === "scroll") && parent.scrollHeight > parent.clientHeight) return parent;
      parent = parent.parentElement;
    }
    return null;
  }

  async function commencer() {
    if (!appui) return;
    const { numero, element } = appui;
    const cadre = element.getBoundingClientRect();
    decalage = { x: appui.x - cadre.left, y: appui.y - cadre.top };
    fantome = { x: pointeur.x - decalage.x, y: pointeur.y - decalage.y, largeur: cadre.width };
    ordreInitial = [...pages];
    glissee = numero;
    appui = null;
    defilement = conteneurDefilant(grille);
    boucle = requestAnimationFrame(defilerAuBord);

    // Le fantôme reprend l'image déjà dessinée de la miniature, sans redemander le rendu à pdf.js
    await tick();
    const source = element.querySelector("canvas");
    if (source && canvasFantome) {
      canvasFantome.width = source.width;
      canvasFantome.height = source.height;
      canvasFantome.style.width = `${source.offsetWidth}px`;
      canvasFantome.style.height = `${source.offsetHeight}px`;
      canvasFantome.getContext("2d")?.drawImage(source, 0, 0);
    }
  }

  function glisser(e: PointerEvent) {
    pointeur = { x: e.clientX, y: e.clientY };
    if (appui && Math.hypot(e.clientX - appui.x, e.clientY - appui.y) > SEUIL) commencer();
    if (glissee === null || !fantome) return;
    fantome.x = e.clientX - decalage.x;
    fantome.y = e.clientY - decalage.y;
    placer();
  }

  /** Déplace la page vers l'emplacement de la page survolée par le curseur. */
  function placer() {
    if (glissee === null || pause) return;
    // Le fantôme ignore le pointeur (pointer-events: none) : on trouve la page qui est dessous
    const cible = document
      .elementFromPoint(pointeur.x, pointeur.y)
      ?.closest<HTMLElement>("[data-numero]");
    if (!cible) return;
    const vers = pages.findIndex((p) => p.numero === Number(cible.dataset.numero));
    const de = pages.findIndex((p) => p.numero === glissee);
    if (vers === -1 || vers === de) return;
    deplacer(glissee, vers);
    // Laisse l'animation des autres pages se terminer avant le déplacement suivant (évite les allers-retours)
    pause = true;
    setTimeout(() => {
      pause = false;
      placer();
    }, 180);
  }

  /** Fait défiler la zone quand le curseur approche de son bord haut ou bas (longs documents). */
  function defilerAuBord() {
    if (glissee === null) return;
    if (defilement) {
      const zone = defilement.getBoundingClientRect();
      let vitesse = 0;
      if (pointeur.y < zone.top + BANDE_DEFILEMENT) vitesse = pointeur.y - (zone.top + BANDE_DEFILEMENT);
      else if (pointeur.y > zone.bottom - BANDE_DEFILEMENT) vitesse = pointeur.y - (zone.bottom - BANDE_DEFILEMENT);
      if (vitesse !== 0) {
        // Plus le curseur est près du bord, plus le défilement est rapide
        defilement.scrollBy(0, Math.max(-28, Math.min(28, vitesse / 3)));
        placer();
      }
    }
    boucle = requestAnimationFrame(defilerAuBord);
  }

  function finGlisser() {
    appui = null;
    glissee = null;
    fantome = null;
    cancelAnimationFrame(boucle);
  }

  /** Échap pendant un déplacement : la page retourne à sa place d'origine. */
  function annulerGlisser(e: KeyboardEvent) {
    if (e.key !== "Escape" || glissee === null) return;
    pages = ordreInitial;
    finGlisser();
  }

  function clavier(e: KeyboardEvent, page: Page, index: number) {
    if (mode !== "reorganiser") return;
    if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
      e.preventDefault();
      deplacer(page.numero, index + (e.key === "ArrowLeft" ? -1 : 1));
      // Garde le focus sur la page déplacée
      requestAnimationFrame(() =>
        document.querySelector<HTMLElement>(`[data-numero="${page.numero}"]`)?.focus(),
      );
    } else if (e.key.toLowerCase() === "r") {
      page.rotation += 90;
    }
  }

  async function enregistrer() {
    if (!chemin) return;
    enCours = true;
    erreur = null;
    try {
      const gardees = pages
        .filter((p) => !p.supprimee)
        .map((p) => ({ source: 0, page: p.numero, rotation: p.rotation % 360 }));
      const suffixe = mode === "reorganiser" ? "réorganisé" : "pages supprimées";
      resultat = await assemblerPdf(
        [chemin],
        gardees,
        await dossierPour(chemin),
        `${nomSansExtension(chemin)} (${suffixe})`,
      );
    } catch (e) {
      erreur = String(e);
    } finally {
      enCours = false;
    }
  }

  async function recommencer() {
    await pdf?.loadingTask.destroy();
    pdf = null;
    chemin = null;
    pages = [];
    resultat = null;
  }

  onMount(() => {
    const arret = recevoirDepots(recevoir);
    return () => {
      arret();
      pdf?.loadingTask.destroy();
    };
  });
</script>

<svelte:window
  onpointermove={glisser}
  onpointerup={finGlisser}
  onpointercancel={finGlisser}
  onkeydown={annulerGlisser}
/>

<!-- Copie de la page déplacée, qui suit le curseur -->
{#if fantome && pageGlissee}
  <div
    class="page fantome"
    style:width="{fantome.largeur}px"
    style:transform="translate({fantome.x}px, {fantome.y}px) rotate(2deg) scale(1.04)"
    aria-hidden="true"
  >
    <div class="apercu">
      <div class="cadre-fantome" style:width="{LARGEUR_MINIATURE}px" style:height="{LARGEUR_MINIATURE * 1.3}px">
        <canvas bind:this={canvasFantome} style:transform="rotate({pageGlissee.rotation}deg)"></canvas>
      </div>
    </div>
    <div class="pied-page">
      <span class="poignee"><GripVertical size={16} strokeWidth={2.4} /></span>
      <span class="numero">Page {pageGlissee.numero}</span>
    </div>
  </div>
{/if}

<ToolLayout
  titre={mode === "reorganiser" ? "Réorganiser les pages" : "Supprimer des pages"}
  description={mode === "reorganiser"
    ? "Changez l'ordre des pages d'un PDF et faites-les pivoter."
    : "Retirez les pages inutiles d'un PDF."}
  {onretour}
>
  {#if resultat}
    <ResultatPdf {resultat} onrecommencer={recommencer} />
  {/if}
  {#if erreur}
    <p class="erreur" role="alert">{erreur}</p>
  {/if}

  {#if !pdf}
    <DepotPdf
      titre={chargement ? "Ouverture du PDF…" : "Glissez un PDF ici"}
      aide="Les miniatures des pages s'afficheront ici. Le fichier d'origine ne sera pas modifié."
      formats="PDF"
      onparcourir={parcourir}
    />
  {:else}
    <div class="barre">
      <div class="fichier">
        <div class="nom" title={chemin}>{nom}</div>
        <div class="aide">
          {#if mode === "reorganiser"}
            Faites glisser les pages pour changer leur ordre (Échap pour annuler, ou flèches ← → du
            clavier), et <RotateCw size={13} strokeWidth={2.6} /> pour les faire pivoter.
          {:else}
            Cliquez sur les pages à supprimer. {aSupprimer} sur {pages.length} sélectionnée{aSupprimer > 1 ? "s" : ""}.
          {/if}
        </div>
      </div>
      <button class="secondaire" disabled={!modifie || enCours} onclick={retablir}>
        <Undo2 size={16} strokeWidth={2.6} />
        Rétablir
      </button>
      <button class="secondaire" disabled={enCours} onclick={parcourir}>Changer de PDF</button>
    </div>

    <div class="grille" class:glissement={glissee !== null} bind:this={grille}>
      {#each pages as page, index (page.numero)}
        <div
          class="page"
          class:supprimee={page.supprimee}
          class:glissee={glissee === page.numero}
          class:selectionnable={mode === "supprimer"}
          data-numero={page.numero}
          role="button"
          aria-roledescription={mode === "reorganiser" ? "page déplaçable" : undefined}
          aria-pressed={mode === "supprimer" ? page.supprimee : undefined}
          aria-label={`Page ${page.numero}`}
          tabindex="0"
          animate:flip={{ duration: 180 }}
          onpointerdown={(e) => debutGlisser(e, page.numero)}
          onclick={() => mode === "supprimer" && (page.supprimee = !page.supprimee)}
          onkeydown={(e) => {
            if (mode === "supprimer" && (e.key === " " || e.key === "Enter")) {
              e.preventDefault();
              page.supprimee = !page.supprimee;
            } else {
              clavier(e, page, index);
            }
          }}
        >
          <div class="apercu">
            <Miniature document={pdf} numero={page.numero} rotation={page.rotation} largeur={LARGEUR_MINIATURE} />
            {#if page.supprimee}
              <span class="corbeille"><Trash size={28} strokeWidth={2.4} /></span>
            {/if}
          </div>
          <div class="pied-page">
            {#if mode === "reorganiser"}
              <span class="poignee"><GripVertical size={16} strokeWidth={2.4} /></span>
            {/if}
            <span class="numero">Page {page.numero}</span>
            {#if mode === "reorganiser"}
              <button
                class="tourner"
                aria-label={`Faire pivoter la page ${page.numero}`}
                onclick={() => (page.rotation += 90)}
              >
                <RotateCw size={16} strokeWidth={2.4} />
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}

  {#snippet pied()}
    {#if pdf}
      <QueueFooter libelle={libelleBouton} desactive={!pret} onlancer={enregistrer} />
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

  .aide :global(svg) {
    vertical-align: -2px;
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
    grid-template-columns: repeat(auto-fill, minmax(172px, 1fr));
    gap: 16px;
  }

  .grille.glissement {
    cursor: grabbing;
  }

  .page {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px 14px 10px;
    border-radius: 22px;
    border: 2px solid var(--ligne);
    background: var(--surface);
    cursor: grab;
    touch-action: none;
    transition:
      border-color var(--transition),
      background-color var(--transition),
      opacity var(--transition);
  }

  .page.selectionnable {
    cursor: pointer;
  }

  .page:hover {
    border-color: var(--ligne-forte);
  }

  /* Emplacement laissé par la page déplacée : là où elle sera déposée */
  .page.glissee {
    border: 2px dashed var(--givre);
    background: transparent;
  }

  .page.glissee > * {
    visibility: hidden;
  }

  /* Fantôme : copie de la page qui suit le curseur, mise en avant (contour 3 px Givre de la charte) */
  .page.fantome {
    position: fixed;
    top: 0;
    left: 0;
    z-index: 50;
    border: 3px solid var(--givre);
    background: var(--surface-active);
    cursor: grabbing;
    pointer-events: none;
    transform-origin: top left;
  }

  .cadre-fantome {
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }

  .cadre-fantome canvas {
    border-radius: 6px;
    background: var(--blanc);
  }

  /* Page à supprimer : contour Braise et aperçu atténué */
  .page.supprimee {
    border-color: var(--braise);
  }

  .page.supprimee .apercu :global(.cadre) {
    opacity: 0.3;
  }

  .apercu {
    position: relative;
    display: flex;
    justify-content: center;
  }

  .corbeille {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--braise);
  }

  .pied-page {
    display: flex;
    align-items: center;
    gap: 6px;
    min-height: 44px;
  }

  .poignee {
    display: flex;
    color: var(--texte-secondaire);
  }

  .numero {
    flex: 1;
    font-size: 13px;
    font-weight: 800;
  }

  /* Bouton « Icône seule » de la charte */
  .tourner {
    width: 44px;
    height: 44px;
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

  .tourner:hover {
    border-color: var(--ligne-forte);
  }
</style>
