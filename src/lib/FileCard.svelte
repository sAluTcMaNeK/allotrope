<!-- Carte d'un fichier dans la file d'attente. -->
<script lang="ts">
  import { X } from "@lucide/svelte";
  import { FAMILLES, formaterTaille, type InfoFichier } from "./fichiers";

  interface Props {
    fichier: InfoFichier;
    onretirer: () => void;
  }
  let { fichier, onretirer }: Props = $props();

  const famille = $derived(fichier.famille ? FAMILLES[fichier.famille] : null);
  // Le badge affiche au plus 4 lettres (« JPEG », « DOCX »)
  const badge = $derived(fichier.extension.slice(0, 4).toUpperCase() || "?");
</script>

<div class="carte">
  <div
    class="badge"
    class:inconnu={!famille}
    style:background={famille?.couleur}
    style:color={famille?.texte}
  >
    {badge}
  </div>

  <div class="infos">
    <div class="nom" title={fichier.chemin}>{fichier.nom}</div>
    <div class="meta">{formaterTaille(fichier.taille)}</div>
  </div>

  <div class="etat">{famille ? "En attente" : "Format non pris en charge"}</div>

  <button class="retirer" aria-label={`Retirer ${fichier.nom}`} onclick={onretirer}>
    <X size={18} strokeWidth={2.4} />
  </button>
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

  .etat {
    font-size: 13px;
    font-weight: 700;
    color: var(--texte-secondaire);
    text-align: right;
  }

  .retirer {
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

  .retirer:hover {
    border-color: var(--ligne-forte);
  }
</style>
