<!-- Carte d'un fichier dans la file d'attente : choix du format (conversion) et état du traitement. -->
<script lang="ts">
  import { ArrowRight, Check, ChevronDown, Folder, LoaderCircle, X } from "@lucide/svelte";
  import {
    FAMILLES,
    LIBELLES_FORMAT,
    formaterTaille,
    traitable,
    type ElementFile,
    type FormatImage,
    type Mode,
    type TypeCompression,
  } from "./fichiers";

  interface Props {
    element: ElementFile;
    mode: Mode;
    /** Réglage de compression utilisé (pour le message « déjà sous la cible »). */
    typeCompression: TypeCompression;
    onformat: (format: FormatImage) => void;
    onretirer: () => void;
    onouvrir: () => void;
  }
  let { element, mode, typeCompression, onformat, onretirer, onouvrir }: Props = $props();

  const info = $derived(element.info);
  const famille = $derived(info.famille ? FAMILLES[info.famille] : null);
  // Le badge affiche au plus 4 lettres (« JPEG », « DOCX »)
  const badge = $derived(info.extension.slice(0, 4).toUpperCase() || "?");
  const actif = $derived(traitable(element, mode));
  const dejaOptimise = $derived(element.etat === "termine" && element.resultat?.chemin === null);

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
    return info.dimensions ? `${taille} · ${info.dimensions[0]} × ${info.dimensions[1]}` : taille;
  });

  // Message affiché quand le fichier ne peut pas être traité par l'outil actif
  const indisponible = $derived.by(() => {
    if (!famille) return "Format non pris en charge";
    // Image convertible, mais le format demandé pour le lot est déjà le sien
    if (mode === "convertir" && info.formatsSortie.length > 0) return "Déjà dans ce format";
    if (info.famille !== "image") return "Bientôt disponible";
    return "Pas encore pris en charge";
  });
</script>

<div class="carte" class:inactive={!actif}>
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
        onchange={(e) => onformat(e.currentTarget.value as FormatImage)}
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
    <button class="carre" aria-label="Afficher dans le dossier" onclick={onouvrir}>
      <Folder size={18} strokeWidth={2.4} />
    </button>
  {:else if element.etat === "encours"}
    <div class="etat">
      <span class="rotation"><LoaderCircle size={16} strokeWidth={2.6} /></span>
      {mode === "convertir" ? "Conversion…" : "Compression…"}
    </div>
  {:else if element.etat === "erreur"}
    <div class="etat echec">Échec</div>
  {:else}
    <div class="etat">{actif ? "En attente" : indisponible}</div>
  {/if}

  {#if element.etat !== "encours"}
    <button class="carre" aria-label={`Retirer ${info.nom}`} onclick={onretirer}>
      <X size={18} strokeWidth={2.4} />
    </button>
  {/if}
</div>

<style>
  .carte {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 16px;
    border-radius: 22px;
    border: 2px solid var(--ligne);
    background: var(--surface);
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
