<!-- Ligne d'une liste ordonnée de fichiers (fusion, images en PDF) : monter, descendre, retirer. -->
<script lang="ts">
  import { ArrowDown, ArrowUp, X } from "@lucide/svelte";

  interface Props {
    position: number;
    badge: string;
    couleur: string;
    texte: string;
    nom: string;
    meta: string;
    /** Message d'erreur (PDF protégé, illisible…). */
    erreur?: string;
    premier: boolean;
    dernier: boolean;
    desactive: boolean;
    onmonter: () => void;
    ondescendre: () => void;
    onretirer: () => void;
  }
  let {
    position,
    badge,
    couleur,
    texte,
    nom,
    meta,
    erreur,
    premier,
    dernier,
    desactive,
    onmonter,
    ondescendre,
    onretirer,
  }: Props = $props();
</script>

<div class="ligne" class:en-erreur={erreur}>
  <span class="position">{position}</span>
  <div class="badge" style:background={couleur} style:color={texte}>{badge}</div>
  <div class="infos">
    <div class="nom" title={nom}>{nom}</div>
    {#if erreur}
      <div class="erreur">{erreur}</div>
    {:else}
      <div class="meta">{meta}</div>
    {/if}
  </div>
  <button class="carre" aria-label={`Monter ${nom}`} disabled={premier || desactive} onclick={onmonter}>
    <ArrowUp size={18} strokeWidth={2.4} />
  </button>
  <button class="carre" aria-label={`Descendre ${nom}`} disabled={dernier || desactive} onclick={ondescendre}>
    <ArrowDown size={18} strokeWidth={2.4} />
  </button>
  <button class="carre" aria-label={`Retirer ${nom}`} disabled={desactive} onclick={onretirer}>
    <X size={18} strokeWidth={2.4} />
  </button>
</div>

<style>
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

  .position {
    width: 24px;
    flex: none;
    text-align: center;
    font-size: 14px;
    font-weight: 900;
    color: var(--texte-secondaire);
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

  .infos {
    flex: 1;
    min-width: 0;
  }

  .nom {
    font-size: 15px;
    font-weight: 800;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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

  .carre:hover:not(:disabled) {
    border-color: var(--ligne-forte);
  }

  .carre:disabled {
    opacity: 0.35;
    cursor: default;
  }
</style>
