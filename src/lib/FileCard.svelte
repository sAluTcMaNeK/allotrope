<!-- Carte d'un fichier dans la file d'attente : choix du format (conversion) et état du traitement. -->
<script lang="ts">
  import { ArrowRight, Check, ChevronDown, Columns2, Folder, LoaderCircle, X } from "@lucide/svelte";
  import {
    FAMILLES,
    LIBELLES_FORMAT,
    estMedia,
    formaterTaille,
    traitable,
    type ElementFile,
    type FormatSortie,
    type Mode,
    type TypeCompression,
  } from "./fichiers";
  import { EXTENSIONS_DOCUMENT } from "./pdf/pdf";

  interface Props {
    element: ElementFile;
    mode: Mode;
    /** Réglage de compression utilisé (pour le message « déjà sous la cible »). */
    typeCompression: TypeCompression;
    onformat: (format: FormatSortie) => void;
    onretirer: () => void;
    onouvrir: () => void;
    /** Ouvre l'aperçu avant / après. */
    onapercu: () => void;
    /** Arrête le traitement en cours (audio / vidéo). */
    onannuler: () => void;
  }
  let { element, mode, typeCompression, onformat, onretirer, onouvrir, onapercu, onannuler }: Props = $props();

  const info = $derived(element.info);
  const famille = $derived(info.famille ? FAMILLES[info.famille] : null);
  // Le badge affiche au plus 4 lettres (« JPEG », « DOCX »)
  const badge = $derived(info.extension.slice(0, 4).toUpperCase() || "?");
  const actif = $derived(traitable(element, mode));
  const dejaOptimise = $derived(element.etat === "termine" && element.resultat?.chemin === null);
  const media = $derived(estMedia(info));
  const annule = $derived(element.etat === "erreur" && element.erreur === "Annulé");
  /** Barre de progression : couleur de la famille (Améthyste clair pour la vidéo, voir DESIGN.md). */
  const couleurProgression = $derived(info.famille === "video" ? "var(--amethyste-clair)" : famille?.couleur);
  const pourcentage = $derived(Math.round(element.progression ?? 0));

  /** « 3 min 12 s », « 1 h 05 min ». */
  function dureeLisible(secondes: number): string {
    const s = Math.round(secondes);
    if (s >= 3600) return `${Math.floor(s / 3600)} h ${String(Math.floor((s % 3600) / 60)).padStart(2, "0")} min`;
    if (s >= 60) return `${Math.floor(s / 60)} min ${s % 60} s`;
    return `${s} s`;
  }

  const meta = $derived.by(() => {
    const taille = formaterTaille(info.taille);
    const resultat = element.resultat;
    if (element.etat === "termine" && resultat) {
      if (resultat.chemin === null) {
        return typeCompression === "taille" ? `${taille} · aucune compression nécessaire` : `${taille} · aucun gain possible`;
      }
      if (mode === "convertir" && element.format) {
        return `${taille} → ${LIBELLES_FORMAT[element.format]} · ${formaterTaille(resultat.taille)}`;
      }
      const gain = Math.round((1 - resultat.taille / info.taille) * 100);
      return `${taille} → ${formaterTaille(resultat.taille)} · −${gain} %`;
    }
    const details = [taille];
    if (info.dimensions) details.push(`${info.dimensions[0]} × ${info.dimensions[1]}`);
    if (info.duree) details.push(dureeLisible(info.duree));
    return details.join(" · ");
  });

  // Message affiché quand le fichier ne peut pas être traité par l'outil actif
  const indisponible = $derived.by(() => {
    if (info.extension === "pdf") return "Voir les Outils PDF";
    if (EXTENSIONS_DOCUMENT.includes(info.extension)) return "Voir Outils PDF › Word en PDF";
    if (!famille) return "Format non pris en charge";
    // Fichier convertible, mais le format demandé pour le lot est déjà le sien
    if (mode === "convertir" && info.formatsSortie.length > 0) return "Déjà dans ce format";
    // WAV, FLAC… : les alléger, c'est les convertir (MP3, M4A, OPUS)
    if (mode === "compresser" && info.famille === "audio") return "Sans perte : à convertir";
    return "Pas encore pris en charge";
  });
</script>

