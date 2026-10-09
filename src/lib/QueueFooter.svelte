<!-- Pied de la file d'attente : dossier de destination et bouton principal de lancement. -->
<script lang="ts">
  import { ArrowRight, Folder } from "@lucide/svelte";
  import type { Mode } from "./fichiers";

  interface Props {
    mode: Mode;
    /** Nombre de fichiers que l'outil actif va traiter. */
    aTraiter: number;
    enCours: boolean;
    /** Destination lisible (« Documents › Allotrope », « À côté des originaux »…). */
    destination: string;
    onmodifierdestination: () => void;
    onlancer: () => void;
  }
  let { mode, aTraiter, enCours, destination, onmodifierdestination, onlancer }: Props = $props();

  const libelle = $derived.by(() => {
    if (enCours) return mode === "convertir" ? "Conversion en cours…" : "Compression en cours…";
    if (aTraiter === 0) return "Rien à traiter";
    const verbe = mode === "convertir" ? "Convertir" : "Compresser";
    return `${verbe} ${aTraiter} fichier${aTraiter > 1 ? "s" : ""}`;
  });
</script>

<footer class="pied">
  <div class="destination">
    <span class="icone"><Folder size={20} strokeWidth={2.4} /></span>
    <span class="libelle">Enregistrer dans</span>
    <span class="chemin" title={destination}>{destination}</span>
    <button class="lien" onclick={onmodifierdestination}>Modifier</button>
  </div>
  <button class="principal" disabled={enCours || aTraiter === 0} onclick={onlancer}>
    {libelle}
    <ArrowRight size={18} strokeWidth={3} />
  </button>
</footer>

<style>
  .pied {
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
    padding: 16px 40px;
    border-top: 2px solid var(--ligne);
    background: var(--surface);
  }

  .destination {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
    font-size: 14px;
  }

  .icone {
    display: flex;
    color: var(--texte-secondaire);
  }

  .libelle {
    font-weight: 600;
    color: var(--texte-secondaire);
    white-space: nowrap;
  }

  .chemin {
    font-weight: 800;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .lien {
    border: none;
    background: none;
    padding: 0;
    min-height: 44px;
    color: var(--lien);
    font-size: 14px;
    font-weight: 800;
    text-decoration: underline;
    cursor: pointer;
  }

  .lien:hover {
    color: var(--givre);
  }

  /* Bouton « Principal » de la charte : le seul appel à l'action Braise de l'écran */
  .principal {
    height: 54px;
    padding: 0 30px;
    border-radius: 999px;
    border: 3px solid var(--givre);
    background: var(--braise);
    color: var(--abysse);
    font-size: 17px;
    font-weight: 900;
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
    transition: opacity var(--transition);
  }

  .principal:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>