<div class="carte" class:inactive={!actif}>
  <div class="ligne">
  <div
    class="badge"
    class:inconnu={!famille}
    style:background={famille?.couleur}
    style:color={famille?.texte}
  >
    {badge}
  </div>

  <div class="infos">
    <div class="nom" title={info.chemin}>{info.nom}</div>
    <div class="meta">{meta}</div>
    {#if element.etat === "erreur"}
      <div class="erreur" title={element.erreur}>{element.erreur}</div>
    {/if}
  </div>

  {#if mode === "convertir" && info.formatsSortie.length > 0 && element.etat !== "termine"}
    <span class="fleche"><ArrowRight size={20} strokeWidth={2.6} /></span>
    <label class="selecteur">
      <select
        aria-label="Format de sortie"
        value={element.format ?? ""}
        disabled={element.etat === "encours"}
        onchange={(e) => onformat(e.currentTarget.value as FormatSortie)}
      >
        {#if !element.format}
          <option value="" disabled>Choisir…</option>
        {/if}
        {#each info.formatsSortie as format (format)}
          <option value={format}>{LIBELLES_FORMAT[format]}</option>
        {/each}
      </select>
      <span class="chevron"><ChevronDown size={14} strokeWidth={3} /></span>
    </label>
  {/if}

  {#if dejaOptimise}
    <div class="pastille neutre">{typeCompression === "taille" ? "Déjà sous la cible" : "Déjà optimisé"}</div>
  {:else if element.etat === "termine"}
    <div class="pastille succes">
      <Check size={16} strokeWidth={3} />
      Terminé
    </div>
    <button class="carre" aria-label="Aperçu avant / après" title="Aperçu avant / après" onclick={onapercu}>
      <Columns2 size={18} strokeWidth={2.4} />
    </button>
    <button class="carre" aria-label="Afficher dans le dossier" onclick={onouvrir}>
      <Folder size={18} strokeWidth={2.4} />
    </button>
  {:else if element.etat === "encours" && media}
    <div class="etat fort">{pourcentage} %</div>
  {:else if element.etat === "encours"}
    <div class="etat">
      <span class="rotation"><LoaderCircle size={16} strokeWidth={2.6} /></span>
      {mode === "convertir" ? "Conversion…" : "Compression…"}
    </div>
  {:else if annule}
    <div class="etat">Annulé</div>
  {:else if element.etat === "erreur"}
    <div class="etat echec">Échec</div>
  {:else}
    <div class="etat">{actif ? "En attente" : indisponible}</div>
  {/if}

  {#if element.etat !== "encours"}
    <button class="carre" aria-label={`Retirer ${info.nom}`} onclick={onretirer}>
      <X size={18} strokeWidth={2.4} />
    </button>
  {:else if media}
    <button class="carre" aria-label={`Annuler le traitement de ${info.nom}`} onclick={onannuler}>
      <X size={18} strokeWidth={2.4} />
    </button>
  {/if}
  </div>

  {#if element.etat === "encours" && media}
    <!-- Barre de progression de la charte : 12 px, fond Ligne, remplissage couleur de la famille -->
    <div
      class="progression"
      role="progressbar"
      aria-label={`Avancement de ${info.nom}`}
      aria-valuemin="0"
      aria-valuemax="100"
      aria-valuenow={pourcentage}
    >
      <div class="remplissage" style:width="{element.progression ?? 0}%" style:background={couleurProgression}></div>
    </div>
  {/if}
</div>

<style>
  .carte {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    border-radius: 22px;
    border: 2px solid var(--ligne);
    background: var(--surface);
  }

  .ligne {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .progression {
    height: 12px;
    border-radius: 999px;
    background: var(--ligne);
    overflow: hidden;
  }

  .remplissage {
    height: 100%;
    border-radius: 999px;
    transition: width 400ms linear;
  }

  /* Fichier que l'outil actif ne sait pas (encore) traiter */
  .carte.inactive .badge,
  .carte.inactive .infos {
    opacity: 0.6;
  }

  .badge {
    width: 48px;
    height: 48px;
    flex: none;
    border-radius: 14px;
    border: 2px solid var(--givre);
    font-size: 12px;
    font-weight: 900;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  /* Format non reconnu : pas de couleur de famille */
  .badge.inconnu {
    background: var(--ligne-forte);
    color: var(--givre);
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

  .fleche {
    display: flex;
    color: var(--texte-secondaire);
  }

  /* Sélecteur de format : liste native habillée comme le bouton de la maquette */
  .selecteur {
    position: relative;
    display: flex;
    align-items: center;
  }

  select {
    appearance: none;
    height: 44px;
    padding: 0 36px 0 16px;
    border-radius: 14px;
    border: 2px solid var(--ligne-forte);
    background: var(--abysse);
    color: var(--givre);
    font-family: inherit;
    font-size: 14px;
    font-weight: 800;
    cursor: pointer;
    transition: border-color var(--transition);
  }

  select:hover:not(:disabled) {
    border-color: var(--azur);
  }

  select:disabled {
    cursor: default;
    opacity: 0.6;
  }

  option {
    background: var(--surface);
    color: var(--givre);
  }

  .chevron {
    position: absolute;
    right: 14px;
    display: flex;
    pointer-events: none;
  }

  .etat {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 88px;
    justify-content: flex-end;
    font-size: 13px;
    font-weight: 700;
    color: var(--texte-secondaire);
    text-align: right;
  }

  /* Pourcentage d'avancement, bien lisible */
  .etat.fort {
    font-size: 14px;
    font-weight: 900;
    color: var(--givre);
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
